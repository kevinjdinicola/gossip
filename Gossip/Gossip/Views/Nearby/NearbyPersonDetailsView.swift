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
    var isEditorPresented: Bool = false
    @State
    var editorBioText: String = "asdf"
    
    @State
    var profileImage: UIImage?
    
    @State
    var gallerySelectionMode = false
    @State
    var gallerySelectedIndicies: Set<Int> = Set()
    
    @State var selectedPhotoPictureItems: [PhotosPickerItem] = []
    
    @State private var showModal = false
    
//    @State private var photos: [(String, UIImage)] = [("a",UIImage(named:"crow")!)]

    
    
    let items = Array(1...4).map { "Item \($0)" }
    
    var body: some View {
        ScrollView {
            Group {
                HStack(alignment: .top) {
                    CircularImageView(uiImage: profileImage, editable: false, frame: true)
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
                .background(Color(.secondarySystemBackground))
                .cornerRadius(10)
            }
            //            .background(Color(.systemBackground))
            
            //            .frame(maxWidth: /*@START_MENU_TOKEN@*/.infinity/*@END_MENU_TOKEN@*/)
            .padding(.horizontal, 20)
            .padding(.bottom, 10)

            if !viewModel.isAvailable {
                VStack {
                    Spacer()
                    if viewModel.initialized {
                        
                        Text("Bio Unavailable")
                            .bold()
                    } else {
                        ProgressView().progressViewStyle(CircularProgressViewStyle())
                    }
                    Spacer()
                }
                .frame(maxWidth: /*@START_MENU_TOKEN@*/.infinity/*@END_MENU_TOKEN@*/, maxHeight: .infinity)
                
                    
            } else {
                if viewModel.isEditable {
                    VStack {
                        Toggle(isOn: $viewModel.shareBio) {
                            Text("Shared")
                        }
//                        Toggle(isOn: $viewModel.shareBio) {
//                            Text("Allow Invites")
//                        }
                        
                    }
                    .frame(maxWidth: .infinity)
                    .padding(15)
                    .background(Color(.secondarySystemBackground))
                    .cornerRadius(10)
                    .padding(.horizontal,20)
                    .padding(.bottom, viewModel.isEditable ? 10: 20)
                    
                }
                
                VStack {
                    
                    HStack {
                        Text(viewModel.bioText)
                            .padding(.bottom, viewModel.isEditable ? 0 : 15)
                        Spacer()
                    }
                    if viewModel.isEditable {
                        Divider()
                        Button {
                            editorBioText = viewModel.bioText
                            isEditorPresented = true
                        } label: {
                            Text("Edit Bio")
                                .frame(maxWidth: .infinity)
                                .padding(.top, 5)
                                .padding(.bottom, 10)
                        }
                    }
                    
                }
                
                .frame(maxWidth: .infinity)
                .padding(.top,15)
                .padding(.horizontal, 15)
                .padding(.bottom,0)
                .background(Color(.secondarySystemBackground))
                .cornerRadius(10)
                .padding(.horizontal,20)
                .sheet(isPresented: $isEditorPresented, content: {
                    TextEditorModal(title: "Edit Bio", isPresented: $isEditorPresented, text: $editorBioText, done: {
                        Task {
                            try await controller?.setBioText(text: editorBioText)
                        }
                    })
                })
                
                
                
                let columns = [GridItem(.adaptive(minimum: 100), spacing: 1)]
                VStack {
                    
                    if viewModel.isEditable {
                        
                        
                        HStack {
                            Spacer()
                            if !gallerySelectionMode {
                                Button {
                                    gallerySelectionMode = true
                                } label: {
                                    Text("Select")
                                        .smallPillButton(.gray)
                                        .foregroundStyle(.white)
                                }
                            } else {
                                Button {
                                    gallerySelectionMode = false
                                } label: {
                                    Text("Cancel")
                                        .smallPillButton(.gray)
                                        .foregroundStyle(.white)
                                }
                                Button {
                                    Task {
                                        await deleteSelectedGalleryImages()
                                    }
                                } label: {
                                    Image(systemName: "trash")
                                        .smallPillButton(gallerySelectedIndicies.count > 0 ? .red : .gray)
                                        .foregroundStyle(.white)
                                }
                            }
                            
                        }
                        .padding(.horizontal, 10)
                    }
                    
                    LazyVGrid(columns: columns, alignment: .leading, spacing: 1) {
                        ForEach(EnumaratedBlob.list(from: viewModel.galleryPics), id: \.id) { thing in
                            ZStack {
                                Rectangle()
                                    .fill(.gray)
                                    .aspectRatio(1.0, contentMode: .fill)
                                    .overlay(content: {
                                        BlobImage(hash: thing.hash)
                                            .aspectRatio(contentMode: .fill)
                                    })
                                    .clipped()
                                    .contentShape(Rectangle())
                                if gallerySelectionMode && gallerySelectedIndicies.contains(thing.index) {
                                    Rectangle()
                                        .fill(Color(.white).opacity(0.5))
                                }
                            }
                            .clipped()
                            .onTapGesture(perform: {
                                if gallerySelectionMode {
                                    // change selections
                                    
                                    if gallerySelectedIndicies.contains(thing.index) {
                                        gallerySelectedIndicies.remove(thing.index)
                                    } else {
                                        gallerySelectedIndicies.insert(thing.index)
                                    }
                                    print("tapped \(thing.index), it's now \(gallerySelectedIndicies)")
                                } else {
                                    // open previewer
                                    Task {
                                        await photosViewModal.setImagesFromHashes(hashes: viewModel.galleryPics)
                                        photosViewModal.index = thing.index
                                        photosViewModal.isShowing = true
                                    }
                                    
                                }
                                 
                            })

                        }
                        if viewModel.isEditable && viewModel.galleryPics.count < 9 {
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
                    WideIdView(wideId: self.pk)
                        .padding(.top)


                }
                
                .background(Color(.systemGroupedBackground))
                .padding(.top, 20)
            }
            
            
        }
        .background(Color(.systemGroupedBackground))
        .onAppear {
            controller = RustApp.host?.nearbyDetails(viewModel: self.viewModel, subjectPk: self.pk)
            Task {
                await loadProfilePic()
            }
        }
        .onChange(of: pk) {
            // this SHOULD get rid of the old controller and i should see a deallocation
            controller = RustApp.host?.nearbyDetails(viewModel: self.viewModel, subjectPk: self.pk)
            Task {
                await loadProfilePic()
            }
        }
        .onChange(of: viewModel.pic) { oldVal, newVal in
            Task {
               await loadProfilePic()
            }
        }

        .onChange(of: selectedPhotoPictureItems) {
            loadImage()
        }
        .onChange(of: viewModel.shareBio) {
            if !viewModel.initialized { return }
            Task {
                // only works because im not syncing the value back.. is this bad?
                try await controller?.setShareBio(shouldShare: viewModel.shareBio)
            }
        }
        
    }
    
    func loadProfilePic() async {
        self.profileImage = viewModel.pic != nil ? await ImageLoader.load(hash: viewModel.pic!) : nil
    }
    
    func deleteSelectedGalleryImages() async {
        gallerySelectionMode = false
        var newPics: [WideId] = [];
        viewModel.galleryPics.enumerated().forEach { i,hash in
            if !gallerySelectedIndicies.contains(i) {
                newPics.append(hash)
            }
        }
        
        do {
            try await controller?.setGalleryPic(pics: newPics)
        } catch {
            print("fuck")
        }
        
        
        gallerySelectedIndicies = []
    }
    

    func loadImage() {
        if (selectedPhotoPictureItems.count == 0) { return; }
        let item = selectedPhotoPictureItems[0]

        item.loadTransferable(type: Data.self) { result in
            switch result {
            case .success(let image):
                if let image = image {
                    Task {
                        if let newImg: WideId = try await RustApp.host?.saveBlob(data: image) {
                            var newPicSet = viewModel.galleryPics
                            newPicSet.append(newImg)
                            try await controller?.setGalleryPic(pics: newPicSet)
                        }
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
    ImageLoader.setLocalImage("sylvie", for: WideId(1))
    ImageLoader.setLocalImage("crow", for: WideId(2))
    var vm = NearbyPersonDetailsVM()
    vm.initialized = false
    
    vm.name = "kevin"
    vm.pic = WideId(1)
    vm.bioText="cheeeeese"
    vm.status = "shiii"
    vm.isEditable = false
    vm.isAvailable = true
    vm.galleryPics = [WideId(1),WideId(2)]
    
    return NearbyPersonDetailsView(pk: WideId(3), viewModel: vm)
        .environment(pbm)
}
