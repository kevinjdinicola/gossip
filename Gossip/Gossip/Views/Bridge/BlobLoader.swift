//
//  BlobLoader.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 5/30/24.
//

import Foundation

@Observable
@MainActor
class BlobLoader : ObservableObject, BlobDataResponder {
    
    var blobHash: WideId?;
    var state: BlobDataState = BlobDataState.empty;
    
    init() {
        
    }
    
    init(blobHash: WideId) {
        self.blobHash = blobHash
        Task {
            await hydrate()
        }
    }
    
    init(withData: Data) {
        self.blobHash = WideId(0)
        self.state = .loaded(self.blobHash!, withData)
    }
    
    func update(state: BlobDataState) async {
        if case let .loaded(h, data) = state {
            print("loaded data for \(wideidToString(wideId: h)) is size \(data.count)")
            BlobCache.shared.setData(data, for: h)
        }
        self.state = state;
    }
    
    func hash() async -> WideId? {
        blobHash
    }
    
    func loadHash(hash: WideId?) async {
        if let hash = hash {
            self.blobHash = hash
            if let cachedData = BlobCache.shared.getData(for: hash) {
                print("cached data for \(wideidToString(wideId: hash)) is size \(cachedData.count)")
                state = .loaded(hash, cachedData)
            } else {
                print("loading from file")
                await hydrate()
            }
        } else {
            self.blobHash = nil
            self.state = .empty
        }
    }
    
    private func hydrate() async {
        if blobHash != nil {
            await RustApp.host?.blobs().hydrate(bdr: self)
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
