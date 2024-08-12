use std::sync::Arc;

use anyhow::{Error, Result};
use async_trait::async_trait;

use crate::blob_dispatcher::LoadCollectionDelegate;
use crate::data::{BlobHash, WideId};
use crate::events::{start_with, Starter};
use crate::identity::IdentityService;
use crate::identity::model::{Identity, IdentityServiceEvents};
use crate::nearby::{NearbyService, NearbyServiceEvents};
use crate::nearby::model::{ConState, DocData, DisplayMessage, NearbyProfile, Status};
use crate::settings::{SettingsEvent, SettingsService};
use crate::views::errors::GossipError;

pub mod errors;
pub mod node_stat;
pub mod nearby_details;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GlobalViewModel: Send + Sync + 'static {

    async fn own_public_key_updated(&self, pk: WideId);
    async fn name_updated(&self, name: String);
    async fn pic_updated(&self, pic: BlobHash);
    async fn broadcasting_updated(&self, broadcasting: bool);
    async fn nearby_profiles_updated(&self, profiles: Vec<NearbyProfile>);
    async fn status_updated(&self, status: Status);
    async fn doc_data_updated(&self, status: DocData);

    async fn connection_state_updated(&self, state: ConState);

    async fn all_messages_updated(&self, messages: Vec<DisplayMessage>);

    async fn received_one_message(&self, message: DisplayMessage);
}

#[derive(uniffi::Object, Clone)]
pub struct Global {
    identity_service: IdentityService,
    nearby_service: NearbyService,
    settings_service: SettingsService,
    view_model: Arc<dyn GlobalViewModel>
}
impl Global {
    pub fn new(view_model: Arc<dyn GlobalViewModel>, identity_service: IdentityService, nearby_service: NearbyService, settings_service: SettingsService) -> Arc<Self> {
        Arc::new(start_with(Global {
            identity_service,
            nearby_service,
            view_model,
            settings_service
        }))
    }
}

#[async_trait]
impl Starter for Global {
    async fn start(&self) -> std::result::Result<(), Error> {
        self.view_model.status_updated(self.settings_service.get_status().await?).await;
        self.view_model.connection_state_updated(self.nearby_service.get_con_state().await).await;
        if let Some(iden) = self.identity_service.get_default_identity().await? {
            self.view_model.own_public_key_updated(iden.pk).await;
            self.view_model.name_updated(iden.name).await;
            if let Some((pic,_)) = self.identity_service.get_pic(iden.pk).await? {
                self.view_model.pic_updated(pic).await;
            }
        }
        self.view_model.broadcasting_updated(self.nearby_service.should_broadcast().await).await;


        self.listen().await;
        Ok(())
    }
}

impl Global {

    async fn listen(&self) {
        let mut iden_sub = self.identity_service.subscribe();
        let mut nearby_sub = self.nearby_service.subscribe();
        let mut settings_sub = self.settings_service.subscribe();

        self.nearby_service.broadcast_doc_data().await;
        self.nearby_service.broadcast_all_messages().await.expect("broadcast all messages");

        let mut listen = true;
        while listen {
            tokio::select! {
                Ok(e) = settings_sub.recv() => {
                    match e {
                        SettingsEvent::StatusSettingChanged(s) => {
                            self.view_model.status_updated(s).await;
                        }
                        SettingsEvent::OwnPublicBioUpdated(_) => {}
                        SettingsEvent::SettingChanged(_,_,_) => {}}
                }
                Ok(e) = iden_sub.recv() => {
                    match e {
                        IdentityServiceEvents::DefaultIdentityUpdated(i) => {
                            self.view_model.name_updated(i.name).await;
                        }
                        IdentityServiceEvents::DefaultIdentityPicUpdated(hash, _) => {
                            self.view_model.pic_updated(hash).await;
                        }
                    }
                }
                Ok(e) = nearby_sub.recv() => {
                    match e {
                        NearbyServiceEvents::BroadcastingUpdated(val) => {
                            self.view_model.broadcasting_updated(val).await;
                        },
                        NearbyServiceEvents::IdentitiesUpdated(idens) => {
                            self.view_model.nearby_profiles_updated(idens).await;
                        },
                        NearbyServiceEvents::DocDataUpdated(state) => {
                            self.view_model.doc_data_updated(state).await;
                        },
                        NearbyServiceEvents::AllMessagesUpdated(msgs) => {
                            self.view_model.all_messages_updated(msgs).await;
                        }
                        NearbyServiceEvents::ReceivedOneNewMessage(msg) => {
                            self.view_model.received_one_message(msg).await;
                        },
                        NearbyServiceEvents::ConStateUpdated(c) => {
                            self.view_model.connection_state_updated(c).await;
                        }
                        _ => {}
                    }
                },
                else => {
                    listen = false;
                }
            }
        }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl Global {

    pub async fn leave_nearby_group(&self) -> Result<(), GossipError> {
        self.nearby_service.leave_group().await?;
        Ok(())
    }

    pub async fn set_broadcasting(&self, should_broadcast: bool)-> Result<(), GossipError> {
        self.nearby_service.update_ble_broadcast(should_broadcast).await?;
        Ok(())
    }

    pub async fn set_status(&self, status: String) {
        let s = Status { text: status };
        self.settings_service.set_status(&s).await.expect("set status")
    }

    pub async fn send_message(&self, text: String, payload_dir: Option<String>) {
        self.nearby_service.post_message(text, payload_dir).await.expect("send message");
    }

    pub async fn start_sync(&self) -> Result<(), GossipError> {
        self.nearby_service.start_sync().await?;
        Ok(())
    }

    pub async fn cancel_connection_attempt(&self) -> Result<(), GossipError> {
        self.nearby_service.cancel_connection_attempt().await?;
        Ok(())
    }

    pub async fn start_scanning(&self) -> Result<(), GossipError> {
        self.nearby_service.start_scanning().await?;
        Ok(())
    }

    pub async fn load_nearby_payload(&self, hash: BlobHash, collection_delegate: Arc<dyn LoadCollectionDelegate>) {
        self.nearby_service.get_or_download_collection(hash, collection_delegate).await.expect("downloading collection")
    }
    pub async fn set_name(&self, name: String) -> Result<(), GossipError> {
        let pk = self.identity_service.get_default_identity_pk().await?;
        let iden = self.identity_service.get_identity(pk).await?;
        let iden_to_save = match iden {
            None => {
                Identity { name, pk }
            }
            Some(mut existing) => {
                existing.name = name;
                existing
            }
        };
        self.identity_service.save_identity(&iden_to_save).await?;

        Ok(())
    }

    pub async fn set_pic(&self, pic_data: Vec<u8>) -> Result<(), GossipError> {
        let pk = self.identity_service.get_default_identity_pk().await?;
        self.identity_service.set_pic(pk, pic_data).await?;
        Ok(())
    }
}
