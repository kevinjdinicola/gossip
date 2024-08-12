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
    func shareBioUpdated(shareBio: Bool) async {
        self.shareBio = shareBio
    }
    

    // Props
    
    var initialized: Bool = false
    var shareBio: Bool = false

    var isAvailable: Bool = false
    var isEditable: Bool = false
    
    var pic: WideId?
    var name: String = ""
    var status: String = ""
    var allowsInvite: Bool = false
    
    var bioText = ""
    var galleryPics: [WideId] = []
    
    var enumeratedGalleryPics: [(String, Int, WideId)] {
        return self.galleryPics.enumerated().map{ idx,hash in
            ("\(idx)_\(hash)", idx, hash)
        }
    }
    
    // Setters
    
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
    func nameUpdated(name: String) async {
        self.name = name
    }
    
    func initializedUpdated(initialized: Bool) async {
        self.initialized = initialized
    }
    
  
}
