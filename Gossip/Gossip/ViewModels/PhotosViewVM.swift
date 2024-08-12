//
//  PhotosViewVM.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/2/24.
//

import Foundation
import SwiftUI

@Observable
@MainActor
class PhotosViewVM: ObservableObject {
    
    var isShowing: Bool = false
    var index: Int = 0
    var images: [UIImage] = []
    
    public func setImagesFromHashes(hashes: [WideId]) async {
        self.images = hashes.map { hash in
            if let img = ImageLoader.getData(for: hash) {
                return img
            } else {
                return UIImage()
            }
        }
    }
}
