use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures_lite::StreamExt;
use iroh::base::node_addr::AddrInfoOptions::Id;
use iroh::base::ticket;
use iroh::blobs::Hash;
use iroh::client::docs::Entry;
use iroh::client::docs::ShareMode::Write;
use iroh::docs::{Capability, DocTicket};
use iroh::docs::store::Query;
use iroh::net::NodeAddr;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::channel;
use tokio::sync::{Notify, RwLock};

use crate::ble::{AddressData, BLEGossipBroadcaster, BLEGossipScanner, BluetoothPeerEvent, DocumentData, GossipScannerDelegate, PeerData};
use crate::blob_dispatcher::{CollectionState, LoadCollectionDelegate, NamedBlob};
use crate::data::{BlobHash, collection_from_dir, PublicKey, replace_or_add_blob, UUID};
use crate::device::DeviceApiServiceProvider;
use crate::doc::{CoreDoc, Doc, InsertEntry, key_of, Node};
use crate::events::{broadcast, create_broadcast, Subscriber, WeakService};
use crate::identity::domain::{IdentityDomain, IdentityDomainResponder};
use crate::identity::IdentityService;
use crate::identity::model::{ID_PIC, IDENTITY, Identity, IdentityServiceEvents};
use crate::nearby::model::{BioDetails, DebugState, display_msg_map, DisplayMessage, NearbyProfile, Post, Status};
use crate::nearby::NearbyServiceEvents::{AllMessagesUpdated, BioUpdated, DebugStateUpdated, IdentitiesUpdated, ReceivedOneNewMessage, ScanningUpdated};
use crate::nearby::peer_calc::{collect_addrs_for_doc, find_best_doc_from_peers};
use crate::nearby::post::{PostDomain, PostDomainResponder};
use crate::nearby::State::{Ready, Uninitialized};
use crate::settings::{SettingsEvent, SettingsService};

pub use self::Service as NearbyService;

pub mod model;
mod peer_calc;
mod post;

pub const PUBLIC_STATUS: &str = "status";
pub const MESSAGES: &str = "messages";

pub const BIO: &str = "public_bio";
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
    ReceivedOneNewMessage(DisplayMessage),
    BioUpdated(PublicKey),
}

