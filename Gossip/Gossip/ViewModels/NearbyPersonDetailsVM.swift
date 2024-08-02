//
//  NearbyPersonDetailsVM.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/31/24.
//
import Foundation

@Observable
@MainActor
class NearbyPersonDetailsVM: ObservableObject {
    var pic: WideId?
    var name: String = ""
    var allowsInvite: Bool = false
    
    var moreText = ""
    var galleryPics: [WideId] = []
    
    func setPic(hash: WideId) async {
//        pic = hash
//        BlobLoader(blobHash: <#T##WideId#>)
    }
  
}
