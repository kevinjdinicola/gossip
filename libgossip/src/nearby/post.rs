use std::marker::PhantomData;
use std::sync::Weak;

use anyhow::Result;
use async_trait::async_trait;
use iroh::client::blobs::BlobStatus;
use iroh::docs::store::{Query, SortBy, SortDirection};

use crate::doc::{Doc, InsertEntry};
use crate::events::WeakService;
use crate::nearby::MESSAGES;
use crate::nearby::model::{message_key, message_payload_key, Post};

#[async_trait]
pub trait PostDomainResponder: Send + Sync + 'static {
    async fn all_posts_updated(&self, posts: Vec<Post>) -> Result<()>;
    async fn one_post_updated(&self, count: usize, post: Post) -> Result<()>;
}
pub struct PostDomain<S, I>
{
    doc: Doc,
    posts: Vec<Post>,
    responder: Weak<I>,
    _phantom: PhantomData<S>
}

impl<S, I> PostDomain<S, I>
where
    S: PostDomainResponder + WeakService<I,S>
{
    pub fn new(doc: &Doc, responder: &S) -> Self
    {
        PostDomain {
            doc: doc.clone(),
            posts: vec![],
            responder: responder.get_weak(),
            _phantom: PhantomData
        }
    }
    pub fn set_doc(&mut self, doc: &Doc) {
        self.doc = doc.clone()
    }
    pub async fn initialize(&mut self) -> Result<()> {
        self.posts.clear();
        let posts: Vec<Post> = self.doc.read_blobs_by_query(Query::key_prefix(MESSAGES).sort_by(SortBy::KeyAuthor, SortDirection::Asc)).await?;
        self.posts = posts;
        Ok(())
    }

    pub async fn posts(&self) -> Vec<Post> {
        self.posts.clone()
    }

    pub async fn create_post(&self, p: Post) -> Result<()> {
        let key = message_key(&p);
        self.doc.write_blob(&key,&p).await?;
        if let Some(b) = p.payload {
            if let BlobStatus::Complete { size } = self.doc.blobs().status(b.into()).await? {
                let key = message_payload_key(&p);
                self.doc.set_hash(p.pk.into(), key, b.into(), size).await?;
            } else {
                eprintln!("failed to add payload to message")
            };
        }
        Ok(())
    }

    pub fn handles(&self, key: &str) -> bool {
        key.starts_with(MESSAGES)
    }

    pub async fn insert_entry(&mut self, e: InsertEntry) -> Result<()> {
        let post: Post = self.doc.read_blob_by_hash(e.entry.content_hash().into()).await?;
        self.insert_post(post).await?;
        Ok(())
    }



    async fn insert_post(&mut self, p: Post) -> Result<()> {
        let mut requires_reload = false;

        if let Some(last) = self.posts.last() {
            if p.created_at.lt(&last.created_at) {
                // we're out of order
                requires_reload = true;
            }
        }
        self.posts.push(p);
        let count = self.posts.len();

        if requires_reload {
            self.posts.sort_by(|a,b| {
                a.created_at.partial_cmp(&b.created_at).unwrap()
            });
            if let Some(resp) = S::from_weak(&self.responder) {
                let posts = self.posts.clone();
                tokio::spawn(async move {
                    resp.all_posts_updated(posts).await.expect("fuck");
                });

            }
        } else {
            if let Some(resp) = S::from_weak(&self.responder) {
                let post = self.posts.last().cloned().unwrap();
                tokio::spawn(async move {
                    resp.one_post_updated(count, post).await.expect("fuck");
                });
            }
        }

        Ok(())
    }
}

