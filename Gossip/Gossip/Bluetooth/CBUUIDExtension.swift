//
//  CBUUIDExtension.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 7/6/24.
//

import Foundation
import CoreBluetooth


extension CBUUID {
    static let GOSSIP_SERVICE = CBUUID(string: "abf872e9-edba-47ad-acbd-e59167f081aa");
    static let DOCUMENT_CHARACTERISTIC = CBUUID(string: "c8bbbd7c-c2fe-4641-b01a-e1f80fc5e768")
    static let ADDRESS_CHARACTERISTIC = CBUUID(string: "98675d38-f549-4e42-8bb5-4cf66fdb28a4")
}
