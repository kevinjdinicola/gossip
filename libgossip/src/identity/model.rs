use serde::{Deserialize, Serialize};
use crate::data::{BlobHash, PublicKey, WideId};

pub const IDENTITY: &str = "identity";
pub const ID_PIC: &str = "id_pic";

pub const IDENTITY_PREFIX: &str = "identity/by_pk";
pub const ID_PIC_PREFIX: &str = "identity/pic/by_pk";
pub fn identity_prefix(pk: WideId) -> String {
    format!("{IDENTITY_PREFIX}/{pk}")
}

pub fn identity_pic_prefix(pk: WideId) -> String {
    format!("{ID_PIC_PREFIX}/{pk}")
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[derive(uniffi::Record)]
pub struct Identity {
    pub(crate) name: String,
    pub(crate) pk: PublicKey
}

#[derive(Clone, Debug)]
pub enum IdentityServiceEvents {
    DefaultIdentityUpdated(Identity),
    DefaultIdentityPicUpdated(BlobHash, u64)
}