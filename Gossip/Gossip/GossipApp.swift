//
//  GossipApp.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/13/24.
//

import SwiftUI

@main
struct GossipApp: App {
    
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    var globalModel: GlobalVM
    static var global: GlobalProtocol?
    
    
    init() {
        let libraryDirectoryURL = FileManager.default.urls(for: .libraryDirectory, in: .userDomainMask).first
        FileManager().changeCurrentDirectoryPath(libraryDirectoryURL!.path)
        
        RustApp.startRustApp()
        self.globalModel = GlobalVM()
        
        GossipApp.global = RustApp.host?.global(viewModel: globalModel)
    }

    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(self.globalModel)
        }
    }
}
