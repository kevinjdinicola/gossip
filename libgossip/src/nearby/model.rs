use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::{BlobHash, PublicKey};
use crate::nearby::MESSAGE_PAYLOADS;
use crate::nearby::MESSAGES;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[derive(uniffi::Record)]
pub struct Status {
    pub(crate) text: String
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct NearbyProfile {
    pub pk: PublicKey,
    pub name: String,
    pub pic: Option<BlobHash>,
    pub status: Status
}

#[derive(Debug, Clone)]
#[derive(uniffi::Record)]
pub struct DisplayMessage {
    pub id: u32,
    pub text: String,
    pub is_self: bool,
    pub payload: Option<BlobHash>
}

#[derive(Debug, Clone)]
#[derive(uniffi::Record)]
pub struct DebugState {
    pub doc_id: String,
    pub found_group: bool,
}


pub fn message_key(msg: &Post) -> String {
    format!("{MESSAGES}/{}",msg.created_at.to_string())
}

pub fn message_payload_key(msg: &Post) -> String {
    format!("{MESSAGE_PAYLOADS}/{}",msg.created_at.to_string())
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Post {
    pub pk: PublicKey,
    pub created_at: DateTime<Utc>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub payload: Option<BlobHash>
}

impl Post {
    pub fn new(pk: PublicKey) -> Self {
        Post {
            pk,
            created_at: Utc::now(),
            title: None,
            body: None,
            payload: None
        }
    }
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(String::from(title));
        self
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn payload(mut self, payload: Option<BlobHash>) -> Self {
        self.payload = payload;
        self
    }
}

pub fn display_msg_map(idx: usize, me: &PublicKey, msg: Post) -> DisplayMessage {
    DisplayMessage {
        id: idx as u32,
        text: msg.body.unwrap_or_else(||String::default()),
        is_self: me == &msg.pk,
        payload: msg.payload
    }
}
