use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::docs::AuthorId;
use iroh::docs::store::Query;
use tokio::sync::broadcast::{Receiver, Sender};

pub use Service as SettingsService;

use crate::doc::{Doc, key_of};
use crate::events::{broadcast, create_broadcast};
use crate::identity::model::{ID_PIC, IDENTITY, Identity};
use crate::nearby::model::Status;
use crate::nearby::PUBLIC_STATUS;
use crate::settings::SettingsEvent::StatusSettingChanged;

const NODE_SETTINGS_FILE: &str = "node_root_settings_doc.bin";

pub const CURRENT_STATUS_SETTING: &str = "current_status";
pub const DEFAULT_IDENTITY: &str = "default_identity";

#[derive(Clone, Debug)]
pub enum SettingsEvent {
    StatusSettingChanged(Status)
}

#[derive(Clone)]
pub struct Service(Arc<ServiceInner>);
pub struct ServiceInner {
    bc: Sender<SettingsEvent>,
    root_doc: Doc,
}

impl Deref for Service {
    type Target = ServiceInner;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Service {
    pub fn new(root_doc: Doc) -> Service {
        let inner = Arc::new(ServiceInner {
            bc: create_broadcast(),
            root_doc
        });
        let s = Service(inner);
        let o = s.clone();
        tokio::spawn(async move { o.doc_watch_loop().await });
        s
    }

    pub async fn doc_watch_loop(&self) -> Result<()> {
        let mut stream = self.root_doc.subscribe().await?;
        while let Some(e) = stream.next().await {
            match key_of(&e.entry).as_ref() {
                CURRENT_STATUS_SETTING => {
                    let s: Status = self.root_doc.read_blob_by_hash(e.entry.content_hash()).await?;
                    broadcast(&self.bc, StatusSettingChanged(s))?;
                }
                _ => {}
            }
        };
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<SettingsEvent> {
        self.bc.subscribe()
    }

    pub async fn get_status(&self) -> Result<Status> {
        if let Some(status) = self.root_doc.read_own_blob(CURRENT_STATUS_SETTING).await? {
            Ok(status)
        } else {
            let new_status = Status { text: String::from("whats up?") };
            self.set_status(&new_status).await?;
            Ok(new_status)
        }
    }

    pub async fn set_status(&self, status: &Status) -> Result<()> {
        self.root_doc.write_blob(CURRENT_STATUS_SETTING, &status).await?;
        Ok(())
    }
    pub async fn identity_doc(&self) -> &Doc {
        &self.root_doc
    }

}

pub fn settings_file_path(base_path: impl AsRef<Path>) -> impl AsRef<Path> {
    base_path.as_ref().join(NODE_SETTINGS_FILE)
}