use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use futures_lite::StreamExt;
use iroh::base::node_addr::AddrInfoOptions::{Id, RelayAndAddresses};
use iroh::base::ticket;
use iroh::blobs::format::collection::Collection;
use iroh::blobs::Hash;
use iroh::blobs::util::SetTagOption;
use iroh::client::blobs::{BlobStatus, WrapOption};
use iroh::client::docs::Entry;
use iroh::client::docs::ShareMode::Write;
use iroh::docs::{Capability, DocTicket};
use iroh::docs::store::{Query, SortBy, SortDirection};
use iroh::net::NodeAddr;
use tokio::fs::DirEntry;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;

use crate::ble::{AddressData, BLEGossipBroadcaster, BLEGossipScanner, DocumentData, GossipScannerDelegate, PeerData};
use crate::ble::PeerState::{Scanning, Settled};
use crate::blob_dispatcher::{CollectionState, LoadCollectionDelegate};
use crate::data::{BlobHash, PublicKey, UUID};
use crate::device::DeviceApiServiceProvider;
use crate::doc::{Doc, key_of, Node};
use crate::events::{broadcast, create_broadcast, Subscribable};
use crate::identity::IdentityService;
use crate::identity::model::{ID_PIC, IDENTITY, Identity, IdentityServiceEvents};
use crate::nearby::model::{DebugState, DisplayMessage, message_key, message_payload_key, NearbyProfile, Post, Status};
use crate::nearby::NearbyServiceEvents::{AllMessagesUpdated, DebugStateUpdated, IdentitiesUpdated, ReceivedOneNewMessage, ScanningUpdated};
use crate::nearby::peer_calc::{collect_addrs_for_doc, find_best_doc_from_peers};
use crate::nearby::State::{Ready, Uninitialized};
use crate::settings::{SettingsEvent, SettingsService};

pub use self::Service as NearbyService;

pub mod model;
mod peer_calc;

pub const PUBLIC_STATUS: &str = "status";
pub const MESSAGES: &str = "messages";
pub const MESSAGE_PAYLOADS: &str = "message_payloads";
#[derive(Clone)]
pub struct Service(Arc<InnerService>);

impl Deref for Service {
    type Target = InnerService;
    fn deref(&self) -> &Self::Target { self.0.as_ref() }
}

#[derive(Clone, Debug)]
pub enum NearbyServiceEvents {
    IdentitiesUpdated(Vec<NearbyProfile>),
    ScanningUpdated(bool),
    DebugStateUpdated(DebugState),
    AllMessagesUpdated(Vec<DisplayMessage>),
    ReceivedOneNewMessage(DisplayMessage)
}

pub enum State {
    Uninitialized {
        node: Node
    },
    Ready {
        doc: Doc,
        doc_share: DocTicket,
        identities: Vec<Identity>,
        pics: HashMap<PublicKey, BlobHash>,
        statuses: HashMap<PublicKey, Status>,
        found_group: bool,
        should_scan: bool,
        should_broadcast: bool,
        ble_peers: HashMap<UUID, PeerData>,
        messages: Vec<Post>
    },
}

pub struct InnerService {
    bc: Sender<NearbyServiceEvents>,
    ble_broadcaster: Arc<dyn BLEGossipBroadcaster>,
    ble_scanner: Arc<dyn BLEGossipScanner>,
    identity_service: IdentityService,
    settings_service: SettingsService,
    state: RwLock<State>,
}

impl Service {
    pub fn new(node: Node, identity_service: IdentityService, settings_service: SettingsService, device: Arc<dyn DeviceApiServiceProvider>) -> Service {
        let s = Service(Arc::new(InnerService {
            bc: create_broadcast(),
            ble_broadcaster: device.ble_broadcaster(),
            ble_scanner: device.ble_scanner(),
            identity_service,
            settings_service,
            state: RwLock::new(Uninitialized { node }),
        }));
        let o = s.clone();
        tokio::spawn(async move { o.initialize().await });
        s
    }
}

impl Service {
    pub fn subscribe(&self) -> Receiver<NearbyServiceEvents> {
        self.bc.subscribe()
    }

