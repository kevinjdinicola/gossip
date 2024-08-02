use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::ble::PeerState::{Scanning, Settled};
use crate::data::UUID;

#[derive(uniffi::Object)]
pub struct GossipScannerDelegate(pub Sender<(UUID, PeerData)>);


#[uniffi::export]
impl GossipScannerDelegate {
    fn peer_data_discovered(&self, uuid: UUID, address_data: AddressData, document_data: DocumentData, peer_state: u8) {
        self.0.try_send((uuid, PeerData { address_data, document_data, peer_state: peer_state.into() }))
            .expect("failed to send peer discovery data")
    }
}

pub type BluetoothPeerEvent = (UUID, PeerData);

pub type AddressData = Vec<u8>;
pub type DocumentData = Vec<u8>;
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PeerData {
    pub address_data: AddressData,
    pub document_data: DocumentData,
    pub peer_state: PeerState,
}

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PeerState {
    Scanning = 0,
    Settled = 1,
}
impl From<u8> for PeerState {
    fn from(value: u8) -> Self {
        match value {
            1 => Settled,
            _ => Scanning,
        }
    }
}
impl From<PeerState> for u8 {
    fn from(value: PeerState) -> Self {
        match value {
            Settled => 1,
            _ => 0,
        }
    }
}


#[uniffi::export(with_foreign)]
pub trait BLEGossipScanner: Send + Sync {
    fn start_scanning(&self);
    fn stop_scanning(&self);
    fn set_delegate(&self, delegate: Arc<GossipScannerDelegate>);
}

#[uniffi::export(with_foreign)]
pub trait BLEGossipBroadcaster: Send + Sync {
    fn start(&self);
    fn stop(&self);
    fn set_document_data(&self, document_data: Vec<u8>);
    fn set_address_data(&self, address_data: Vec<u8>);
    fn set_peer_state(&self, peer_state: u8);
}
