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
        
    @EnvironmentObject
    var photoViewModal: PhotosViewVM
    
    @StateObject
    var picData: BlobLoader = BlobLoader()
    
    func uiImageFromData(uiImage: Data?) -> UIImage? {
        uiImage.flatMap { UIImage(data: $0) }
    }
    
    func imageView() -> Image {
        if let img = img {
            Image(uiImage: img)
                .resizable()
        } else {
            Image(systemName: "questionmark.square.dashed")
        }
    }
    
    var body: some View {
        Group {
            
            imageView()
            .onTapGesture(count: 1, perform: {
                if blobHash != nil {
                    print("my hash is \(wideidToString(wideId: blobHash!))")
                    photoViewModal.index = 0
                    photoViewModal.images = [img!]
                    photoViewModal.isShowing = true
                } else {
                    print("no blobhash here...")
                }
                
            })
                
        }
        .onAppear {
            if (img == nil && blobHash != nil) {
                Task {
                    await picData.loadHash(hash: blobHash)
                }
            }
        }
        .onChange(of: blobHash) {
            Task {
                await picData.loadHash(hash: blobHash)
            }
        }
        .onChange(of: picData.data) {
            img = uiImageFromData(uiImage: picData.data)
        }
    }
    
}

#Preview {
    BlobCache.shared.setLocalImage("crow", for: WideId(1))
    return BlobImage(blobHash: WideId(1))
        .aspectRatio(contentMode: .fit)
        .clipShape(RoundedRectangle(cornerRadius: 10))
}
