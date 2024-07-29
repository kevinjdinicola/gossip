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
        VStack(alignment: .leading, content: {
            
//            !message.isSelf ? Text(message.sender + ":")
//                .bold()
//                .foregroundColor(Color.gray)
//            : nil
            if message.text.count > 0 {
                Text(message.text)
                    .padding(10)
                    .background(message.isSelf ? Color.gray : Color.blue)
                    .foregroundColor(Color.white)
                    .cornerRadius(10)
            }
            ForEach(payloadDel.blobs, id: \.name) { item in
                VStack(alignment: .trailing) {

                    BlobImage(blobHash: item.hash)
                        .padding(5)
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 200)
                        .cornerRadius(10)
                    
                }
            }
            if (message.payload != nil && payloadDel.blobs.isEmpty) {
                Text("empty blobs! but real payload")
            }

            
        })
        .onAppear {
            if payloadDel.state == .empty {
                if let payload = message.payload {
                    Task {
                        await GossipApp.global?.loadNearbyPayload(hash: payload, collectionDelegate: payloadDel);
                    }
                }
            }
        }
        
    }
}

#Preview {
    MessageView(message: DisplayMessage(id: 1, text: "me", isSelf: true))
}
