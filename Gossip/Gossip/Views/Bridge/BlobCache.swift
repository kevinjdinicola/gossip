//
//  BlobCache.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 5/31/24.
//

import Foundation
import UIKit

class BlobCache {
    private var cache: [WideId: Data] = [:]
    private init() {}
    
    static let shared = BlobCache()
    
    func setData(_ data: Data, for wideId: WideId) {
        cache[wideId] = data
    }
    
    func setLocalImage(_ name: String, for wideId: WideId) {
        cache[wideId] = UIImage(named: name)!.pngData()!
    }
    
    func getData(for wideId: WideId) -> Data? {
        return cache[wideId]
    }
    
    func removeData(for wideId: WideId) {
        cache.removeValue(forKey: wideId)
    }
    
    func clearCache() {
        cache.removeAll()
    }
}
