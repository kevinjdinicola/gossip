//
//  MessageView.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 5/10/24.
//

import SwiftUI

struct MessageView: View {
    var message: DisplayMessage
    
    @StateObject
    var payloadDel: CollectionDelegate = CollectionDelegate()

    
    var body: some View {
        VStack(alignment: message.isSelf ? .trailing : .leading, content: {

            if message.text.count > 0 {
                Text(message.text)
                    .padding(10)
                    .background(message.isSelf ? Color.blue : Color.gray)
                    .foregroundColor(Color.white)
                    .cornerRadius(10)
            }
            ForEach(payloadDel.blobs, id: \.name) { item in
                BlobImage(blobHash: item.hash)
                    .clipShape(RoundedRectangle(cornerRadius: 10))
                    .aspectRatio(contentMode: .fit)
                    .frame(maxHeight: 200)
                    .padding(5)
            }
            if (message.payload != nil && payloadDel.blobs.isEmpty) {
                HStack {
                    ProgressView().progressViewStyle(CircularProgressViewStyle())
                    Text("Downloading message")
                        .italic()
                        .font(.footnote)
                }
                .padding(10)
                .background(message.isSelf ? Color.blue : Color.gray)
                .foregroundColor(Color.white)
                .cornerRadius(10)
            }

            
        })
        .onAppear {
            if payloadDel.state == .empty {
                if let payload = message.payload {
                    Task {
                        await CollectionLoader.shared.load(collectionHash: payload, delegate: payloadDel);
                    }
                }
            }
        }
        
    }
}

#Preview {
    var cd = CollectionDelegate()
    BlobCache.shared.setLocalImage("crow", for: WideId(1))
    cd.blobs = [NamedBlob(name: "crow.png", hash: WideId(1))];
    return MessageView(message: DisplayMessage(id: 1, text: "me", isSelf: true), payloadDel: cd)
        .environment(PhotosViewVM())
}
