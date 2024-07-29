//
//  DeviceApiProvider.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/20/24.
//

import Foundation

class DeviceApiProvider: DeviceApiServiceProvider {
    func bleScanner() -> any BleGossipScanner {
        return BLEGossipScanner()
    }
    
    func bleBroadcaster() -> any BleGossipBroadcaster {
        return BLEGossipBroadcaster()
    }
    
    
}
