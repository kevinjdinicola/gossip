//
//  NearbyPersonDetailsView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/24/24.
//

import SwiftUI
import PhotosUI


struct NearbyPersonDetailsView: View {
    
    var pk: WideId
    
    @EnvironmentObject
    var photosViewModal: PhotosViewVM
    
    @StateObject
    var viewModel = NearbyPersonDetailsVM()
    
    @State
    var controller: NearbyDetailsViewController?
    
    @State
    var shareBio: Bool = false;
    
    @State
    var bioTextDebounceTimer: Timer?
    @State
    var bioText: String = ""
    
    
    @StateObject
    var blobLoader = BlobLoader()
    
    @State var selectedPhotoPictureItems: [PhotosPickerItem] = []
    
    @State private var showModal = false
    
//    @State private var photos: [(String, UIImage)] = [("a",UIImage(named:"crow")!)]
    
    @State private var gallery: [(String, BlobLoader)] = []
    
    
    let items = Array(1...4).map { "Item \($0)" }
    
    var body: some View {

        ScrollView {
            Group {
                HStack(alignment: .top) {
                    CircularImageView(uiImage: blobLoader.dataAsImage, editable: false, frame: true)
                        .frame(width: /*@START_MENU_TOKEN@*/100/*@END_MENU_TOKEN@*/, height: 100)
                        .padding(.trailing, 10)
                    VStack(alignment: .leading) {
                        Text(viewModel.name)
                            .font(.title3)
                            .bold()
                        Text(viewModel.status)
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
            
            if !viewModel.isAvailable {
                Spacer()
                Text("Bio Unavailable")
                    .bold()
                Spacer()
            } else {
                Group {
                    Group {
                        VStack(alignment: .leading) {
                            if viewModel.isEditable {
                                Toggle("Share Bio", isOn: $shareBio)
                                Divider()
                            }
                            ZStack(alignment: .topLeading) {
                                Text(viewModel.bioText)
                                    
                                    .padding(5)
                                    .padding(.top, 3)
                                    .padding(.bottom, 5)
                                    .opacity(viewModel.isEditable ? 0 : 1 )
                                    
                                    
                                if viewModel.isEditable {
                                    TextEditor(text:$viewModel.bioText)
                                }
                                
                            }
                            .frame(maxWidth: .infinity)
                            
                        }
                        .frame(maxWidth: .infinity)
                        
                        
                    }
                    .padding(15)
                    .background(Color(.systemBackground))
                    .cornerRadius(10)
                    
                }
                .padding(.horizontal,20)
                
                
                
                let columns = [GridItem(.adaptive(minimum: 100), spacing: 1)]
                VStack {
                    
                    LazyVGrid(columns: columns, alignment: .leading, spacing: 1) {
                        ForEach(Array(gallery.enumerated()), id: \.element.0) { index, item in
                            Button(action: {
                                let x = gallery.map { g in
                                    g.1.dataAsImage!
                                };
                                photosViewModal.images = x

                                photosViewModal.index = index
                                photosViewModal.isShowing = true
                            }, label: {
                                Rectangle()
                                    .fill(.gray)
                                    .aspectRatio(1.0, contentMode: .fill)
                                    .overlay {
                                        if let imageData = item.1.dataAsImage {
        
                                            Image(uiImage: imageData)
                                                .resizable()
                                                .aspectRatio(contentMode: .fill)
        
                                        }
                                    }
                                    .clipped()

                                //                            Image(uiImage: item.1)
                                //                                .resizable()
                                //                                .aspectRatio(1.0, contentMode: .fit)
                            })
                        }
                        if viewModel.isEditable {
                            PhotosPicker(selection: $selectedPhotoPictureItems, maxSelectionCount: 1,
                                         matching: .images,
                                         photoLibrary: .shared()) {
                                Rectangle()
                                    .fill(Color(.systemFill))
                                    .aspectRatio(1.0, contentMode: .fill)
                                    .overlay {
                                        VStack(alignment:.center) {
                                            Text("Add")
                                            Image(systemName: "plus")
                                                .resizable()
                                                .aspectRatio(1, contentMode: .fit)
                                                .frame(width: 10)
                                        }
                                    }
                            }
                        }
                    }
                }
                
                .background(Color(.systemBackground))
                .padding(.top, 20)
            }
            
            
        }
        .background(Color(.secondarySystemBackground))
        .onAppear {
            controller = RustApp.host?.nearbyDetails(viewModel: self.viewModel, subjectPk: self.pk)
            Task {
                await loadProfilePic()
            }
            Task {
                await loadGalleryPics()
            }
        }
        .onChange(of: viewModel.pic) { oldVal, newVal in
            Task {
               await loadProfilePic()
            }
        }
        .onChange(of: viewModel.galleryPics) { _, _ in
            Task {
               await loadGalleryPics()
            }
        }
        .onChange(of: viewModel.bioText) { oldVal, newVal in
            bioText = newVal
        }
        .onChange(of: selectedPhotoPictureItems) {
            loadImage()
        }
        .onChange(of: bioText) {_, newVal in
            if viewModel.isEditable {
                bioTextDebounceTimer?.invalidate()
                bioTextDebounceTimer = Timer.scheduledTimer(withTimeInterval: 1.5, repeats: false) { _ in
                    Task {
                        try await controller?.setBioText(text: newVal)
                    }
                }

                
            }
        }
    }
    
    func loadProfilePic() async {
        await blobLoader.loadHash(hash: viewModel.pic)
    }
    
    func loadGalleryPics() async  {
        var newGallery: [(String, BlobLoader)] = []
        for gp in viewModel.galleryPics {
            let bl = BlobLoader(blobHash: gp.hash)
            await bl.loadHash(hash: bl.blobHash)
            newGallery.append((gp.name, bl))
        }
        gallery = newGallery
    }
    
    func loadImage() {
        if (selectedPhotoPictureItems.count == 0) { return; }
        let item = selectedPhotoPictureItems[0]
        let nextPicId = viewModel.galleryPics.count
        item.loadTransferable(type: Data.self) { result in
            switch result {
            case .success(let image):
                if let image = image {
                    Task {
                        try await controller?.setGalleryPic(index: UInt32(nextPicId), data: image)
                    }
                }
            case .failure(let error):
                print("Error loading image: \(error.localizedDescription)")
            }
            
        }
        selectedPhotoPictureItems = []
    }
}

#Preview {
    let pbm = PhotosViewVM();
    pbm.isShowing = true
    BlobCache.shared.setLocalImage("sylvie", for: WideId(1))
    BlobCache.shared.setLocalImage("crow", for: WideId(2))
    var vm = NearbyPersonDetailsVM()
    vm.name = "kevin"
    vm.pic = WideId(1)
    vm.bioText="cheeeeese"
    vm.status = "shiii"
    vm.isEditable = false
    vm.isAvailable = true
    let bl = BlobLoader()
    vm.galleryPics = [NamedBlob(name: "sylvie", hash: WideId(1)),NamedBlob(name: "crow", hash: WideId(2))]
    
    return NearbyPersonDetailsView(pk: WideId(3), viewModel: vm)
        .environment(pbm)
}
