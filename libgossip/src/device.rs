use std::sync::Arc;

use crate::ble::{BLEGossipBroadcaster, BLEGossipScanner, GossipScannerDelegate};

#[uniffi::export(with_foreign)]
pub trait DeviceApiServiceProvider: Send + Sync {
    fn ble_scanner(&self) -> Arc<dyn BLEGossipScanner>;
    fn ble_broadcaster(&self) -> Arc<dyn BLEGossipBroadcaster>;
}


#[uniffi::export]
pub fn create_dummy_provider() -> Arc<dyn DeviceApiServiceProvider> {
    Arc::new(DummyApiServiceProvider { })
}
pub struct DummyApiServiceProvider;
impl DeviceApiServiceProvider for DummyApiServiceProvider {
    fn ble_scanner(&self) -> Arc<dyn BLEGossipScanner> {
        Arc::new(DummyBLEScanner{})
    }

    fn ble_broadcaster(&self) -> Arc<dyn BLEGossipBroadcaster> {
        Arc::new(DummyBLEBroadcaster{})
    }
}

struct DummyBLEScanner;
struct DummyBLEBroadcaster;
impl BLEGossipScanner for DummyBLEScanner {
    fn start_scanning(&self) {

    }

    fn stop_scanning(&self) {

    }

    fn set_delegate(&self, _delegate: Arc<GossipScannerDelegate>) {

    }
}
impl BLEGossipBroadcaster for DummyBLEBroadcaster {
    fn start(&self) {

    }

    fn stop(&self) {

    }

    fn set_document_data(&self, _document_data: Vec<u8>) {

    }

    fn set_address_data(&self, _address_data: Vec<u8>) {

    }

    fn set_peer_state(&self, _peer_state: u8) {

    }
}