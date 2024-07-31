use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use iroh::blobs::Hash;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::debug;

use crate::blob_dispatcher::BlobDataState::{Failed, Loaded, Loading};
use crate::data::BlobHash;
use crate::doc::Node;


#[derive(uniffi::Object)]
pub struct BlobDataDispatcher {
    node: Node,
}

#[derive(uniffi::Enum)]
pub enum BlobDataState {
    Empty,
    Loading,
    Loaded(BlobHash, Vec<u8>),
    Failed(String)
}

#[derive(uniffi::Enum)]
pub enum CollectionState {
    Empty,
    Loading,
    Loaded(Vec<NamedBlob>),
    Failed(String)
}

#[derive(uniffi::Record)]
pub struct NamedBlob {
    pub name: String,
    pub hash: BlobHash
}



#[uniffi::export(with_foreign)]
#[async_trait]
pub trait LoadCollectionDelegate: Send + Sync + 'static {
    async fn update(&self, state: CollectionState);
}


#[uniffi::export(with_foreign)]
#[async_trait]
pub trait BlobDataResponder: Send + Sync {
    async fn update(&self, state: BlobDataState);
    async fn hash(&self) -> Option<BlobHash>;
}

async fn load_bytes(node: &Node, hash: Hash) -> anyhow::Result<Bytes> {
    let mut r = node.blobs().read(hash).await?;
    let b = r.read_to_bytes().await?;
    Ok(b)
}



impl BlobDataDispatcher {
    pub fn new(node: Node) -> BlobDataDispatcher {
        BlobDataDispatcher {
            node
        }
    }
}

#[uniffi::export]
impl BlobDataDispatcher {
    pub async fn hydrate(&self, bdr: Arc<dyn BlobDataResponder>) {
        if let Some(bh) = bdr.hash().await {
            bdr.update(Loading).await;

            let b: Bytes = load_bytes(&self.node, bh.as_bytes().into()).await.expect("load bytes of existing blob");
            bdr.update(Loaded(bh, b.into())).await;
        }
    }
}
