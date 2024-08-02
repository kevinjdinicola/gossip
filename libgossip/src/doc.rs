use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use futures_lite::stream::{Stream, StreamExt};
use iroh::blobs::{BlobFormat, Hash};
use iroh::blobs::util::SetTagOption;
use iroh::client::{authors, blobs};
use iroh::client::blobs::{AddOutcome, DownloadOptions};
use iroh::client::blobs::DownloadMode::Queued;
use iroh::client::docs::{Entry, LiveEvent};
use iroh::docs::{ContentStatus, NamespaceId};
use iroh::docs::store::Query;
use iroh::net::key::PublicKey;
use iroh::net::NodeAddr;
use iroh::node::FsNode;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;

use crate::blob_dispatcher::NamedBlob;
use crate::data::BlobHash;
use crate::doc::Origin::{Local, Remote};

pub type Node = FsNode;
pub type CoreDoc = iroh::client::docs::Doc;

#[derive(Clone)]
pub struct Doc(pub CoreDoc, pub Node);

impl Deref for Doc {
    type Target = CoreDoc;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub enum Origin { Local, Remote(PublicKey) }
pub struct InsertEntry {
    pub location: Origin,
    pub entry: Entry,
    pub content_status: ContentStatus
}


impl Doc {
    pub fn blobs(&self) -> &blobs::Client {
        self.1.blobs()
    }

    pub fn authors(&self) -> &authors::Client {
        self.1.authors()
    }

    pub async fn start_sync_with_known_peers(&self) -> Result<()> {
        let nodes = self.get_peer_nodes().await;
        self.start_sync(nodes).await?;
        Ok(())
    }

    pub async fn get_or_download_collection(&self, hash: BlobHash) -> Result<Vec<NamedBlob>> {
        let collection = if let Ok(collection) = self.blobs().get_collection(hash.into()).await {
            collection
        } else {
            let downloading = self.blobs().download_with_opts(hash.into(), DownloadOptions {
                format: BlobFormat::HashSeq,
                nodes: self.get_peer_nodes().await,
                tag: SetTagOption::Auto,
                mode: Queued
            }).await?;
            let done = downloading.finish().await?;
            println!("downloaded collection size {}", done.downloaded_size);
            self.blobs().get_collection(hash.into()).await?
        };
        let blobs: Vec<NamedBlob> = collection.into_iter().map(|(name,hash)|
            NamedBlob { name, hash: hash.into() }
        ).collect();

        Ok(blobs)
    }

    pub async fn get_peer_nodes(&self) -> Vec<NodeAddr> {
        if let Some(peers) = self.0.get_sync_peers().await.expect("get sync peers") {
            let peers: Vec<NodeAddr>= peers.iter().map(|peer_id_bytes| {
                NodeAddr::new(iroh::net::key::PublicKey::from_bytes(peer_id_bytes).unwrap())
            }).collect();
            peers
        } else {
            vec![]
        }
    }

    pub async fn subscribe(&self) -> Result<impl Stream<Item = InsertEntry>> {
        let stream = self.0.subscribe().await?;
        let mut refmap: HashMap<Hash, InsertEntry> = HashMap::new();

        Ok(stream.filter_map(move |e| {
            match e {
                Ok(LiveEvent::InsertLocal { entry }) => {
                    Some(InsertEntry { location: Local, entry, content_status: ContentStatus::Complete })
                }
                Ok(LiveEvent::InsertRemote { from, entry, content_status }) => {
                    let hash_key = entry.content_hash();
                    let ie = InsertEntry { location: Remote(from), entry, content_status };
                    if matches!(content_status, ContentStatus::Complete) {
                        Some(ie)
                    } else {
                        refmap.insert(hash_key, ie);
                        None
                    }
                }
                Ok(LiveEvent::ContentReady { hash}) => {
                    if let Some(ie) = refmap.remove(&hash) {
                        Some(ie)
                    } else {
                        None
                    }
                },
                Ok(_) => { None },
                Err(e) => {
                    println!("some crazy error! {e}");
                    None
                }

            }
        }))

    }

    pub async fn list_entries_by_query(&self, query: impl Into<Query>) -> Result<Vec<Entry>> {
        let mut stream = self.0.get_many(query).await?;
        let mut output: Vec<Entry> = vec![];
        while let Some(Ok(entry)) = stream.next().await {
            output.push(entry);
        };
        Ok(output)
    }

