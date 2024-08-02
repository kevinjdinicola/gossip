//
//  CollectionDelegate.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/28/24.
//

import Foundation

@Observable
class CollectionDelegate: ObservableObject, LoadCollectionDelegate {
    
    var state: CollectionState = .empty
    var blobs: [NamedBlob] = []
    var hash: WideId?
    
    func update(state: CollectionState) async {
        self.state = state;
        if case let .loaded(data) = state {
            CollectionLoader.shared.setData(data, for: hash!)
            blobs = data
        }
    }
    
    
}
