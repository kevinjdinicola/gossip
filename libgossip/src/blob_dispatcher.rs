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
    bdr_tx: Sender<Arc<dyn BlobDataResponder>>,
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
pub trait BlobDataResponder: Send + Sync {
    fn update(&self, state: BlobDataState);
    fn hash(&self) -> Option<BlobHash>;
}


async fn worker_loop(node: Node, mut bdr_rx: Receiver<Arc<dyn BlobDataResponder>>) {
    // this will die when the tx is dropped, nice little actor
    debug!("BlobDataDispatcher worker loop started");

    while let Some(bdr) = bdr_rx.recv().await {
        let nclone = node.clone();
        tokio::spawn(async move {
            hydrate_responder(nclone, bdr).await
        });
    }
    debug!("BlobDataDispatcher worker ending");
}

async fn hydrate_responder(node: Node, bdr: Arc<dyn BlobDataResponder>) {
    if let Some(bh) = bdr.hash() {
        bdr.update(Loading);

        let res: anyhow::Result<Bytes> = load_bytes(&node, bh.as_bytes().into()).await;
        match res {
            Ok(b) => {
                bdr.update(Loaded(bh, b.into()))
            }
            Err(e) => {
                bdr.update(Failed(e.to_string()))
            }
        }
    }
}

async fn load_bytes(node: &Node, hash: Hash) -> anyhow::Result<Bytes> {
    let mut r = node.blobs().read(hash).await?;
    let b = r.read_to_bytes().await?;
    Ok(b)
}



impl BlobDataDispatcher {
    pub fn new(node: Node) -> BlobDataDispatcher {
        let (tx, rx) = mpsc::channel(16);

        tokio::spawn(async move {
            worker_loop(node, rx).await;
        });

        BlobDataDispatcher {
            bdr_tx: tx
        }
    }
}

#[uniffi::export]
impl BlobDataDispatcher {
    pub fn hydrate(&self, bdr: Arc<dyn BlobDataResponder>) {
        if bdr.hash().is_some() {
            self.bdr_tx.try_send(bdr).expect("Failed to send BDR");
        }
    }
}
