//
//  Global.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/20/24.
//

import Foundation

@Observable
@MainActor
class GlobalVM: GlobalViewModel, ObservableObject {
    
    var ownPk: WideId = WideId(0)
    var name: String = ""
    var status: Status = Status(text: "")
    var pic: WideId?
    
    var identities: [NearbyProfile] = []
    var messages: [DisplayMessage] = []
    var isScanning: Bool = false
    
    var debugState: DebugState = DebugState(docId: "", foundGroup: false)
    
    func nameUpdated(name: String) async {
        self.name = name
    }

    func scanningUpdated(scanning: Bool) async {
        self.isScanning = scanning
    }
    
    func nearbyProfilesUpdated(profiles: [NearbyProfile]) async {
        self.identities = profiles
    }
    
    func statusUpdated(status: Status) async {
        self.status = status
    }
    
    func picUpdated(pic: WideId) async {
        self.pic = pic
    }
    
    func debugStateUpdated(status: DebugState) async {
        self.debugState = status
    }
    
    func allMessagesUpdated(messages: [DisplayMessage]) async {
        self.messages = messages;
    }
    
    func receivedOneMessage(message: DisplayMessage) async {
        self.messages.append(message);
    }
    
    func ownPublicKeyUpdated(pk: WideId) async {
        self.ownPk = pk
    }
    
    
    
}
