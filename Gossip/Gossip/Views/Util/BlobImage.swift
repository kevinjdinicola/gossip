//
//  BlobImage.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/28/24.
//

import SwiftUI

struct BlobImage: View {
    
    var blobHash: WideId?
    
    @State
    var img: UIImage?;
    
    @StateObject
    var picData: BlobLoader = BlobLoader()
    
    func uiImageFromData(uiImage: Data?) -> UIImage? {
        uiImage.flatMap { UIImage(data: $0) }
    }
    
    var body: some View {
        Group {
            if let img = img {
                Image(uiImage: img)
                    .resizable()
            } else {
                Image(systemName: "questionmark.square.dashed")
            }
        }
        .onAppear {
            if (img == nil && blobHash != nil) {
                picData.loadHash(hash: blobHash)
            }
        }
        .onChange(of: blobHash) {
            picData.loadHash(hash: blobHash)
        }
        .onChange(of: picData.data) {
            img = uiImageFromData(uiImage: picData.data)
        }
    }
    
}

#Preview {
    BlobCache.shared.setLocalImage("crow", for: WideId(1))
    return BlobImage(blobHash: WideId(1))
}
