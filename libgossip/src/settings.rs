use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use futures_lite::StreamExt;
use iroh::base::key::PublicKey;
use iroh::blobs::format::collection::Collection;
use iroh::blobs::store::Store;
use iroh::blobs::util::SetTagOption;
use iroh::client::blobs::{AddOutcome, BlobStatus};
use iroh::client::docs::Entry;
use iroh::docs::NamespaceId;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, Sender};

pub use Service as SettingsService;
use crate::blob_dispatcher::NamedBlob;
use crate::data::{BlobHash, WideId};

use crate::doc::{Doc, key_of};
use crate::events::{broadcast, create_broadcast};
use crate::nearby::BIO;
use crate::nearby::model::{BioDetails, message_payload_key, Status};
use crate::settings::SettingsEvent::{OwnPublicBioUpdated, StatusSettingChanged};

const NODE_SETTINGS_FILE: &str = "node_root_settings_doc.bin";

pub const CURRENT_STATUS_SETTING: &str = "current_status";
pub const CURRENT_NEARBY_DOC_ID: &str = "current_nearby_doc_id";

pub const SETTINGS_STORE_KEY: &str = "settings_store";

#[derive(Clone, Debug)]
pub enum SettingsEvent {
    StatusSettingChanged(Status),
    OwnPublicBioUpdated(Entry),
    SettingChanged(String, Option<StoreValue>, StoreValue)
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


pub const SHARE_NEARBY_PUBLIC_BIO: &str = "share_nearby_public_bio";

// pub enum SettingKey {
//     ShareNearbyPublicBio("THING")
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StoreValue {
    Bool(bool), String(String), Int(i32), WideId(WideId)
}
#[derive(Serialize, Deserialize)]
struct SettingsStore(HashMap<String, StoreValue>);
impl SettingsStore {
    fn new() -> Self {
        SettingsStore(HashMap::new())
    }

}

impl <T> From<StoreValue> for Option<T>
where
    T: OptionValue + Sized
{
    fn from(value: StoreValue) -> Self {
        T::from_store_value(value)
    }
}


trait OptionValue: Sized + Debug {
    fn from_store_value(v: StoreValue) -> Option<Self>;
    fn make_store_value(v: Self) -> StoreValue;
}
impl OptionValue for bool {
    fn from_store_value(v: StoreValue) -> Option<Self> {
        match v {
            StoreValue::Bool(b) => Some(b),
            _ => None
        }
    }

    fn make_store_value(v: Self) -> StoreValue {
        StoreValue::Bool(v)
    }
}

impl OptionValue for WideId {
    fn from_store_value(v: StoreValue) -> Option<Self> {
        match v {
            StoreValue::WideId(w) => Some(w),
            _ => None

        }
    }

    fn make_store_value(v: Self) -> StoreValue {
        StoreValue::WideId(v)
    }
}
impl OptionValue for i32 {
    fn from_store_value(v: StoreValue) -> Option<Self> {
        match v {
            StoreValue::Int(i) => Some(i),
            _ => None
        }
    }

    fn make_store_value(v: Self) -> StoreValue {
        StoreValue::Int(v)
    }
}
impl OptionValue for String {
    fn from_store_value(v: StoreValue) -> Option<Self> {
        match v {
            StoreValue::String(s) => Some(s),
            _ => None
        }
    }
    fn make_store_value(v: Self) -> StoreValue {
        StoreValue::String(v)
    }
}

impl <T: OptionValue> From<T> for StoreValue {
    fn from(value: T) -> Self {
        T::make_store_value(value)
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
                BIO => {
                    broadcast(&self.bc, OwnPublicBioUpdated(e.entry))?;
                }
                _ => {}
            }
        };
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<SettingsEvent> {
        self.bc.subscribe()
    }
    pub async fn get_bio_entry(&self) -> Result<Option<Entry>> {
        self.root_doc.get_exact(self.root_doc.me().await, BIO, false).await
    }

    pub async fn get_bio(&self) -> Result<Vec<NamedBlob>> {
        let bio = self.get_bio_entry().await?;
        match bio {
            None => {
                Ok(vec![])
            }
            Some(bio) => {
                self.root_doc.get_or_download_collection(bio.content_hash().into()).await
            }
        }
    }

    pub async fn set_bio(&self, blobs: Vec<NamedBlob>) -> Result<()> {
        self.root_doc.set_collection(BIO, blobs).await
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
        self.root_doc.write_keyed_blob(CURRENT_STATUS_SETTING, &status).await?;
        Ok(())
    }

    async fn get_settings_store(&self) -> Result<SettingsStore> {
        let maybe_settings: Option<SettingsStore> = self.root_doc.read_own_blob(SETTINGS_STORE_KEY).await?;
        Ok(match maybe_settings {
            None => {
                let new_settings = SettingsStore::new();
                self.set_settings_store(&new_settings).await?;
                new_settings
            }
            Some(s) => s
        })
    }

    async fn set_settings_store(&self, ss: &SettingsStore) -> Result<AddOutcome> {
        self.root_doc.write_keyed_blob(SETTINGS_STORE_KEY, ss).await
    }

    pub async fn bullshit(&self) -> Result<()> {
        let mut ss: HashMap<String, StoreValue> = HashMap::new();
        // ss.0.insert(SettingKey::ShareNearbyPublicBio, StoreValue::Bool(true));
        ss.insert(String::from(SHARE_NEARBY_PUBLIC_BIO), StoreValue::Bool(true));
        self.root_doc.write_keyed_blob(SETTINGS_STORE_KEY, &ss).await?;
        let out: Option<HashMap<String, StoreValue>> = self.root_doc.read_own_blob(SETTINGS_STORE_KEY).await?;
        let x = out.unwrap();
        let z = x.get(SHARE_NEARBY_PUBLIC_BIO);
        println!("{:?}", z);
        Ok(())
    }


    pub async fn get_setting<T: OptionValue>(&self, key: &str) -> Result<Option<T>> {
        let ss = self.get_settings_store().await?;
        let value: Option<T> = read_setting_from_store(&key, &ss);
        Ok(value)
    }
    pub async fn get_setting_defaulted<T, F>(&self, key: &str, default: F) -> Result<T>
    where
        F: Fn() -> T,
        T: OptionValue
    {
        let ss = self.get_settings_store().await?;
        let value: Option<T> = read_setting_from_store(&key, &ss);

        Ok(value.unwrap_or_else(default))
    }

    pub async fn set_setting<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: OptionValue + Debug + Clone
    {
        let sv: StoreValue = value.into();
        let new_value = sv.clone();
        let mut ss = self.get_settings_store().await?;
        let old_value: Option<StoreValue> = ss.0.get(key).cloned();

        ss.0.insert(String::from(key), sv);
        self.set_settings_store(&ss).await?;
        broadcast(&self.bc, SettingsEvent::SettingChanged(String::from(key), old_value, new_value))?;
        Ok(())
    }

    pub async fn identity_doc(&self) -> &Doc {
        &self.root_doc
    }

}

fn read_setting_from_store<T: OptionValue>(key: &str, store: &SettingsStore) -> Option<T> {
    let value: Option<StoreValue> = store.0.get(key).cloned();
    let value: Option<T> = match value {
        None => None,
        Some(v) => {
            v.into()
        }
    };
    value
}
pub fn settings_file_path(base_path: impl AsRef<Path>) -> impl AsRef<Path> {
    base_path.as_ref().join(NODE_SETTINGS_FILE)
}