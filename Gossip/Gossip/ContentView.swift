//
//  ContentView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/13/24.
//

import SwiftUI

struct ContentView: View {
    
    @EnvironmentObject
    var model: GlobalVM
    
    @EnvironmentObject
    var photosViewModel: PhotosViewVM
    
    @State private var selectedTab = 0
    
    var body: some View {
        ZStack {
            TabView(selection: $selectedTab) {
                NearbyContainerView(model: model)
                    .tabItem {
                        Image(systemName: "mappin.and.ellipse")
                        Text("Nearby")
                    }
                    .tag(0)
                MeView()
                    .tabItem {
                        Image(systemName: "person.fill")
                        Text("Me")
                    }
                    .tag(1)
                ControlsContainer()
                    .tabItem {
                        Image(systemName: "gear")
                        Text("Controls")
                    }
                    .tag(2)
            }

            if photosViewModel.isShowing {
                PhotosViewModal()
            }
        }
       
    }
}

#Preview {
    let x = GlobalVM();
    x.name = "hi"
    return ContentView()
        .environmentObject(x)
        .environment(PhotosViewVM())
}
