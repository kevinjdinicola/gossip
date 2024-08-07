//
//  PhotosViewModal.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/2/24.
//

import SwiftUI

struct PhotosViewModal: View {
    
    
    @EnvironmentObject
    var viewModel: PhotosViewVM
    
    @State private var currentIndex = 0
    @State private var dragOffset: CGFloat = 0
    
    var body: some View {
        GeometryReader { geometry in
            ZStack {
                Group {
                    
                    ZStack {
                        TabView(selection: $currentIndex) {
                            ForEach(0..<viewModel.images.count, id: \.self) { index in
                                ZoomableScrollView {
                                    Image(uiImage: viewModel.images[index])
                                        .resizable()
                                        .scaledToFit()
                                        .tag(index)
                                        .background(.black)
                                        
                                }
                                .background(.black)
                                
                              
                            }.background(.black)
                        }.background(.black)
//                        Rectangle()
//                            .fill(.green)
//                            .offset(x:0, y:0)
//                            .frame(width: dragOffset)
//                            .frame(maxHeight: .infinity)
////                            .offset(x: -dragOffset)
//                        Rectangle()
//                            .fill(.red)
//                            .offset(x:210, y:0)
//                            .frame(width: 30)
//                            .frame(maxHeight: /*@START_MENU_TOKEN@*/.infinity/*@END_MENU_TOKEN@*/)
//                            .offset(x: -dragOffset)
                    }
                   
//                    .gesture(
//                        DragGesture()
//                            .onChanged { value in
//                                dragOffset = value.translation.width
//                                print(value.translation.width)
//                            }
//                            .onEnded { value in
//                                print(value)
//                                let threshold = geometry.size.width / 2
//                                if value.translation.width > threshold {
//                                    currentIndex = max(currentIndex - 1, 0)
//                                } else if value.translation.width < -threshold {
//                                    currentIndex = min(currentIndex + 1, self.viewModel.images.count - 1)
//                                }
//                                dragOffset = 0
//                            }
//                    )
                    .tabViewStyle(PageTabViewStyle())
                    
                }.frame(maxHeight: .infinity)
                VStack {
                    HStack {
                        Spacer()
                        Button(action: {
                            viewModel.isShowing = false
                            viewModel.images = []
                        }, label: {
                            Text("Done").foregroundStyle(.yellow)
                        })
                        .padding(.trailing, 10)
                    }
                    .padding(.bottom,20)
                    .background(.ultraThinMaterial)

                    Spacer()
                }

            }
            .foregroundStyle(.white)
            .frame(maxWidth: /*@START_MENU_TOKEN@*/.infinity/*@END_MENU_TOKEN@*/, maxHeight: .infinity)
            .background(.black)
            .onChange(of: viewModel.index) { oldVal, newVal in
                currentIndex = newVal
            }
            .onAppear {
                currentIndex = viewModel.index
            }
        }
    }
}

struct ZoomableScrollView<Content: View>: UIViewRepresentable {
    let content: Content

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    func makeUIView(context: Context) -> UIScrollView {
        let scrollView = UIScrollView()
        scrollView.delegate = context.coordinator
        scrollView.maximumZoomScale = 5.0
        scrollView.minimumZoomScale = 1.0
        scrollView.bouncesZoom = true
        scrollView.backgroundColor = .black

        let hostedView = context.coordinator.hostingController.view!
        hostedView.translatesAutoresizingMaskIntoConstraints = true
        hostedView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        hostedView.frame = scrollView.bounds
        hostedView.backgroundColor = .black
        scrollView.addSubview(hostedView)

        return scrollView
    }

    func updateUIView(_ uiView: UIScrollView, context: Context) {
        context.coordinator.hostingController.rootView = content
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(hostingController: UIHostingController(rootView: content))
    }

    class Coordinator: NSObject, UIScrollViewDelegate {
        var hostingController: UIHostingController<Content>

        init(hostingController: UIHostingController<Content>) {
            self.hostingController = hostingController
        }

        func viewForZooming(in scrollView: UIScrollView) -> UIView? {
            return hostingController.view
        }
    }
}

#Preview {
    var viewModel = PhotosViewVM()
    viewModel.images = [UIImage(named:"crow")!,UIImage(named:"crow")!]
    return PhotosViewModal()
        .environment(viewModel)
}
