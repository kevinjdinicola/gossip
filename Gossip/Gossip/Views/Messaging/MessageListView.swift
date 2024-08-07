//
//  MessageListView.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 7/7/24.
//

import SwiftUI
import PhotosUI
import CryptoKit

struct MessageListView: View {
    
    var messages: [DisplayMessage]
    
    @State var selectedPhotoPictureItems: [PhotosPickerItem] = []
    
    
    @Binding var composingMessage: String
    @Binding var attachments: [(String, Data)]
    var sendPressed: (() -> Void)?
    
    @FocusState private var isComposingFieldFocused: Bool
    
    func scrollToBottom(svp: ScrollViewProxy) {
        if (messages.count > 0) {
            svp.scrollTo(messages[messages.count-1].id, anchor: .bottom)
        }
    }
    
    func hashImageData(_ imageData: Data) -> String {
        let hash = SHA256.hash(data: imageData)
        return hash.map { String(format: "%02hhx", $0) }.joined()
    }
    
    func loadImages(from items: [PhotosPickerItem]) {
        var indexStartOffset = attachments.count
        for (index, item) in items.enumerated() {
            item.loadTransferable(type: Data.self) { result in
                switch result {
                case .success(let image):
                    if let image = image {
                        DispatchQueue.main.async {
                            self.attachments.append((hashImageData(image),image))
                        }
                    }
                case .failure(let error):
                    print("Error loading image: \(error.localizedDescription)")
                }
            }
        }
    }
    
    
    var body: some View {
        VStack {
            ScrollView {
                ScrollViewReader(content: { proxy in
                    VStack(alignment: .leading) {
                        ForEach(messages, id: \.id) { message in
                            HStack {
                                message.isSelf ? Spacer() : nil
                                MessageView(message: message)
                                !message.isSelf ? Spacer() : nil
                            }
                        }
                        .onChange(of: messages) {
                            withAnimation(.easeInOut(duration: 0.3)) {
                                scrollToBottom(svp: proxy)
                            }
                            
                        }
                        .onAppear(perform: {
                            scrollToBottom(svp: proxy)
                        })
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(10)
                    .onChange(of: isComposingFieldFocused) { _, focused in
                        if focused {
                            DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) { // 2 seconds delay
                                //so hacky, do this after the keyboard shows
                                withAnimation(.easeInOut(duration: 0.3)) {
                                    scrollToBottom(svp: proxy)
                                }
                            }
                        }
                    }
                    
                })
                .frame(maxWidth: .infinity)
            }
            .frame(maxWidth: .infinity)
            VStack(alignment: .leading) {
                HStack {
                    ForEach(attachments, id: \.0) { item in
                        ZStack(alignment: .topLeading) {
                            
                            Image(uiImage: UIImage(data: item.1)!)
                                .resizable()
                                .aspectRatio(contentMode: .fit)
                                .frame(maxHeight: 100)
                                .cornerRadius(10)
                            Button(action: {
                                for (i,a) in attachments.enumerated() {
                                    if a.0 == item.0 {
                                        attachments.remove(at: i)
                                        return
                                    }
                                }
                                
                            }) {
                                Image(systemName: "x.circle.fill")
                                    .resizable()
                                    .aspectRatio(1, contentMode: .fit)
                                    .frame(maxHeight: 20)
                                    .foregroundColor(.black)
                                    .background(Circle().foregroundColor(.white).frame(width: 22, height: 22))
                                    .padding(5)
                            }
                        }
                    }
                }
                
                HStack {
                    TextField("Message", text: $composingMessage)
                        .focused($isComposingFieldFocused)
                    PhotosPicker(selection: $selectedPhotoPictureItems, maxSelectionCount: 10,
                                 matching: .images,
                                 photoLibrary: .shared()) {
                        Image(systemName: "plus.circle.fill")
                            .resizable()
                            .aspectRatio(1, contentMode: .fit)
                            .frame(maxHeight: 30)
                            .foregroundColor(.secondary)
                    }
                                 .onChange(of: selectedPhotoPictureItems) {
                                     loadImages(from: selectedPhotoPictureItems)
                                 }
                    
                    Button(action: {
                        sendPressed?()
                    }, label: {
                        Image(systemName: "arrow.up.circle.fill")
                            .resizable()
                            .aspectRatio(1, contentMode: .fit)
                            .frame(maxHeight: 30)
                    })
                }
            }
            
            .padding(10)
            .overlay(
                RoundedRectangle(cornerRadius: 20)
                    .stroke(Color.gray, lineWidth: 1)
            )
            .padding(.horizontal, 10)
            .padding(.bottom, 5)
            
        }
        
    }
}

#Preview {
    let images: [Image] = [Image("crow")]
    let msgs = [
        DisplayMessage(id: 0, text: "hi", isSelf: true),
        DisplayMessage(id: 1, text: "bye", isSelf: false),
    ]
    return MessageListView(messages: msgs, composingMessage: Binding.constant(""), attachments: Binding.constant([])) {
        
    }
}