    pub async fn read_blobs_by_query<T: for<'a> Deserialize<'a>>(&self, query: impl Into<Query>) -> Result<Vec<T>> {
        let mut stream = self.0.get_many(query).await?;
        let mut output: Vec<T> = vec![];
        while let Some(Ok(entry)) = stream.next().await {
            output.push(self.read_blob_by_hash(entry.content_hash()).await?);
        }
        Ok(output)
    }

    pub async fn read_blob_by_hash<T: for<'a> Deserialize<'a>>(&self, hash: Hash) -> Result<T> {
        let bytes = self.1.blobs().read_to_bytes(hash).await?;
        let r = flexbuffers::Reader::get_root(bytes.iter().as_slice()).unwrap();
        let decoded = T::deserialize(r).expect("Deserialization failed");
        Ok(decoded)
    }

    pub async fn read_any_blob<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<Option<T>> {
        let hash = self.0.get_one(Query::key_exact(key)).await?;
        self.maybe_read_blob(hash).await
    }
    pub async fn read_own_blob<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<Option<T>> {
        let author = self.1.authors().default().await.unwrap();
        let hash = self.0.get_exact(author, key,false).await?;
        self.maybe_read_blob(hash).await
    }

    async fn maybe_read_blob<T: for<'a> Deserialize<'a>>(&self, hash: Option<Entry>) -> Result<Option<T>> {
        match hash {
            None => Ok(None),
            Some(entry) => {
                let decoded: T = self.read_blob_by_hash(entry.content_hash()).await?;
                Ok(Some(decoded))
            }
        }
    }

    pub async fn write_blob<T: Serialize>(&self, key: &str, data: T) -> anyhow::Result<AddOutcome> {
        let mut s = flexbuffers::FlexbufferSerializer::new();
        data.serialize(&mut s).expect("Serialization failure");
        let add = self.1.blobs().add_bytes(s.take_buffer()).await.expect("Persistence failure");
        let author = self.1.authors().default().await.unwrap();
        self.0.set_hash(author, String::from(key), add.hash, add.size).await.expect("Hash set failure");
        Ok(add)
    }
}



pub async fn create_or_load_from_fs_reference(node: &Node, path: impl AsRef<Path>) -> Doc {
    let doc = if path.as_ref().exists() {
        let mut file = File::open(&path).await.expect("Couldn't open existing settings namespace");
        let mut contents: [u8; 32] = [0; 32];
        if let Ok(_) = file.read(&mut contents).await {
            let read_namespace = NamespaceId::from(&contents);
            info!("Existing settings namespace found, opening: {}", read_namespace);
            node.docs().open(read_namespace).await.unwrap_or(None)
        } else {
            info!("Error reading namespace file contents, recreating");
            None
        }
    } else {
        None
    };
    match doc{
        Some(doc) => Doc(doc, node.clone()),
        None => {
            let doc = node.docs().create().await.unwrap();
            let mut file = File::create(&path).await.expect("couldnt open");
            let created_ns = doc.id();
            file.write(created_ns.as_bytes()).await.expect("Failed to save settings doc");
            Doc(doc, node.clone())
        }
    }
}

pub fn key_of(entry: &Entry) -> Cow<'_, str> {
    String::from_utf8_lossy(entry.key())
}

pub fn value_after<'a, 'b>(prefix: &'b str, key: &'a str) -> &'a str {
    &key[prefix.len()+1..]
}

// #[allow(async_fn_in_trait)]
// pub trait BlobsSerializer {
//
//     async fn deserialize_read_blob<T: for<'a> Deserialize<'a>>(&self, hash: Hash) -> anyhow::Result<T>;
//
//     async fn serialize_write_blob<T: Serialize>(&self, data: T) -> anyhow::Result<AddOutcome>;
// }
//
// impl<C> BlobsSerializer for iroh::node::Node<C> {
//     async fn deserialize_read_blob<T: for<'a> Deserialize<'a>>(&self, hash: Hash) -> anyhow::Result<T> {
//         let bytes = self.blobs().read_to_bytes(hash).await?;
//
//         let r = flexbuffers::Reader::get_root(bytes.iter().as_slice()).unwrap();
//         let decoded = T::deserialize(r).expect("Deserialization failed");
//         Ok(decoded)
//     }
//
//     async fn serialize_write_blob<T: Serialize>(&self, data: T) -> anyhow::Result<AddOutcome> {
//         let mut s = flexbuffers::FlexbufferSerializer::new();
//         data.serialize(&mut s).expect("Serialization failure");
//         self.blobs().add_bytes(s.take_buffer()).await
//     }
// }
//
