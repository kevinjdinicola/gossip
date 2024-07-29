//
//  BLEGossipBroadcaster.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 7/6/24.
//

import Foundation
import CoreBluetooth


class BLEGossipBroadcaster: NSObject, CBPeripheralManagerDelegate, BleGossipBroadcaster {
    
    private var peripheralManager: CBPeripheralManager?
    private var gossipService: CBMutableService?
    private var documentCharc: CBMutableCharacteristic?
    private var addressCharc: CBMutableCharacteristic?

    private var shouldBroadcast = false
    
    private var documentData: Data
    private var addressData: Data
    private var peerState: UInt8
    
    private var subscribers: [CBCentral] = []
    
    
    override init() {
        documentData = Data()
        addressData = Data()
        peerState = 0 // zero will mean still scanning/not in a dedicated cluster
        super.init()
        peripheralManager = CBPeripheralManager(delegate: self, queue: nil)

    }
    
    func start() {
        print(documentData.base64EncodedString())
        print(addressData.base64EncodedString())
        shouldBroadcast = true
        broadcast_characteristics()
    }
    
    func stop() {
        shouldBroadcast = false
        peripheralManager?.stopAdvertising()
    }
    
    func setDocumentData(documentData: Data) {
        self.documentData = documentData
        broadcastDocCharcUpdateToSubscribers()
    }
    
    func setAddressData(addressData: Data) {
        self.addressData = addressData
        // todo if someone is listening here i should trigger an update
    }
    
    func setPeerState(peerState: UInt8) {
        self.peerState = peerState
        broadcastDocCharcUpdateToSubscribers()
    }
    
    func broadcastDocCharcUpdateToSubscribers() {
        guard let charc = documentCharc else { return }
        peripheralManager?.updateValue(calcDocCharcData(), for: charc, onSubscribedCentrals: subscribers)
    }


    func broadcast_characteristics() {
        // if we're not powered on OR we're already advertising, no point in doing it
        if peripheralManager!.state != .poweredOn || peripheralManager!.isAdvertising {
            return
        }
        
        register_characteristics()
        
        let advertisementData: [String: Any] = [
            CBAdvertisementDataServiceUUIDsKey: [CBUUID.GOSSIP_SERVICE]
        ]
        peripheralManager!.startAdvertising(advertisementData)
        print("advertising")
    }
    
    func register_characteristics() {
        if gossipService != nil {
            return
        }
        documentCharc = CBMutableCharacteristic(
            type: CBUUID.DOCUMENT_CHARACTERISTIC,
            properties: [.read, .notify],
            value: nil,
            permissions: [.readable]
        )
        addressCharc = CBMutableCharacteristic(
            type: CBUUID.ADDRESS_CHARACTERISTIC,
            properties: [.read, .notify],
            value: nil,
            permissions: [.readable]
        )

        gossipService = CBMutableService(type: CBUUID.GOSSIP_SERVICE, primary: true)
        gossipService?.characteristics = [documentCharc!, addressCharc!]

        peripheralManager?.add(gossipService!)
    }
    
    func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) {
        if peripheral.state == .poweredOn {
            if shouldBroadcast {
                broadcast_characteristics()
            } else {
                peripheral.stopAdvertising()
            }
        }
    }
    
    func isBroadcasting() -> Bool {
        shouldBroadcast && peripheralManager?.state == .poweredOn
    }
    
    func calcDocCharcData() -> Data {
        Data([peerState]) + documentData
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveRead request: CBATTRequest) {
        var charcData: Data?
        
        switch request.characteristic.uuid {
        case documentCharc?.uuid:
            charcData = calcDocCharcData()
        case addressCharc?.uuid:
            charcData = addressData
        default:
            charcData = nil
        }
        
        guard let charcData = charcData else {
            peripheral.respond(to: request, withResult: .attributeNotFound)
            return
        }
        
        if request.offset > 0 {
            print("oh fuck the offset wasnt zero fix this shit")
        }
        request.value = charcData
        peripheral.respond(to: request, withResult: .success)
        
    }
    
    // subscriptions
    func peripheralManager(
        _ peripheral: CBPeripheralManager,
        central: CBCentral,
        didSubscribeTo characteristic: CBCharacteristic
    ) {
        // i only care about subscriptions to document
        print("got a subscriber!, it takes updates of size \(central.maximumUpdateValueLength)")
        
        subscribers.append(central)
    }
    
    func peripheralManager(
        _ peripheral: CBPeripheralManager,
        central: CBCentral,
        didUnsubscribeFrom characteristic: CBCharacteristic
    ) {
        print("lost a subscriber!")
        subscribers.removeAll(where: { $0.identifier == central.identifier})
    }
    
    func peripheralManagerIsReady(toUpdateSubscribers peripheral: CBPeripheralManager) {
        print("Error must have occured updating, because we're getting a call to retransmit, doing that...")
        broadcastDocCharcUpdateToSubscribers()
    }
    
    
}
