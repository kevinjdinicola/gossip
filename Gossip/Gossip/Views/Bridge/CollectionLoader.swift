//
//  CollectionLoader.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/31/24.
//

import Foundation

class CollectionLoader {
    private var cache: [String: [NamedBlob]] = [:]
    private init() {}
    
    static let shared = CollectionLoader()
    
    func load(collectionHash: WideId?, delegate: CollectionDelegate) async {
        delegate.hash = collectionHash
        if let collectionHash = collectionHash {

            if let cachedList = CollectionLoader.shared.getData(for: collectionHash) {
                print("loaded collection details from cache")
                delegate.blobs = cachedList
                delegate.state = .loaded(cachedList)
            } else {
                print("loaded collection details from thing")
                Task {
                    await GossipApp.global?.loadNearbyPayload(hash: collectionHash, collectionDelegate: delegate);
                }
            }
        } else {
            delegate.state = .empty
            delegate.blobs = []
        }
    }
    
    func setData(_ data: [NamedBlob], for wideId: WideId) {
        cache[wideidToString(wideId: wideId)] = data
    }

    func getData(for wideId: WideId) -> [NamedBlob]? {
        return cache[wideidToString(wideId: wideId)]
    }
    
    func removeData(for wideId: WideId) {
        cache.removeValue(forKey: wideidToString(wideId: wideId))
    }
    
    func clearCache() {
        cache.removeAll()
    }
}
