//
//  NearbyPersonDetailsVM.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/31/24.
//
import Foundation
import SwiftUI

@Observable
@MainActor
class NearbyPersonDetailsVM: ObservableObject, NearbyDetailsViewModel {
   

    var isAvailable: Bool = false
    var isEditable: Bool = false
    
    var pic: WideId?
    var name: String = ""
    var status: String = ""
    var allowsInvite: Bool = false
    
    var bioText = ""
    var galleryPics: [NamedBlob] = []
    
    func nameUpdated(name: String) async {
        self.name = name
    }
    
    func statusUpdate(status: Status) async {
        self.status = status.text
    }
    
    func picUpdated(pic: WideId?) async {
        self.pic = pic
    }
    
    func availabilityUpdated(available: Bool) async {
        self.isAvailable = available
    }
    
    func editableUpdated(editable: Bool) async {
        self.isEditable = editable
    }
    
    func bioDetailsUpdated(details: BioDetails) async {
        self.isEditable = details.editable
        self.bioText = details.text
        self.galleryPics = details.pics
    }
  
}