pub enum State {
    Uninitialized {
        node: Node
    },
    Ready {
        doc: Doc,
        doc_stop: Arc<Notify>,
        doc_share: DocTicket,
        identities: IdentityDomain<Service, InnerService>,
        statuses: HashMap<PublicKey, Status>,
        found_group: bool,
        should_scan: bool,
        should_broadcast: bool,
        ble_peers: HashMap<UUID, PeerData>,
        messages: PostDomain<Service, InnerService>
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

#[async_trait]
impl PostDomainResponder for Service {
    async fn all_posts_updated(&self, posts: Vec<Post>) -> Result<()> {
        let me = self.identity_service.get_default_identity_pk().await?;
        let dms: Vec<DisplayMessage> = posts.into_iter().enumerate()
            .map(|(i,m)| { display_msg_map(i, &me, m) })
            .collect();
        broadcast(&self.bc, AllMessagesUpdated(dms))?;
        Ok(())
    }

    async fn one_post_updated(&self, count: usize, post: Post) -> Result<()> {
        let me = self.identity_service.get_default_identity_pk().await?;
        let dm: DisplayMessage = display_msg_map(count-1, &me, post);
        broadcast(&self.bc, ReceivedOneNewMessage(dm))?;
        Ok(())
    }
}

#[async_trait]
impl IdentityDomainResponder for Service {
    async fn identities_did_update(&self, added_new: bool) -> Result<()> {
        let profiles: Vec<NearbyProfile> = {
            let lock = self.state.read().await;
            if let Ready { ref identities, ref statuses, .. } = *lock {
                let pics = identities.pics();
                let idens = identities.identities().clone();
                idens.into_iter().map(|i| {
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
        if added_new {
            // no point if we're just changing our name
            self.check_if_found_group().await?;
        }

        broadcast(&self.bc, IdentitiesUpdated(profiles))?;
        Ok(())
    }

    async fn pics_did_update(&self) -> Result<()> {
        self.identities_did_update(false).await?;
        Ok(())
    }
}

#[async_trait]
impl Subscriber<SettingsEvent, InnerService, Service> for Service {
    async fn event(&self, event: SettingsEvent) -> Result<()> {
        let lock = self.state.read().await;
        if let Ready { ref doc, .. } = *lock {
            match event {
                SettingsEvent::StatusSettingChanged(s) => {
                    self.update_my_status_on_doc(&s, doc).await?;
                }
                SettingsEvent::OwnPublicBioUpdated(e) => {
                    self.update_my_bio_on_doc(e, doc).await?;
                }
            }
        };

        Ok(())
    }
}

#[async_trait]
impl Subscriber<IdentityServiceEvents, InnerService, Service> for Service {
    async fn event(&self, event: IdentityServiceEvents) -> Result<()> {
        let lock = self.state.read().await;
        if let Ready { ref doc, .. } = *lock {
            match event {
                IdentityServiceEvents::DefaultIdentityUpdated(iden)=> {
                    self.update_my_identity_on_doc(&iden, doc).await?;
                }
                IdentityServiceEvents::DefaultIdentityPicUpdated(hash, size) => {
                    self.update_my_pic_on_doc(hash, size, doc).await?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Subscriber<BluetoothPeerEvent, InnerService, Service> for Service {
    async fn event(&self, (uuid, data): BluetoothPeerEvent) -> Result<()> {
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
}

impl Service {
    pub fn subscribe(&self) -> Receiver<NearbyServiceEvents> {
        self.bc.subscribe()
    }

    async fn setup_scanning(&self) -> Result<()> {
        let (tx, rx) = channel(16);
        let delegate = Arc::new(GossipScannerDelegate(tx));
        self.listen_mpsc(rx);
        self.ble_scanner.set_delegate(delegate);
        Ok(())
    }


    async fn get_initial_doc(&self, node: &Node) -> Result<CoreDoc> {
        let previous_nearby = self.settings_service.get_nearby_doc().await?;
        let mut doc: Option<CoreDoc> = None;
        if let Some(previous_nearby) = previous_nearby {
            doc = node.docs().open(previous_nearby).await?;
        }
        if matches!(doc, None) {
            doc = Some(node.docs().create().await?);
        }
        Ok(doc.unwrap())
    }

    async fn initialize(&self) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Uninitialized { ref node } = *lock {
            let doc = Doc(self.get_initial_doc(node).await?, node.clone());
            *lock = self.ready_state_with_doc(doc).await;
            drop(lock);

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

    pub async fn get_profile_by_key(&self, pk: &PublicKey) -> Result<NearbyProfile> {
        let lock = self.state.read().await;
        if let Ready { ref identities, ref statuses, ..} = *lock {
            let pics = identities.pics();
            let iden = identities.identities_ref().into_iter().find(|i|&i.pk == pk);
            if let Some(iden) = iden {
                return Ok(NearbyProfile {
                    pk: iden.pk,
                    name: iden.name.clone(),
                    pic: pics.get(pk).copied(),
                    status: statuses.get(pk).cloned().unwrap_or(Status { text: String::default() }),
                })
            }
        }
        Err(anyhow!("profile does not exist here"))
    }

    async fn ready_state_with_doc(&self, doc: Doc) -> State {
        let ticket = doc.share(Write, Id).await.expect("generating doc ticket");
        let messages = PostDomain::new(&doc, self);
        let identities = IdentityDomain::new(&doc, self);

        Ready {
            doc,
            doc_stop: Arc::new(Notify::new()),
            doc_share: ticket,
            identities,
            statuses: HashMap::new(),
            ble_peers: HashMap::new(),
            found_group: false,
            should_scan: false,
            should_broadcast: false,
            messages
        }
    }

    pub async fn start_sync(&self) -> Result<()> {
        let lock = self.state.read().await;
        if let Ready { ref doc, .. } = *lock {
            doc.start_sync_with_known_peers().await?;
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
        self.listen_bc(self.identity_service.subscribe());
        self.listen_bc(self.settings_service.subscribe());
        Ok(())
    }

    async fn status_setting_changed(&self, new_status: Status) -> Result<()> {
        let lock = self.state.read().await;
        if let Ready { ref doc, .. } = *lock {
            doc.write_keyed_blob(PUBLIC_STATUS, new_status).await?;
        }
        Ok(())
    }


    pub async fn should_scan(&self) -> bool {
        let lock = self.state.read().await;
        if let Ready { ref should_scan, ..} = *lock {
            return *should_scan
        }
        return false
    }

    pub async fn leave_group(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if let Ready { ref doc, ref doc_stop, .. } = *state {
            let node = doc.1.clone();
            // doc.leave().await?;
            doc_stop.notify_waiters();

            let old_doc_to_delete = doc.id();
            // doc.close().await?;

            let doc = Doc(node.docs().create().await?, node.clone());
            *state = self.ready_state_with_doc(doc).await;

            println!("note to self, delete {}", old_doc_to_delete);
            // node.docs().drop_doc(old_doc_to_delete).await?;
        };
        drop(state);
        self.load_doc().await?;

        Ok(())
    }
    pub async fn update_scanning(&self, new_should_scan: bool) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut should_scan, .. } = *lock {
            if *should_scan != new_should_scan {
                *should_scan = new_should_scan;
                if *should_scan {
                    println!("started ble scanning");
                    self.ble_scanner.start_scanning();
                } else {
                    println!("stopped ble scanning");
                    self.ble_scanner.stop_scanning();
                }
            }
            broadcast(&self.bc, ScanningUpdated(*should_scan))?;
        }

        Ok(())
    }

    async fn evaluate_peers_for_connection(&self) -> Result<()> {
        let lock = self.state.read().await;
        let new_doc = if let Ready { ref doc,
            ref doc_share,
            ref doc_stop,
            ref ble_peers, .. } = *lock
        {
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

            let doc_ticket = DocTicket { capability: cap, nodes: addrs };
            let new_doc = doc.1.docs().import(doc_ticket.clone()).await?;
            doc_stop.notify_waiters(); // we're about to LEAVE THE OLD DOC BEHIND VERY IMPORTANT,
            Doc(new_doc, doc.1.clone())
        } else { return Ok(()) };
        drop(lock);

        {
            let mut lock = self.state.write().await;
            // this should drop the old doc and all that shit
            *lock = self.ready_state_with_doc(new_doc).await;
        }
        self.load_doc().await?;

        Ok(())
    }

    async fn update_ble_broadcast(&self, new_should_broadcast: bool) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref doc_share, ref mut should_broadcast, ref mut found_group, .. } = *lock {
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
        doc.write_keyed_blob(IDENTITY, iden).await?;
        Ok(())
    }
    pub async fn update_my_status_on_doc(&self, status: &Status, doc: &Doc) -> Result<()> {
        doc.write_keyed_blob(PUBLIC_STATUS, status).await?;
        Ok(())
    }

    pub async fn update_my_pic_on_doc(&self, hash: BlobHash, size: u64, doc: &Doc) -> Result<()> {
        let pk = self.identity_service.get_default_identity_pk().await?;
        doc.set_hash(pk.into(), ID_PIC, hash.into(), size).await?;
        Ok(())
    }

    pub async fn update_my_bio_on_doc(&self, entry: Entry, doc: &Doc) -> Result<()> {
        println!("propagating my bio doc from settings to this nearby doc, i am {}, bio hash is {}", entry.author(), entry.content_hash());
        doc.set_hash(doc.me().await, BIO, entry.content_hash(), entry.content_len()).await?;
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

            if let Some(entry) = self.settings_service.get_bio_entry().await? {
                self.update_my_bio_on_doc(entry, doc).await?;
            }
        } else {
            println!("Couldn't put self on doc cuz no identity, but will do it when its there!");
        }
        Ok(())
    }

    pub async fn get_bio(&self, owner: &PublicKey) -> Result<Option<BioDetails>> {
        let doc = self.clone_doc().await?;
        let me: PublicKey = doc.me().await.into();

        let entry = doc.0.get_exact(owner.into(), BIO, false).await?;
        let collection = match entry {
            None => {
                if &me == owner {
                    // allow me to return a blank bio to myself
                    vec![]
                } else {
                    return Ok(None)
                }
            }
            Some(e) => {
                doc.get_or_download_collection(e.content_hash().into()).await?
            }
        };

        let mut bio_text_hash: Option<BlobHash> = None;
        let pics: Vec<NamedBlob> = collection.into_iter().filter(|b| {
            if &b.name == "bio_text.txt" {

                bio_text_hash = Some(b.hash);
                return false;
            }
            return true;
        }).collect();
        let bio_text: String = match bio_text_hash {
            None => { String::default() }
            Some(hash) => { doc.read_blob_by_hash(hash.into()).await? }
        };

        let editable = *owner == me;

        Ok(Some(BioDetails {
            text: bio_text,
            shared: true, //todo fix this
            editable,
            pics,
        }))

    }

    pub async fn set_bio_text(&self, bio_text: String) -> Result<()> {
        let mut bio_blobs = self.settings_service.get_bio().await?;
        let doc = self.clone_doc().await?;
        let new_text_blob = doc.write_blob(bio_text).await?;
        replace_or_add_blob("bio_text.txt", new_text_blob.hash.into(), &mut bio_blobs).await;
        self.settings_service.set_bio(bio_blobs).await?;
        Ok(())
    }

    pub async fn set_share_bio(&self, share_bio: bool) -> Result<()> {
        // TODO figure this out
        Ok(())
    }

    pub async fn set_gallery_pic(&self, index: usize, pic: Vec<u8>) -> Result<()>{
        if index > 9 {
            return Err(anyhow!("only supports up to 9 gallery pics"))
        };
        let blob_key = format!("{index}.png");
        let doc = self.clone_doc().await?;
        let new_pic_blob = doc.1.blobs().add_bytes(pic).await?;

        let mut bio_blobs = self.settings_service.get_bio().await?;
        replace_or_add_blob(&blob_key, new_pic_blob.hash.into(), &mut bio_blobs).await;
        self.settings_service.set_bio(bio_blobs).await?;

        Ok(())
    }

    pub async fn clear_gallery_pic(&self, index: usize) -> Result<()> {
        let mut bio_blobs = self.settings_service.get_bio().await?;
        let blob_key = format!("{index}.png");
        bio_blobs = bio_blobs.into_iter().filter(|b| b.name == blob_key).collect();
        self.settings_service.set_bio(bio_blobs).await?;
        Ok(())
    }

    pub async fn load_doc(&self) -> Result<()> {
        println!("load_doc");
        // load existing data
        let outs = {
            let mut lock = self.state.write().await;
             if let Ready {
                ref mut doc,
                ref doc_stop,
                ref mut found_group,
                ref mut identities,
                ref doc_share,
                ref mut messages,
                ref mut statuses, .. } = *lock
            {
                // should i combine this with initialize?
                identities.set_doc(doc);
                messages.set_doc(doc);

                // whenever we load a new doc, lets make sure we broadcast it
                self.ble_broadcaster.set_document_data(get_document_data(doc_share));
                self.ble_broadcaster.set_peer_state(0);

                self.put_self_on_doc(&doc).await?;
                // we may set this to true in just a bit if this doc is existing
                *found_group = false;

                messages.initialize().await?;
                identities.initialize().await?;

                let status_entries: Vec<Entry> = doc.list_entries_by_query(Query::key_exact(PUBLIC_STATUS)).await?;
                *statuses = HashMap::new();
                for se in status_entries {
                    let status: Status = doc.read_blob_by_hash(se.content_hash()).await?;
                    statuses.insert(se.author().into(), status);
                }
                println!("got here");
                self.settings_service.set_nearby_doc(doc.id()).await?;
                Some((doc.id(), doc.subscribe().await?, doc_stop.clone()))
            } else { None }
        };
        // we may have loaded a doc that we already have identities and a group on
        // this may update the peerstate on ble

        // we may have opened up a doc that already exists and has people on it
        self.check_if_found_group().await?;
        self.push_debug_state().await;
        self.identities_did_update(false).await?;
        self.broadcast_all_messages().await?;
        self.update_scanning(false).await?;
        println!("about here now");

        let (id, mut stream, doc_stop_clone) = outs.unwrap();

        let self_clone = self.clone();

        println!("🟢 Starting listening to {id}");
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(e) = stream.next() => {
                        if let Err(e) =  self_clone.handle_insert_entry(e).await {
                            eprintln!("error handling insert entry {}", e.to_string())
                        }
                    }
                    _ = doc_stop_clone.notified() => {
                        println!("we got notified!");
                        break;
                    }
                }
            }
            println!("🛑 Stopping listening to {id}");
            Ok::<(), anyhow::Error>(())
        });
        Ok(())
    }

    async fn handle_insert_entry(&self, e: InsertEntry) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready {
            ref mut messages, ref mut identities, ref doc,  ..} = *lock
        {
            match key_of(&e.entry).as_ref() {
                PUBLIC_STATUS => {
                    let s: Status = doc.read_blob_by_hash(e.entry.content_hash()).await?;
                    drop(lock); //IMPORTANT TO DO
                    self.status_update(e.entry.author().into(), s).await?;
                }
                BIO => {
                    drop(lock);
                    broadcast(&self.bc, BioUpdated(e.entry.author().into()))?;
                }
                key if identities.handles(key) => {
                    identities.insert_entry(e).await?;
                }
                key if messages.handles(key) => {
                    messages.insert_entry(e).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn post_message(&self, text: String, payload_dir: Option<String>) -> Result<()> {
        let me = self.identity_service.get_default_identity_pk().await?;
        let doc = self.clone_doc().await?;
        // create collection if needed
        let payload = if let Some(dir) = payload_dir {
            Some(collection_from_dir(&doc, &dir).await?)
        } else { None };

        // create post
        let post = Post::new(me).body(text).payload(payload);

        let mut lock = self.state.write().await;
        if let Ready { ref mut messages,..  } = *lock {
            messages.create_post(post).await?;
        }

        Ok(())
    }

    pub async fn broadcast_all_messages(&self) -> Result<()> {
        let me = self.identity_service.get_default_identity_pk().await?;
        let lock = self.state.read().await;
        if let Ready { ref messages, .. } = *lock {
            let msgs: Vec<DisplayMessage> = messages.posts().await.into_iter().enumerate()
                .map(|(i,m)| { display_msg_map(i, &me, m) })
                .collect();
            broadcast(&self.bc, AllMessagesUpdated(msgs))?;
        };
        Ok(())
    }

    pub async fn clone_doc(&self) -> Result<Doc> {
        let lock = self.state.read().await;
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
        delegate.update(CollectionState::Loading).await;
        match doc.get_or_download_collection(hash).await {
            Ok(items) => {
                delegate.update(CollectionState::Loaded(items)).await;
            }
            Err(err) => {
                delegate.update(CollectionState::Failed(err.to_string())).await;
            }
        }
        Ok(())
    }

    pub async fn check_if_found_group(&self) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut found_group, ref identities,.. } = *lock {
            if identities.identities_ref().len() > 1 {
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

    pub async fn status_update(&self, pk: PublicKey, status: Status) -> Result<()> {
        let mut lock = self.state.write().await;
        if let Ready { ref mut statuses, .. } = *lock {
            statuses.insert(pk, status);
        }
        drop(lock);
        self.identities_did_update(false).await?;

        Ok(())
    }
}

fn get_document_data(ticket: &DocTicket) -> DocumentData {
    postcard::to_stdvec(&ticket.capability).expect("serializing document data")
}

fn get_address_data(ticket: &DocTicket) -> AddressData {
    postcard::to_stdvec(&ticket.nodes).expect("serializing address data")
}


impl WeakService<InnerService, Service> for Service {
    fn get_weak(&self) -> Weak<InnerService> {
        Arc::downgrade(&self.0)
    }

    fn from_weak(weak: &Weak<InnerService>) -> Option<Service> {
        weak.upgrade().map(|i|Service(i))
    }
}
