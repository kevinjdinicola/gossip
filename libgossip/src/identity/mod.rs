use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use futures_util::StreamExt;
use iroh::client::blobs::AddOutcome;
use iroh::client::docs::Entry;
use iroh::docs::store::Query;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::data::{BlobHash, PublicKey};
use crate::doc::{Doc, key_of, value_after};
use crate::events::{broadcast, create_broadcast};
use crate::identity::model::{ID_PIC_PREFIX, Identity, identity_pic_prefix, identity_prefix, IDENTITY_PREFIX, IdentityServiceEvents};
use crate::identity::model::IdentityServiceEvents::{DefaultIdentityPicUpdated, DefaultIdentityUpdated};

pub use self::Service as IdentityService;

pub mod model;
pub mod domain;


#[derive(Clone)]
pub struct Service(Arc<InnerService>);
impl Deref for Service {
    type Target = InnerService;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
pub struct InnerService {
    bc: Sender<IdentityServiceEvents>,
    doc: Doc
}
impl Service{
    pub fn new(doc: Doc) -> Service {
        let s = Service(Arc::new(InnerService{
            bc: create_broadcast(),
            doc,
        }));
        let o = s.clone();
        tokio::spawn(async move { o.observe_doc().await });
        s
    }
}

impl InnerService {
    pub fn subscribe(&self) -> Receiver<IdentityServiceEvents> {
        self.bc.subscribe()
    }
    pub async fn observe_doc(&self) -> Result<()> {
        let mut stream = self.doc.subscribe().await?;
        while let Some(e) = stream.next().await {
            match key_of(&e.entry) {
                s if s.starts_with(IDENTITY_PREFIX) => {
                    let i: Identity = self.doc.read_blob_by_hash(e.entry.content_hash()).await?;
                    self.identity_inserted(e.entry, i).await?;
                }
                s if s.starts_with(ID_PIC_PREFIX) => {
                    self.id_pic_inserted(e.entry).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn identity_inserted(&self, entry: Entry, iden: Identity) -> Result<()> {
        let key = key_of(&entry);
        let value = value_after(IDENTITY_PREFIX, &key);


        println!("got new identity {}", value);
        if iden.pk == self.get_default_identity_pk().await? {
            broadcast(&self.bc, DefaultIdentityUpdated(iden))?;
        }
        Ok(())
    }

    pub async fn id_pic_inserted(&self, entry: Entry) -> Result<()> {
        let key = key_of(&entry);
        let pk_from_key = value_after(ID_PIC_PREFIX, &key);

        let default = self.get_default_identity_pk().await?;
        if pk_from_key == default.to_string() {
            println!("IDSVC {}, its {}, size {}", key, entry.content_hash(), entry.content_len());
            broadcast(&self.bc, DefaultIdentityPicUpdated(entry.content_hash().into(), entry.content_len()))?;
        }
        Ok(())
    }

    // use default identity thing here
    pub async fn get_default_identity_pk(&self) -> Result<PublicKey> {
        let author = self.doc.authors().default().await?;
        Ok(author.into())
    }

    pub async fn get_default_identity(&self) -> Result<Option<Identity>> {
        let pk = self.get_default_identity_pk().await?;
        self.get_identity(pk).await
    }

    pub async fn get_identity(&self, pk: PublicKey) -> Result<Option<Identity>> {
        self.doc.read_own_blob(identity_prefix(pk).as_str()).await
    }

    pub async fn get_pic(&self, pk: PublicKey) -> Result<Option<(BlobHash, u64)>> {
        let entry = self.doc.0.get_one(Query::key_exact(identity_pic_prefix(pk))).await?;
        if let Some(e) = entry {
            Ok(Some((e.content_hash().into(), e.content_len())))
        } else {
            Ok(None)
        }
    }

    pub async fn save_identity(&self, iden: &Identity) -> Result<AddOutcome> {
        let id = iden.pk;
        let blob = self.doc.write_blob(identity_prefix(id).as_str(), iden).await?;
        Ok(blob)
    }

    pub async fn set_pic(&self, pk: PublicKey, data: Vec<u8>) -> Result<BlobHash> {
        let author = self.doc.authors().default().await?;
        let add_res = self.doc.blobs().add_bytes(data).await?;
        self.doc.set_hash(author, identity_pic_prefix(pk), add_res.hash, add_res.size).await?;

        let bh: BlobHash = add_res.hash.into();
        println!("I set add_res.hash as {}, its bh is {}", add_res.hash, bh);
        Ok(bh)
    }
}
