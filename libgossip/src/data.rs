use std::{fmt, ptr};
use std::fmt::{Debug, Display, Formatter};

use iroh::base::base32;
use iroh::blobs::format::collection::Collection;
use iroh::blobs::Hash;
use iroh::blobs::util::SetTagOption;
use iroh::client::blobs::WrapOption;
use iroh::docs::{AuthorId, NamespaceId};
use serde::{Deserialize, Serialize};
use crate::blob_dispatcher::NamedBlob;
use crate::doc::Doc;

#[repr(C)]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[derive(uniffi::Record)]
pub struct WideId {
    p1: u64,
    p2: u64,
    p3: u64,
    p4: u64,
}

#[repr(C)]
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[derive(uniffi::Record)]
pub struct UUID {
    p1: u64,
    p2: u64
}
#[uniffi::export]
pub fn wideid_to_string(wide_id: WideId) -> String {
    format!("{wide_id}")
}

#[uniffi::export]
pub fn uuid_from_bytes(b1: u8, b2: u8, b3: u8, b4: u8, b5: u8, b6: u8, b7: u8, b8: u8,
                       b9: u8, b10: u8, b11: u8, b12: u8, b13: u8, b14: u8, b15: u8, b16: u8 ) -> UUID {
    let bytes: [u8; 16] = [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16];
    bytes.into()
}

impl WideId {
    pub fn to_bytes(self) -> [u8; 32] {
        self.into()
    }
    pub fn as_bytes(&self) -> [u8; 32] {
        self.into()
    }
}

impl UUID {
    pub fn to_bytes(self) -> [u8; 16] { self.into() }
    pub fn as_bytes(&self) -> [u8; 16] { self.into() }
}


impl From<[u8; 32]> for WideId {
    fn from(value: [u8; 32]) ->  Self {
        unsafe {
            ptr::read(value.as_ptr() as *const Self)
        }
    }
}

impl From<[u8; 16]> for UUID {
    fn from(value: [u8; 16]) ->  Self {
        unsafe {
            ptr::read(value.as_ptr() as *const Self)
        }
    }
}

impl From<WideId> for [u8; 32] {
    fn from(value: WideId) ->  Self {
        unsafe {
            ptr::read((&value as *const WideId) as *const Self)
        }
    }
}
impl From<&WideId> for [u8; 32] {
    fn from(value: &WideId) ->  Self {
        unsafe {
            ptr::read((value as *const WideId) as *const Self)
        }
    }
}

impl From<&UUID> for [u8; 16] {
    fn from(value: &UUID) ->  Self {
        unsafe {
            ptr::read((value as *const UUID) as *const Self)
        }
    }
}
impl From<UUID> for [u8; 16] {
    fn from(value: UUID) ->  Self {
        unsafe {
            ptr::read((&value as *const UUID) as *const Self)
        }
    }
}

impl From<WideId> for AuthorId {
    fn from(value: WideId) -> Self {
        AuthorId::from(<[u8; 32] as Into<AuthorId>>::into(value.to_bytes().into()))
    }
}
impl From<&WideId> for AuthorId {
    fn from(value: &WideId) -> Self {
        AuthorId::from(<[u8; 32] as Into<AuthorId>>::into(value.to_bytes()))
    }
}

impl Display for WideId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", base32::fmt(self.to_bytes()))
    }
}

impl Debug for WideId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", base32::fmt(self.to_bytes()))
    }
}

impl From<AuthorId> for WideId {
    fn from(value: AuthorId) -> Self {
        value.to_bytes().into()
    }
}

impl From<Hash> for WideId {
    fn from(value: Hash) -> Self {
        (*value.as_bytes()).into()
    }
}

impl From<WideId> for Hash {
    fn from(value: WideId) -> Self {
        value.as_bytes().into()
    }
}

impl From<iroh::base::key::PublicKey> for WideId {
    fn from(value: iroh::base::key::PublicKey) -> Self {
        (*value.as_bytes()).into()
    }
}

impl From<NamespaceId> for WideId {
    fn from(value: NamespaceId) -> Self {
        value.to_bytes().into()
    }
}
impl From<WideId> for NamespaceId {
    fn from(value: WideId) -> Self {
        value.to_bytes().into()
    }
}

pub async fn collection_from_dir(doc: &Doc, payload_dir: &str) -> anyhow::Result<BlobHash> {
    let mut hashes: Vec<(String, Hash)> = vec![];
    let mut read_stream = tokio::fs::read_dir(payload_dir).await?;
    while let Some(entry) = read_stream.next_entry().await? {
        let filename = entry.file_name();
        let file_name_str = filename.to_string_lossy(); // Get the file name

        let blob = doc.1.blobs().add_from_path(entry.path(),false, SetTagOption::Auto,WrapOption::NoWrap).await?;
        let x = blob.await?;
        hashes.push((file_name_str.to_string(), x.hash))
    }
    let collection: Collection = hashes.into_iter().collect();
    let (payload_blob,_) = doc.1.blobs().create_collection(collection,SetTagOption::Auto, vec![]).await?;
    tokio::fs::remove_dir_all(&payload_dir).await?;
    Ok(payload_blob.into())
}

pub async fn replace_or_add_blob(name: &str, hash: BlobHash, blob_list: &mut Vec<NamedBlob>) {
    let mut named_blob: Option<NamedBlob> = Some(NamedBlob { name: name.into(), hash });
    for b in blob_list.iter_mut() {
        if b.name.as_str() == named_blob.as_ref().unwrap().name {
            *b = named_blob.take().unwrap();
            break;
        }
    }
    if let Some(named_blob) = named_blob {
        blob_list.push(named_blob);
    }
}


pub type PublicKey = WideId;
// pub type ExchangeId = WideId;

pub type BlobHash = WideId;
