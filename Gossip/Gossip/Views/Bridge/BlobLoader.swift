//
//  BlobLoader.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 5/30/24.
//

import Foundation

@Observable
class BlobLoader : ObservableObject, BlobDataResponder {
    
    func isCollection() -> Bool {
        false
    }

    
    var blobHash: WideId?;
    var state: BlobDataState = BlobDataState.empty;
    
    init() {
        
    }
    
    init(blobHash: WideId) {
        self.blobHash = blobHash
        hydrate()
    }
    
    init(withData: Data) {
        self.blobHash = WideId(0)
        self.state = .loaded(self.blobHash!, withData)
    }
    
    func update(state: BlobDataState) {
        if case let .loaded(h, data) = state {
            BlobCache.shared.setData(data, for: h)
        }
        self.state = state;
    }
    
    func hash() -> WideId? {
        blobHash
    }
    
    func loadHash(hash: WideId?) {
        if let hash = hash {
            self.blobHash = hash
            if let cachedData = BlobCache.shared.getData(for: hash) {
                print("loaded cached image")
                state = .loaded(hash, cachedData)
            } else {
                print("loading from file")
                hydrate()
            }
        } else {
            self.blobHash = nil
            self.state = .empty
        }
    }
    
    private func hydrate() {
        if blobHash != nil {
            RustApp.host?.blobs().hydrate(bdr: self)
        } else {
            state = .empty;
        }
    }
    
    var data: Data? {
        if case let .loaded(_, data) = state {
            return data
        } else {
            return nil
        }
    }
    
    var isLoading: Bool {
        if case .loading = state {
            return true
        } else {
            return false
        }
    }
    
}
