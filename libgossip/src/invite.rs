use std::ops::Deref;
use std::sync::Arc;
use anyhow::Error;
use async_trait::async_trait;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::RwLock;
use crate::device::DeviceApiServiceProvider;
use crate::doc::Node;
use crate::events::{create_broadcast, start_with, Starter};
use crate::identity::IdentityService;
use crate::settings::SettingsService;
pub use self::Service as InviteService;


#[derive(Clone)]
pub struct Service(Arc<InnerService>);

impl Deref for Service {
    type Target = InnerService;
    fn deref(&self) -> &Self::Target { self.0.as_ref() }
}

#[derive(Clone, Debug)]
pub enum InviteServiceEvents {

}

pub struct State {

}

pub struct InnerService {
    bc: Sender<InviteServiceEvents>,
    identity_service: IdentityService,
    settings_service: SettingsService,
    state: RwLock<State>,
}



#[async_trait]
impl Starter for Service {
    async fn start(&self) -> Result<(), Error> {
        todo!()
    }
}

impl Service {
    pub fn new(node: Node, identity_service: IdentityService, settings_service: SettingsService) -> Self {
        start_with(Service(Arc::new(InnerService {
            bc: create_broadcast(),
            identity_service,
            settings_service,
            state: RwLock::new(State {})
        })))
    }

    pub fn subscribe(&self) -> Receiver<InviteServiceEvents> {
        self.bc.subscribe()
    }
}
