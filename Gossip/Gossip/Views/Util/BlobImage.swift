//
//  BlobImage.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/7/24.
//

import SwiftUI

struct BlobImage: View {
    
    @State
    var hash: WideId
    
    @State
    var image: UIImage?
    
    var body: some View {
        Group {
            if let image = image {
                Image(uiImage: image)
                    .resizable()
            }
        }
        .onAppear {
            Task {
                image = await ImageLoader.load(hash: hash)
            }
        }
        .onChange(of: hash) { oldVal, newVal in
            Task {
                image = await ImageLoader.load(hash: newVal)
            }
        }
    }
}

#Preview {
    BlobImage(hash: WideId(1))
}
