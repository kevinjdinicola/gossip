//
//  BLEGossipScanner.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 7/6/24.
//

import Foundation
import CoreBluetooth


class BLEGossipScanner: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate, BleGossipScanner {

    
    private var centralManager: CBCentralManager?
    private var shouldScan: Bool
    var delegate: GossipScannerDelegateProtocol?

    
    var recent_peripherals: [CBPeripheral]
    var perpherals_under_polling: [CBPeripheral] = []
    var connectTimeoutTimer: Timer?
    
    override init() {
        recent_peripherals = []
        shouldScan = false
        super.init()
        centralManager = CBCentralManager(delegate: self, queue: nil)
    }
    
    
    func setDelegate(delegate: GossipScannerDelegate) {
        self.delegate = delegate
    }
    
    
    func startScanning() {
        print("SWIFT - start bt scanning");
        shouldScan = true
        if centralManager!.state == .poweredOn && !centralManager!.isScanning {
            print("doin a scan1")
            centralManager!.scanForPeripherals(withServices: [CBUUID.GOSSIP_SERVICE])
        }
        pollRecentPeripherals()
    }
    
    func pollRecentPeripherals() {
        perpherals_under_polling = recent_peripherals
        connectTimeoutTimer?.invalidate()
        connectTimeoutTimer = Timer.scheduledTimer(withTimeInterval: 10.0, repeats: false) { _ in
            // TODO why doesnt this timer fire
            print("couldn't find these peripherals again! removing \(self.perpherals_under_polling)")
            for removePeri in self.perpherals_under_polling {
                self.recent_peripherals.removeAll(where: {p in
                    p.identifier == removePeri .identifier
                })
            }
            self.perpherals_under_polling = []
        }
        for p in recent_peripherals {
            centralManager?.connect(p)
        }
    }
    
    func stopScanning() {
        print("SWIFT - STOP bt scanning")
        shouldScan = false
        centralManager?.stopScan()
        perpherals_under_polling = []
//        recent_peripherals = []
    }

    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        if central.state == .poweredOn && shouldScan && !central.isScanning {
            print("doin a scan2")
            central.scanForPeripherals(withServices: [CBUUID.GOSSIP_SERVICE])
        }
    }
    
    func centralManager(
        _ central: CBCentralManager,
        didDiscover peripheral: CBPeripheral,
        advertisementData: [String : Any],
        rssi RSSI: NSNumber
    ) {
        // I only want to handle peripherals im not already connected to (aka disconnected ones)
        if peripheral.state != .disconnected {
            return
        }

        // if they're disconnected and I recently talked to them, I dont want to handle them again
        // maybe wait some kind of timeout before probing again
        if recent_peripherals.contains(where: { $0.identifier == peripheral.identifier }) {
            return
        }
        peripheral.delegate = self
        recent_peripherals.append(peripheral)
        central.connect(peripheral)
    }
    
    func centralManager(
        _ central: CBCentralManager,
        didConnect peripheral: CBPeripheral
    ) {
        perpherals_under_polling.removeAll(where: {cb in
            cb.identifier == peripheral.identifier
        })
        peripheral.discoverServices([CBUUID.GOSSIP_SERVICE])
    }
    
    func peripheral(
        _ peripheral: CBPeripheral,
        didDiscoverServices error: (any Error)?
    ) {
        guard let svc = peripheral.services?.first else {
            print("failed to discover services")
            if error != nil {
                print("error \(error!.localizedDescription)")
            }
            return
        }
        // we discovered a service, now lets check its characteristics
        peripheral.discoverCharacteristics([CBUUID.ADDRESS_CHARACTERISTIC, CBUUID.DOCUMENT_CHARACTERISTIC], for: svc)
        
    }
    
    func peripheral(
        _ peripheral: CBPeripheral,
        didDiscoverCharacteristicsFor service: CBService,
        error: (any Error)?
    ) {
        guard let charcs = service.characteristics else {
            print("faild to discover charcs")
            return
        }
        
        if charcs.isEmpty || error != nil {
            print("no charcs found on svc \(service.uuid), error: \(error.debugDescription)")
            return
        }
        
        var missingValues = false
        // read all characteristics we discovered (if missing values)
        for chr in charcs {
            if chr.value == nil {
                missingValues = true
                peripheral.readValue(for: chr)
            }
        }
        // somehow all values were present when discovering characteristics
        if !missingValues {
            checkPeripheralCompletion(peripheral: peripheral)
        }
    }
    
    func peripheral(
        _ peripheral: CBPeripheral,
        didUpdateValueFor characteristic: CBCharacteristic,
        error: (any Error)?
    ) {
        if error != nil {
            print("error reading value! \(error!.localizedDescription)")
            return
        }
        checkPeripheralCompletion(peripheral: peripheral)
    }
    
    func checkPeripheralCompletion(peripheral: CBPeripheral) {
        guard let svc = peripheral.services?.first else { return }
        guard let characteristics = svc.characteristics else { return }
        
        var addressData: Data?
        var documentData: Data?
        var peerState: UInt8?
        
        for charc in characteristics {
            switch charc.uuid {
            case CBUUID.ADDRESS_CHARACTERISTIC:
                addressData = charc.value
            case CBUUID.DOCUMENT_CHARACTERISTIC:
                guard let data = charc.value else { continue }
                documentData = data.subdata(in: 1..<data.count)
                peerState = data[0]
                
                if peerState == 0 {
                    // we discovered a peer that is undecided - if they become decided, i want to hop onto that document, so lets subscribe to updates
                    // on their document characteristic because thats where peerstate is bundled (with document)
                    peripheral.setNotifyValue(true, for: charc)
                } else {
                    peripheral.setNotifyValue(false, for: charc)
                }
            default:
                print("unexpected characteristic found \(charc.uuid)")
            }
        }

        if addressData != nil && documentData != nil && peerState != nil {
            peripheralReady(peripheral: peripheral, addressData: addressData!, documentData: documentData!, peerState: peerState!)
        }
        
    }
    
    func peripheral(
        _ peripheral: CBPeripheral,
        didUpdateNotificationStateFor characteristic: CBCharacteristic,
        error: (any Error)?
    ) {
        guard let error = error else {return}
        print(error)
    }
    
    func peripheralReady(peripheral: CBPeripheral, addressData: Data, documentData: Data, peerState: UInt8) {
        
        delegate?.peerDataDiscovered(uuid: peripheral.identifier.toHostUuid(), addressData: addressData, documentData: documentData, peerState: peerState)
    }

    
    
}
