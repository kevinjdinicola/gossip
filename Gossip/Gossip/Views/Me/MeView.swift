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
    @StateObject
    var picData: BlobLoader = BlobLoader();
    
    @EnvironmentObject
    var global: GlobalVM;
    
    @State
    var name: String = ""
    @State
    var status: String = ""
    
    var body: some View {
        List {
            Section {
                HStack(alignment: .bottom) {
                    PhotosPicker(selection: $selectedItems, maxSelectionCount: 1,
                                 matching: .images,
                                 photoLibrary: .shared()) {
                        
                        CircularImageView(data: picData.data, editable: true)
                            .frame(width: 100, height: 100)
                            .padding(.top, 100)
                        
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
            Task {
                try await GossipApp.global?.setName(name: name)
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
            if let picHash = global.pic {
                picData.loadHash(hash: picHash)
            } else {
                picData.state = .empty
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
    
    return MeView()
        .environmentObject(g);
    
}
