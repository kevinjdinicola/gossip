//
//  CircularImageView.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 5/30/24.
//

import SwiftUI

struct CircularImageView: View {
    
    var uiImage: UIImage?;
    var editable: Bool = false
    var frame: Bool = true
    
    init(data: Data? = nil, editable: Bool = false, frame: Bool = false) {
        let uiImage = data.flatMap { UIImage(data: $0) }
        self.init(uiImage: uiImage, editable: editable, frame: frame)
    }
    // Designated initializer
    init(uiImage: UIImage? = nil, editable: Bool = false, frame: Bool = false) {
        self.editable = editable
        self.frame = frame
        self.uiImage = uiImage
    }

    // Convenience initializer for Data
    init(data: Data?) {
        let uiImage = data.flatMap { UIImage(data: $0) }
        self.init(uiImage: uiImage)
    }

    // Convenience initializer for UIImage
    init(uiImage: UIImage) {
        self.init(uiImage: Optional(uiImage))
    }
    
    var body: some View {
        GeometryReader { geometry in
            ZStack {
                Group {
                    if let img = uiImage {
                        Image(uiImage: img)
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .foregroundStyle(Color.black.opacity(0.3))
                    } else {
                        Image(systemName: "questionmark")
                            .resizable()
                            .aspectRatio(contentMode: .fit)
                            .padding(50)
                            .foregroundStyle(Color.black.opacity(0.3))
                    }
                }
                
                if editable {
                    VStack  {
                        Spacer()
                        ZStack {
                            Rectangle()
                                .fill(Color.black.opacity(0.2))
                                .frame(height: 35)
                            Text("Edit").foregroundStyle(Color.white)
                                .padding(.bottom, 10)
                        }
                        
                    }
                }
                
                
            }
            .frame(width: min(geometry.size.width, geometry.size.height),
                   height: min(geometry.size.width, geometry.size.height))
            .position(x: geometry.size.width / 2, y: geometry.size.height / 2)
        }
        .background(Color.gray)
        .clipShape(Circle())
        .overlay(
            frame ? AnyView(Circle().stroke(Color.white, lineWidth: 4)) : AnyView(EmptyView())
        )
        .shadow(radius: frame ? 0.7 : 0)
    }
}


#Preview {
    //    CircularImageView(data: nil, editable: true)
    CircularImageView(uiImage: UIImage(named: "crow")!, editable: false)
        .padding(5)
}
