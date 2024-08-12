//
//  BlobImage.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/28/24.
//

import SwiftUI

struct MessagingImage: View {
    
    var blobHash: WideId?
    
    @State
    var img: UIImage?;
    
    @EnvironmentObject
    var photoViewModal: PhotosViewVM
    
    func loadImage() async {

        if self.img != nil {
            return
        }

        self.img = blobHash != nil ? await ImageLoader.load(hash: blobHash!) : nil
    }
    
    var body: some View {
        Group {
            if let img = img {
                Image(uiImage: img)
                    .resizable()
                    .onTapGesture(count: 1, perform: {
                        if blobHash != nil {
                            photoViewModal.index = 0
                            photoViewModal.images = [img]
                            photoViewModal.isShowing = true
                        } else {
                            print("no blobhash here...")
                        }
                        
                    })
            } else {
                Text("")
            }
        }
        .onAppear {

            Task {
                await loadImage()
            }
        }
    }
    
    
}

#Preview {
    BlobCache.shared.setLocalImage("crow", for: WideId(1))
    return MessagingImage(blobHash: WideId(1))
        .aspectRatio(contentMode: .fit)
        .clipShape(RoundedRectangle(cornerRadius: 10))
}
