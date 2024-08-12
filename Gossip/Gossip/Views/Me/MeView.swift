//
//  MeView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/24/24.
//

import SwiftUI
import PhotosUI

struct MeView: View {
    
    @State var selectedItems: [PhotosPickerItem] = []
    
    @State
    var profilePic: UIImage?
    
    @EnvironmentObject
    var global: GlobalVM;
    
    @State
    var name: String = ""
    @State
    var nameDebounceTimer: Timer?
    
    @State
    var status: String = ""
    
    var body: some View {
        List {
            Section {
                HStack(alignment: .bottom) {
                    PhotosPicker(selection: $selectedItems, maxSelectionCount: 1,
                                 matching: .images,
                                 photoLibrary: .shared()) {
                        
                        CircularImageView(uiImage: profilePic, editable: true)
                            .frame(width: 100, height: 100)
                       
                        
                    }
                    VStack {
                        TextField("Name", text: $name)
                            .font(.largeTitle)
                        TextField("Status", text: $status)
                            .font(.callout)
                            .italic()
                    }

                }
                .padding(10)

            }
        }

        .onAppear {
            name = global.name
            status = global.status.text
        }
        .onChange(of: name) {
            nameDebounceTimer?.invalidate()
            nameDebounceTimer = Timer.scheduledTimer(withTimeInterval: 1.5, repeats: false) { _ in
                Task {
                    try await GossipApp.global?.setName(name: name)
                }
            }

        }
        .onChange(of: status) {
            Task {
                await GossipApp.global?.setStatus(status: status)
            }
        }
        .onChange(of: global.status) {
            status = global.status.text
        }
        .onChange(of: global.name) {
            name = global.name
        }
        .onChange(of: global.pic) {
            Task {
                profilePic = global.pic != nil ? await ImageLoader.load(hash: global.pic!) : nil
            }

        }
        .onAppear {
            Task {
                profilePic = global.pic != nil ? await ImageLoader.load(hash: global.pic!) : nil
            }

        }
        .onChange(of: selectedItems) {
            Task {
                self.selectedItems[0].loadTransferable(type: Data.self) { result in
                    switch result {
                    case .success(let image?):
                        print("got data, emitting action")
                        Task {
                            try await GossipApp.global?.setPic(picData:image)
                        }
                        // Handle the success case with the image.
                    case .success(nil):
                        print("nada")
                        // Handle the success case with an empty value.
                    case .failure(let error):
                        // Handle the failure case with the provided error.
                        print("failed")
                    }
                }
            }
            
        }
    }
}

#Preview {
    let g = GlobalVM();
    g.name = "kevin"
    g.status = Status(text: "whats up with cats")
    ImageLoader.setLocalImage("crow", for: WideId(1))
    g.pic = WideId(1)
    
    return MeView()
        .environmentObject(g);
    
}
