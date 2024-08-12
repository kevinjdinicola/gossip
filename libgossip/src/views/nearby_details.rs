use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::Error;
use async_trait::async_trait;
use tokio::sync::Notify;
use crate::blob_dispatcher::{LoadCollectionDelegate, NamedBlob};
use crate::data::{BlobHash, PublicKey};
use crate::events::{start_with, Starter};
use crate::identity::IdentityService;
use crate::identity::model::Identity;
use crate::nearby::model::{BioDetails, NearbyProfile, Status};
use crate::nearby::{NearbyService, NearbyServiceEvents};
use crate::settings::{SettingsEvent, SettingsService, SHARE_NEARBY_PUBLIC_BIO};
use crate::views::errors::GossipError;
use crate::views::node_stat::NodeStat;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait NearbyDetailsViewModel: Send + Sync + 'static {
    async fn name_updated(&self, name: String);

    async fn status_update(&self, status: Status);

    async fn pic_updated(&self, pic: Option<BlobHash>);

    async fn editable_updated(&self, editable: bool);

    async fn availability_updated(&self, available: bool);

    async fn bio_details_updated(&self, details: BioDetails);

    async fn share_bio_updated(&self, share_bio: bool);

    async fn initialized_updated(&self, initialized: bool);
}

#[derive(uniffi::Object, Clone)]
pub struct NearbyDetailsViewController {
    subject_pk: PublicKey,
    nearby_service: NearbyService,
    identity_service: IdentityService,
    settings_service: SettingsService,
    view_model: Arc<dyn NearbyDetailsViewModel>,
    stop: Arc<Notify>
}

impl NearbyDetailsViewController {
    pub fn new(subject_pk: PublicKey, view_model: Arc<dyn NearbyDetailsViewModel>, identity_service: IdentityService, nearby_service: NearbyService, settings_service: SettingsService) -> Arc<Self> {
        Arc::new(start_with(NearbyDetailsViewController {
            subject_pk,
            nearby_service,
            identity_service,
            settings_service,
            view_model,
            stop: Arc::new(Notify::new())
        }))
    }
}

impl Drop for NearbyDetailsViewController {
    fn drop(&mut self) {
        self.stop.notify_waiters();
    }
}
#[async_trait]
impl Starter for NearbyDetailsViewController {
    async fn start(&self) -> Result<(), Error> {
        self.view_model.initialized_updated(false).await;
        self.push_data_for_profile(&self.subject_pk).await?;
        self.view_model.initialized_updated(true).await;
        self.listen().await;
        Ok(())
    }
}

type UIResponse = Result<(), GossipError>;

#[uniffi::export(async_runtime = "tokio")]
impl NearbyDetailsViewController {
    pub async fn set_bio_text(&self, text: String) -> UIResponse {
        self.nearby_service.set_bio_text(text).await?;
        Ok(())
    }

    pub async fn set_share_bio(&self, should_share: bool) -> UIResponse {
        self.settings_service.set_setting(SHARE_NEARBY_PUBLIC_BIO, should_share).await?;
        Ok(())
    }

    pub async fn set_gallery_pic(&self, pics: Vec<BlobHash>) ->  UIResponse {
        self.nearby_service.set_gallery_pics(pics).await?;
        Ok(())
    }
}

impl NearbyDetailsViewController {

    async fn push_data_for_profile(&self, pk: &PublicKey) -> anyhow::Result<()> {
        let p = self.nearby_service.get_profile_by_key(pk).await?;
        let me = self.identity_service.get_default_identity_pk().await?;
        let share_it = self.settings_service.get_setting_defaulted(SHARE_NEARBY_PUBLIC_BIO, || false).await?;
        self.view_model.name_updated(p.name).await;
        self.view_model.pic_updated(p.pic).await;
        self.view_model.status_update(p.status).await;
        self.view_model.editable_updated(&me == pk).await;
        self.view_model.share_bio_updated(share_it).await;

        let bio = self.nearby_service.get_bio(&pk).await?;
        let bio= match bio {
            None => {
                self.view_model.availability_updated(false).await;
                return Ok(())
            }
            Some(bio) => {
                self.view_model.availability_updated(true).await;
                bio
            }
        };

        self.view_model.bio_details_updated(bio).await;
        Ok(())
    }
    async fn listen(&self) {
        let mut nearby_sub = self.nearby_service.subscribe();
        let mut setting_sub = self.settings_service.subscribe();
        let me = self.identity_service.get_default_identity_pk().await.expect("getting my own identity");

        let mut listen = true;
        while listen {
            tokio::select! {
                Ok(e) = setting_sub.recv() => {
                    match e {
                        SettingsEvent::OwnPublicBioUpdated(_) => {
                            if me == self.subject_pk {
                                if let Err(e) = self.push_data_for_profile(&self.subject_pk).await {
                                    eprintln!("failed to push bio data {e}")
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(e) = nearby_sub.recv() => {
                    match e {
                        NearbyServiceEvents::BioUpdated(pk) => {
                            if self.subject_pk != pk {
                                break
                            }
                            if let Err(e) = self.push_data_for_profile(&self.subject_pk).await {
                                eprintln!("failed to push bio data {e}")
                            }
                        }
                        _ => {}
                    }
                }
                _ = self.stop.notified() => {
                    listen = false;
                }
            }
        }

    }

}

