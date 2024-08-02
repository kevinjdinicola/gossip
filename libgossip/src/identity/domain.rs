use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Weak;
use async_trait::async_trait;
use crate::data::{BlobHash, PublicKey};
use crate::doc::{Doc, InsertEntry, key_of};
use crate::identity::model::{ID_PIC, Identity, IDENTITY};
use anyhow::Result;
use iroh::client::docs::Entry;

use iroh::docs::store::Query;
use crate::events::WeakService;

#[async_trait]
pub trait IdentityDomainResponder: Send + Sync + 'static {
    async fn identities_did_update(&self, added_new: bool) -> Result<()>;
    async fn pics_did_update(&self) -> Result<()>;
}

pub struct IdentityDomain<S, I>
{
    doc: Doc,
    identities: Vec<Identity>,
    pics: HashMap<PublicKey, BlobHash>,
    responder: Weak<I>,
    _phantom: PhantomData<S>
}

impl<S, I> IdentityDomain<S, I>
where
    S: IdentityDomainResponder + WeakService<I,S>
{
    pub fn new(doc: &Doc, responder: &S) -> Self
    {
        IdentityDomain {
            doc: doc.clone(),
            identities: vec![],
            pics: HashMap::new(),
            responder: responder.get_weak(),
            _phantom: PhantomData
        }
    }

    pub fn set_doc(&mut self, doc: &Doc) {
        self.doc = doc.clone()
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.identities.clear();
        self.pics.clear();

        let loaded_identities: Vec<Identity> = self.doc.read_blobs_by_query(Query::key_exact(IDENTITY)).await?;
        println!("loading doc {}, got idens {:?}", self.doc.id(), loaded_identities);
        let loaded_pics = self.doc.list_entries_by_query(Query::key_exact(ID_PIC)).await?;
        self.identities = loaded_identities;
        loaded_pics.into_iter().for_each(|entry| {
            self.pics.insert(entry.author().into(), entry.content_hash().into());
        });
        Ok(())
    }

    pub fn identities(&self) -> Vec<Identity> {
        self.identities.clone()
    }

    pub fn identities_ref(&self) -> &Vec<Identity> {
        &self.identities
    }


    pub fn pics(&self) -> &HashMap<PublicKey, BlobHash> {
        &self.pics
    }

    async fn identity_updated(&mut self, updated_iden: Identity) -> Result<()> {
        println!("IDENTITY UPDATED {}", updated_iden.name);
        let mut updated_iden = Some(updated_iden);
        let mut added_new_iden = false;


        for iden in self.identities.iter_mut() {
            if iden.pk == updated_iden.as_ref().unwrap().pk {
                *iden = updated_iden.take().unwrap();
                break; // found the iden to update, break out of this loop cause unwrap will fail
            }
        }
        if let Some(updated) = updated_iden {
            self.identities.push(updated);
            added_new_iden = true
        }
        println!("UPDATED IDEN IS NEW {}", added_new_iden);

        if let Some(resp) = S::from_weak(&self.responder) {
            tokio::spawn(async move {
                resp.identities_did_update(added_new_iden).await.expect("shit")
            });
        }
        Ok(())
    }

    async fn id_pic_updated(&mut self, entry: Entry) -> Result<()> {
        self.pics.insert(entry.author().into(), entry.content_hash().into());
        if let Some(resp) = S::from_weak(&self.responder) {
            tokio::spawn(async move {
                resp.pics_did_update().await.expect("shit")
            });
        }
        Ok(())
    }

    pub fn handles(&self, key: &str) -> bool {
        key == IDENTITY || key == ID_PIC
    }

    pub async fn insert_entry(&mut self, e: InsertEntry) -> Result<()> {
        let key = key_of(&e.entry);
        match key.as_ref() {
            IDENTITY => {
                let i: Identity = self.doc.read_blob_by_hash(e.entry.content_hash()).await?;
                self.identity_updated(i).await?;
            }
            ID_PIC => {
                self.id_pic_updated(e.entry).await?;
            }
            _ => {
                eprintln!("unsupported key {}", key)
            }
        };
        Ok(())
    }
}