    async fn initialize(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if let Uninitialized { ref node } = *state {
            let doc = Doc(node.docs().create().await.unwrap(), node.clone());
            let ticket = doc.share(Write, Id).await?;
            *state = Ready {
                doc,
                doc_share: ticket,
                identities: vec![],
                pics: HashMap::new(),
                statuses: HashMap::new(),
                ble_peers: HashMap::new(),
                found_group: false,
                should_scan: false,
                should_broadcast: false,
                messages: vec![],
            };
            drop(state);

            self.update_ble_broadcast(true).await?;
            self.setup_scanning().await?;
            self.update_scanning(false).await?;


            self.load_doc().await?;
            // this will use this async task and loop in it
            self.listen_to_other_services().await?;
        } else {
            panic!("cant start already initialized service")
        }
        Ok(())
    }

    pub async fn push_debug_state(&self) {
        let lock = self.state.read().await;
        if let Ready { ref doc, ref found_group,..} = *lock {
            let ds = DebugState {
                doc_id: doc.0.id().to_string(),
                found_group: *found_group
            };

            drop(lock);
            broadcast(&self.bc, DebugStateUpdated(ds)).expect("push debug state");
        }
    }

    pub async fn listen_to_other_services(&self) -> Result<()> {
        let mut identity_sub = self.identity_service.subscribe();
        let mut settings_sub = self.settings_service.subscribe();

        let s1 = self.clone();
        tokio::spawn(async move {
            while let Ok(e) = identity_sub.recv().await {
                let lock = s1.state.read().await;
                if let Ready { ref doc, .. } = *lock {
                    match e {
                        IdentityServiceEvents::DefaultIdentityUpdated(iden)=> {
                            s1.update_my_identity_on_doc(&iden, doc).await.expect("putting identity on doc");
                        }
                        IdentityServiceEvents::DefaultIdentityPicUpdated(hash, size) => {
                            s1.update_my_pic_on_doc(hash, size, doc).await.expect("putting pic on doc");
                        }
                    }
                }
            };
        });

        let s2 = self.clone();
        tokio::spawn(async move {
            while let Ok(e) = settings_sub.recv().await {
                match e {
                    SettingsEvent::StatusSettingChanged(s) => {
                        let lock = s2.state.read().await;
                        if let Ready { ref doc, .. } = *lock {
                            s2.update_my_status_on_doc(&s, doc).await.expect("putting status on doc");
                        }
                    }
                }
            };
        });

        Ok(())
    }

    async fn status_setting_changed(&self, new_status: Status) -> Result<()> {
        let lock = self.state.read().await;
        if let Ready { ref doc, .. } = *lock {
            doc.write_blob(PUBLIC_STATUS, new_status).await?;
        }
        Ok(())
    }

    pub async fn should_scan(&self) -> bool {
        let mut lock = self.state.read().await;
        if let Ready { ref should_scan, ..} = *lock {
            return *should_scan
        }
        return false
    }

