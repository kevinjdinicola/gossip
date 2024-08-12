//
//  ImageLoader.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/7/24.
//

import Foundation
import UIKit

class ImageLoader {
    
    private static var cache: [WideId: UIImage] = [:]
    
    public static func setLocalImage(_ name: String, for wideId: WideId) {
        cache[wideId] = UIImage(named: name)!
    }
    
    static func getData(for wideId: WideId) -> UIImage? {
        return cache[wideId]
    }
    
    static func removeData(for wideId: WideId) {
        cache.removeValue(forKey: wideId)
    }
    
    static func setData(_ data: UIImage, for wideId: WideId) {
        print("💥 WILL I DIE setting \(wideId) for some UIImage data")
        cache[wideId] = data
    
    }
    
    
    public static func load(hash: WideId) async  -> UIImage {
        var image = getData(for: hash)
        if image == nil {
            do {
                let imgData = try await RustApp.host?.loadBlob(hash: hash)
                if imgData != nil {
                    image = UIImage(data: imgData!)
                    setData(image!, for: hash)
                } else {
                    image = UIImage()
                }

            } catch {
                print("fuck!")
            }
        }
        return image!
    }
}
