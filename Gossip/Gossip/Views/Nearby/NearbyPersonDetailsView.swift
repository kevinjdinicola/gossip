//
//  NearbyPersonDetailsView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/24/24.
//

import SwiftUI

struct NearbyPersonDetailsView: View {
    
    @State
    var status: String = "";
    @State
    var shareBio: Bool = false;
    @State
    var bioText: String = "xPassionate traveler and foodie exploring the world one bite at a time. Tech enthusiast with a love for coding and innovation. Avid reader, coffee addict, and amateur photographer. Living life to the fullest! \n\nClick here! \nhttp://www.google.com/";
    
    @StateObject
    var viewModel = NearbyPersonDetailsVM()
    
    @StateObject
    var blobLoader = BlobLoader()
    
    let items = Array(1...20).map { "Item \($0)" }
    
    var body: some View {
//        Circlu
        ScrollView {
            Group {
                HStack(alignment: .top) {
                    CircularImageView(uiImage: blobLoader.dataAsImage, editable: false, frame: true)
                        .frame(width: /*@START_MENU_TOKEN@*/100/*@END_MENU_TOKEN@*/, height: 100)
                        .padding(.trailing, 10)
                    VStack(alignment: .leading) {
                        Text("kevin")
                            .font(.title3)
                            .bold()
                        Text("some thing")
                            .font(.subheadline)
                            .italic()
                            .foregroundStyle(.gray)
                        
                    }
                    .padding(.top, 30)
                    Spacer()
                }
                .padding(15)
                .background(Color(.systemBackground))
                .cornerRadius(10)
            }
//            .background(Color(.systemBackground))

//            .frame(maxWidth: /*@START_MENU_TOKEN@*/.infinity/*@END_MENU_TOKEN@*/)
            .padding(.horizontal, 20)
            .padding(.bottom, 10)
            

            Group {
                Group {
                    VStack {
                        Toggle("Share Bio", isOn: $shareBio)
                        Divider()
                        ZStack(alignment: .topLeading) {
                            Text(bioText)
                                .padding(5)
                                .padding(.top, 3)
                                .padding(.bottom, 5)
                                .opacity(0)
                                .foregroundColor(.blue)
                            TextEditor(text:$bioText)
                                
                        }
                    }
                    
                }
                .padding(15)
                .background(Color(.systemBackground))
                .cornerRadius(10)
                
            }
            .padding(.horizontal,20)
            
            
            
            let columns = [GridItem(.adaptive(minimum: 100))]
            HStack {
                Spacer()
                Button(action: {
                    
                }, label: {
                    HStack { Image(systemName: "plus"); Text("Add")}
                                        .foregroundColor(.white)
                                        .padding(10)
                                        .background(.gray)
                                        .cornerRadius(30)
                })
                .padding(.trailing,5)
            }
            ScrollView {
                LazyVGrid(columns: columns, alignment: .leading, spacing: 0) {
                    ForEach(items, id: \.self) { item in
                        Image("crow")
                            .resizable()
                            .aspectRatio(1.0, contentMode: .fit)
                        
                    }
                }
                
            }
        }
        .background(Color(.secondarySystemBackground))
        .onAppear {
            Task {
                await blobLoader.loadHash(hash: viewModel.pic)
            }
        }
    }

}

#Preview {
    BlobCache.shared.setLocalImage("crow", for: WideId(1))
    var vm = NearbyPersonDetailsVM()
    vm.name = "kevin"
    vm.pic = WideId(1)
    let bl = BlobLoader()
    return NearbyPersonDetailsView(viewModel: vm)
}
