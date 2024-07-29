//
//  NearbyPersonListItem.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 7/7/24.
//

import SwiftUI


struct NearbyPersonRow: View {
    
    var data: NearbyProfile
    
    @StateObject
    var picData: BlobLoader = BlobLoader()

    var body: some View {
        HStack {
            CircularImageView(data: picData.data)
                .frame(width: 50, height: 50)
                .padding(.trailing, 15)
            VStack(alignment: .leading, content: {
                Text(data.name)
                    .bold()
                if data.status.text.count > 0 {
                    Text(data.status.text)
                        .italic()
                        .foregroundStyle(.gray)
                }
            })

            
            Spacer()
        }
        .onChange(of: data.pic) {
            picData.loadHash(hash: data.pic)
        }
        .onAppear() {
            picData.loadHash(hash: data.pic)
        }
        
    }
}

func nearbyProfileDummy() -> NearbyProfile {
    return NearbyProfile(pk: WideId(0), name: "Crowbert", pic: WideId(1), status: Status(text: "caw! caw! caw!"))
}

#Preview {
    BlobCache.shared.setLocalImage("crow", for: WideId(1))
    return NearbyPersonRow(data: nearbyProfileDummy())
}