    pub async fn update_scanning(&self, new_should_scan: bool) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut should_scan, .. } = *lock {
            if *should_scan != new_should_scan {
                *should_scan = new_should_scan;
                if *should_scan {
                    self.ble_scanner.start_scanning();
                } else {
                    self.ble_scanner.stop_scanning();
                }

                broadcast(&self.bc, ScanningUpdated(*should_scan))?;
            }
        }

        Ok(())
    }
    async fn setup_scanning(&self) -> Result<()> {
        let (tx, mut rx) = channel(16);
        let delegate = Arc::new(GossipScannerDelegate(tx));
        let self2 = self.clone();
        tokio::spawn(async move {

            while let Some(d) = rx.recv().await {
                self2.receive_peer_update(d.0, d.1).await.expect("boom?");
            }
            println!("mini peer receive loop died");
        });
        self.ble_scanner.set_delegate(delegate);
        Ok(())
    }

    async fn receive_peer_update(&self, uuid: UUID, data: PeerData) -> Result<()> {
        let found_group = {
            let mut lock = self.state.write().await;
            if let Ready { ref mut ble_peers, found_group, .. } = *lock {
                ble_peers.insert(uuid, data);
                println!("bluetooth peer found {:?}", uuid);
                found_group
            } else {
                true
            }
        };

        if found_group { return Ok(()); }
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.evaluate_peers_for_connection().await?;

        Ok(())
    }

    async fn evaluate_peers_for_connection(&self) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut doc, ref mut doc_share, ref ble_peers, .. } = *lock {
            let my_doc = get_document_data(doc_share);
            let best_new_doc: DocumentData = find_best_doc_from_peers(&my_doc, &ble_peers);
            if best_new_doc == my_doc {
                println!("best doc is me, i sit here");
                return Ok(());
            }
            println!("lets join a new document!");
            let addrs: Vec<AddressData> = collect_addrs_for_doc(&best_new_doc, ble_peers.values());

            let cap: Capability = postcard::from_bytes(&best_new_doc).map_err(ticket::Error::Postcard).expect("boom1");

            let addrs: Vec<Vec<NodeAddr>> = addrs.iter().map(|a| {
                let inner_addrs: Vec<NodeAddr> = postcard::from_bytes(a.as_slice()).map_err(ticket::Error::Postcard).expect("boom2");
                inner_addrs
            }).collect();
            let addrs: Vec<NodeAddr> = addrs.into_iter().flatten().collect();

            let ticket = DocTicket { capability: cap, nodes: addrs };
            let new_doc = doc.1.docs().import(ticket.clone()).await?;
            println!("doc imported! {}", new_doc.id());
            *doc = Doc(new_doc, doc.1.clone());
            *doc_share = ticket;
        }
        drop(lock);

        self.load_doc().await?;

        Ok(())
    }

    async fn update_ble_broadcast(&self, new_should_broadcast: bool) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref doc_share, ref mut should_broadcast, ref mut found_group, ref mut doc, .. } = *lock {
            *should_broadcast = new_should_broadcast;

            let document = get_document_data(doc_share);
            let addrs: AddressData = get_address_data(doc_share);

            self.ble_broadcaster.set_peer_state(if *found_group { 1 } else { 0 });
            self.ble_broadcaster.set_document_data(document);
            self.ble_broadcaster.set_address_data(addrs);

            if new_should_broadcast {
                self.ble_broadcaster.start();
            } else {
                self.ble_broadcaster.stop();
            }
        }

        Ok(())
    }


    pub async fn update_my_identity_on_doc(&self, iden: &Identity, doc: &Doc) -> Result<()> {
        doc.write_blob(IDENTITY, iden).await?;
        Ok(())
    }
    pub async fn update_my_status_on_doc(&self, status: &Status, doc: &Doc) -> Result<()> {
        doc.write_blob(PUBLIC_STATUS, status).await?;
        Ok(())
    }

    pub async fn update_my_pic_on_doc(&self, hash: BlobHash, size: u64, doc: &Doc) -> Result<()> {
        let pk = self.identity_service.get_default_identity_pk().await?;
        doc.set_hash(pk.into(), ID_PIC, hash.into(), size).await?;
        Ok(())
    }
    pub async fn put_self_on_doc(&self, doc: &Doc) -> Result<()> {
        println!("puttin myself on doc {}", doc.0.id());
        if let Some(iden) = self.identity_service.get_default_identity().await? {
            self.update_my_identity_on_doc(&iden, doc).await?;
            let status = self.settings_service.get_status().await?;
            self.update_my_status_on_doc(&status, doc).await?;

            if let Some((pic_hash, size)) = self.identity_service.get_pic(iden.pk).await? {
                self.update_my_pic_on_doc(pic_hash, size, doc).await?;
            }
        } else {
            println!("Couldn't put self on doc cuz no identity, but will do it when its there!");
        }
        Ok(())
    }

    pub async fn load_doc(&self) -> Result<()> {
        // load existing data
        {
            let mut lock = self.state.write().await;
            if let Ready {
                ref mut doc,
                ref mut found_group,
                ref mut identities,
                ref mut pics,
                ref doc_share,
                ref mut messages,
                ref mut statuses, .. } = *lock
            {

                // whenever we load a new doc, lets make sure we broadcast it
                self.ble_broadcaster.set_document_data(get_document_data(doc_share));
                self.ble_broadcaster.set_peer_state(0);
                // put myself on the doc

                self.put_self_on_doc(&doc).await?;
                // we may set this to true in just a bit if this doc is existing
                *found_group = false;


                let loaded_posts: Vec<Post> = doc.read_blobs_by_query(Query::key_prefix(MESSAGES).sort_by(SortBy::KeyAuthor, SortDirection::Asc)).await?;
                *messages = loaded_posts;

                let loaded_identities: Vec<Identity> = doc.read_blobs_by_query(Query::key_exact(IDENTITY)).await?;
                let status_entries: Vec<Entry> = doc.list_entries_by_query(Query::key_exact(PUBLIC_STATUS)).await?;
                let loaded_pics = doc.list_entries_by_query(Query::key_exact(ID_PIC)).await?;
                *identities = loaded_identities;

                *pics = HashMap::new();
                loaded_pics.into_iter().for_each(|entry| {
                    pics.insert(entry.author().into(), entry.content_hash().into());
                });

                *statuses = HashMap::new();
                for se in status_entries {
                    let status: Status = doc.read_blob_by_hash(se.content_hash()).await?;
                    statuses.insert(se.author().into(), status);
                }
            }
        }
        // we may have loaded a doc that we already have identities and a group on
        // this may update the peerstate on ble
        self.check_if_found_group().await?;
        self.push_debug_state().await;
        self.identities_did_update().await?;
        self.broadcast_all_messages().await?;

        let s = {
            let lock = self.state.read().await;
            if let Ready { ref doc, .. } = *lock {
                Some(doc.clone())
            } else { None }
        };

        if let Some(doc) = s {
            let s1 = self.clone();
            tokio::spawn(async move {
                println!("loaded doc, looping {}", doc.0.id());
                let mut stream = doc.subscribe().await?;
                while let Some(e) = stream.next().await {
                    match key_of(&e.entry).as_ref() {
                        IDENTITY => {
                            let i: Identity = doc.read_blob_by_hash(e.entry.content_hash()).await?;
                            s1.identity_updated(i).await?;
                        }
                        ID_PIC => {
                            s1.id_pic_update(e.entry).await?;
                        }
                        PUBLIC_STATUS => {
                            let s: Status = doc.read_blob_by_hash(e.entry.content_hash()).await?;
                            s1.status_update(e.entry.author().into(), s).await?;
                        }
                        key if key.starts_with(MESSAGE_PAYLOADS) => {
                            println!("remote insert for payload {key}");
                        }
                        key if key.starts_with(MESSAGES) => {
                            let m: Post = doc.read_blob_by_hash(e.entry.content_hash()).await?;
                            s1.message_updated(m).await?;
                        }

                        _ => {}
                    }
                };
                println!("Stopped listening to doc!");
                Ok::<(), anyhow::Error>(())
            });
        }

        println!("finished doc loop!!");
        Ok(())
    }

    pub async fn post_message(&self, text: String, payload_dir: Option<String>) -> Result<()> {
        let lock = self.state.read().await;
        let me = self.identity_service.get_default_identity_pk().await?;

        if let Ready { ref doc,..  } = *lock {
            let doc = doc.clone();
            drop(lock);

            let payload_blob = if let Some(payload_dir)  = payload_dir {
                // let entries = tokio::fs::read_dir(payload_dir).await?;
                // Iterate over the entries
                let mut hashes: Vec<(String, Hash)> = vec![];
                let mut read_stream = tokio::fs::read_dir(&payload_dir).await?;
                while let Some(entry) = read_stream.next_entry().await? {
                    let filename = entry.file_name();
                    let file_name_str = filename.to_string_lossy(); // Get the file name

                    println!("found file {file_name_str}");
                    let blob = doc.1.blobs().add_from_path(entry.path(),true, SetTagOption::Auto,WrapOption::NoWrap).await?;
                    let x = blob.await?;
                    hashes.push((file_name_str.to_string(), x.hash))
                }
                let collection: Collection = hashes.into_iter().collect();
                let (payload_blob,x) = doc.1.blobs().create_collection(collection,SetTagOption::Auto, vec![]).await?;
                Some(payload_blob)
            } else { None };

            let p = Post::new(me)
                .body(text)
                .payload(payload_blob.map(|f| f.into()));
            println!("making a post {:?}", p);

            let key = message_key(&p);
            doc.write_blob(&key,&p).await?;

            if let Some(b) = payload_blob {
                let size = (if let BlobStatus::Complete { size } = doc.1.blobs().status(b).await? {
                    Some(size)
                } else { None }).expect("blob just added not compelte???");
                let key = message_payload_key(&p);
                doc.set_hash(me.into(), key, b.into(), size).await?;
            }
        }

        Ok(())
    }

    pub async fn broadcast_all_messages(&self) -> Result<()> {
        let me = self.identity_service.get_default_identity_pk().await?;
        let mut lock = self.state.read().await;
        if let Ready { ref messages, .. } = *lock {
            let mut msgs: Vec<DisplayMessage> = messages.clone().into_iter().enumerate()
                .map(|(i,m)| { display_msg_map(i, &me, m) })
                .collect();
            broadcast(&self.bc, AllMessagesUpdated(msgs))?;
        };
        Ok(())
    }

    pub async fn clone_doc(&self) -> Result<Doc> {
        let mut lock = self.state.read().await;
        if let Ready { ref doc,.. } = *lock {
            let doc = doc.clone();
            drop(lock);
            Ok(doc)
        } else {
            Err(anyhow!("bad state, no doc!"))
        }
    }

    pub async fn get_or_download_collection(&self, hash: BlobHash, delegate: Arc<dyn LoadCollectionDelegate>) -> Result<()> {
        let doc = self.clone_doc().await?;
        if let Ok(items) = doc.get_or_download_collection(hash).await {
            delegate.update(CollectionState::Loaded(items)).await;
        } else {
            delegate.update(CollectionState::Failed(String::from("who knows"))).await;
        }
        Ok(())
    }
    //
    // pub async fn hacky_fix_up(&self, msgs: &mut Vec<DisplayMessage>) -> Result<()> {
    //     let mut lock = self.state.read().await;
    //     if let Ready { ref doc,.. } = *lock {
    //         let doc = doc.clone();
    //         drop(lock);
    //         for m in msgs {
    //             if let Some(ref mut p) = m.payload {
    //                 let hash = Hash::from_str(p)?;
    //                 let status = doc.1.blobs().status(hash).await?;
    //                 println!("omg status {:?}, downloadeding", status);
    //                 let mut x = doc.get_peer_nodes().await;
    //                 let z = doc.1.blobs().download_hash_seq(hash, x.remove(0)).await?;
    //                 let z = z.finish().await?;
    //                 println!("i downloaded that fucker {:?}",z);
    //
    //
    //                 let mut stream = doc.1.blobs().list_collections()?;
    //
    //                 while let Some(Ok(x)) = stream.next().await {
    //                     println!("omg its {:?}",x);
    //                 }
    //
    //                 println!("BUT I HAVE BLOB {}",hash);
    //                 match status {
    //                     BlobStatus::Partial { size } => {
    //                         println!("Collection blob NOT compeltedly had! {}", size);
    //                     }
    //                     BlobStatus::Complete { size } => {
    //                         println!("Collection blob compelted had! {}", size);
    //                         let b = doc.1.blobs().get_collection(hash).await?;
    //                         println!("and its dets {:?}", b);
    //                         let mut ugh = String::default();
    //                         for (name,hash) in b.into_iter() {
    //                             ugh = format!("{ugh}\n{name} -> {hash}");
    //                         }
    //                         *p = ugh;
    //                     }
    //                 }
    //
    //
    //             }
    //         }
    //     }
    //     Ok(())
    // }

    pub async fn message_updated(&self, message: Post) -> Result<()> {
        let mut lock = self.state.write().await;
        let mut requires_reload = false;
        let me = self.identity_service.get_default_identity_pk().await?;

        let mut msgs_to_send: Vec<DisplayMessage> = vec![];
        if let Ready { ref mut messages, ..} = *lock {
            if messages.len() == 0 {
                messages.push(message.clone());
                msgs_to_send.push(display_msg_map(0, &me, message));

            } else if let Some(last) = messages.last() {
                if message.created_at.lt(&last.created_at) {
                    // we got something out of order...
                    println!("requried reload! out of order");
                    requires_reload = true;
                }
                messages.push(message.clone());

                if requires_reload {
                    messages.sort_by(|a,b| {
                        a.created_at.partial_cmp(&b.created_at).unwrap()
                    });
                    msgs_to_send = messages.clone().into_iter().enumerate()
                        .map(|(i,m)| { display_msg_map(i, &me, m) })
                        .collect();

                } else {
                    msgs_to_send.push(display_msg_map(messages.len()-1, &me, message));
                }
            }
        };

        drop(lock);

        if requires_reload {
            broadcast(&self.bc, AllMessagesUpdated(msgs_to_send))?;
        } else {
            broadcast(&self.bc, ReceivedOneNewMessage(msgs_to_send.remove(0)))?;
        }
        Ok(())
    }


    pub async fn identity_updated(&self, updated_iden: Identity) -> Result<()> {
        let mut lock = self.state.write().await;
        let mut added_new_iden = false;
        if let Ready { ref mut identities, .. } = *lock {
            let mut updated_iden = Some(updated_iden);

            for iden in identities.iter_mut() {
                if iden.pk == updated_iden.as_ref().unwrap().pk {
                    *iden = updated_iden.take().unwrap();
                    break; // found the iden to update, break out of this loop cause unwrap will fail
                }
            }
            if let Some(updated) = updated_iden {
                identities.push(updated);
                added_new_iden = true
            }
        };
        drop(lock);
        if added_new_iden {
            // no point if we're just changing our name
            self.check_if_found_group().await?;
        }
        self.identities_did_update().await?;

        Ok(())
    }

    pub async fn check_if_found_group(&self) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut found_group, ref identities,.. } = *lock {
            if identities.len() > 1 {
                // we DID find a group
                *found_group = true;
            } else {
                *found_group = false;
            }
            self.ble_broadcaster.set_peer_state(if *found_group { 1 } else { 0 })
        };
        drop(lock);
        self.push_debug_state().await;
        Ok(())
    }

    pub async fn did_find_group(&self) -> Result<()> {
        {
            let mut lock = self.state.write().await;
            if let Ready { ref mut found_group, .. } = *lock {
                *found_group = true;
            }
        }
        self.push_debug_state().await;
        Ok(())
    }

    pub async fn id_pic_update(&self, entry: Entry) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut pics, .. } = *lock {
            pics.insert(entry.author().into(), entry.content_hash().into());
        }
        drop(lock);
        self.identities_did_update().await?;
        Ok(())
    }

    pub async fn status_update(&self, pk: PublicKey, status: Status) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut statuses, .. } = *lock {
            statuses.insert(pk, status);
        }
        drop(lock);
        self.identities_did_update().await?;

        Ok(())
    }

    pub async fn identities_did_update(&self) -> Result<()> {
        let profiles: Vec<NearbyProfile> = {
            let lock = self.state.read().await;
            if let Ready { ref doc, ref identities, ref pics, ref statuses, .. } = *lock {
                identities.clone().into_iter().map(|i| {
                    NearbyProfile {
                        pk: i.pk,
                        name: i.name,
                        pic: pics.get(&i.pk).copied(),
                        status: statuses.get(&i.pk).cloned().or_else(|| Some(Status { text: String::new() })).unwrap(),
                    }
                }).collect()
            } else {
                return Err(anyhow!("wtf"));
            }
        };
//        println!("identities did update fired with {:?}", &profiles);
        broadcast(&self.bc, IdentitiesUpdated(profiles))?;
        Ok(())
    }
}

fn get_document_data(ticket: &DocTicket) -> DocumentData {
    postcard::to_stdvec(&ticket.capability).expect("serializing document data")
}

fn get_address_data(ticket: &DocTicket) -> AddressData {
    postcard::to_stdvec(&ticket.nodes).expect("serializing address data")
}

fn display_msg_map(idx: usize, me: &PublicKey, msg: Post) -> DisplayMessage {
    DisplayMessage {
        id: idx as u32,
        text: msg.body.unwrap_or_else(||String::default()),
        is_self: me == &msg.pk,
        payload: msg.payload
    }
}
