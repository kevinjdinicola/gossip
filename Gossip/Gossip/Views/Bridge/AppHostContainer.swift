//
//  AppHostContainer.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/20/24.
//

import Foundation


func getLibraryDataPath() -> URL {
    var libraryPath = URL(string: "./")!;
    if let libraryDirectoryURL = FileManager.default.urls(for: .libraryDirectory, in: .userDomainMask).first {
        libraryPath = libraryDirectoryURL.appendingPathComponent("data")
        if !FileManager.default.fileExists(atPath: libraryPath.path) {
            do {
                try FileManager.default.createDirectory(atPath: libraryPath.path, withIntermediateDirectories: true)
            } catch {
                print("failed to create data dir")
            }
            
            
            print("Library data path: \(libraryPath.path)")
        }
    }
    return libraryPath
}
    

class RustApp {
    
    static var host: AppHostProtocol?
    
    static func startRustApp() {
        let isPreview = ProcessInfo.processInfo.environment["XCODE_RUNNING_FOR_PREVIEWS"] == "1";
        var host: AppHost? = nil
        
        if !isPreview {
            print("starting app host...")
            let cfg = AppConfig(dataPath: getLibraryDataPath().path, logDirective: "ghostlib=debug", devApi: DeviceApiProvider())
            host = AppHost(config: cfg)
        }
        
        RustApp.host = host
    }
    
}
