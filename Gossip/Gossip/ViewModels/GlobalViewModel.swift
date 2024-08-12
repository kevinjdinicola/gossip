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
    var conState: ConState = ConState.offline
    
    var identities: [NearbyProfile] = []
    var messages: [DisplayMessage] = []
    var isBroadcasting: Bool = false
    
    var docData: DocData = DocData(docId: WideId(0))
    
    func nameUpdated(name: String) async {
        self.name = name
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
    
    
    func docDataUpdated(status: DocData) async {
        self.docData = status
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
    
    func connectionStateUpdated(state: ConState) async {
        self.conState = state
    }
    
    func broadcastingUpdated(broadcasting: Bool) async {
        self.isBroadcasting =  broadcasting
    }
    
    
    
    
    
}
