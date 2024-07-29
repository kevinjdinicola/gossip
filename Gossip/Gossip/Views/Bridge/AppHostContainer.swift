//
//  AppHostContainer.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/20/24.
//

import Foundation

func getNewPostAttachmentsPath() -> URL {
    var libraryPath: URL = URL.init(filePath: "/")
    if let libraryDirectoryURL = FileManager.default.urls(for: .libraryDirectory, in: .userDomainMask).first {
        let uuid_str = UUID().uuidString
        libraryPath = libraryDirectoryURL.appendingPathComponent("data/post_attachments/\(uuid_str)")
        if !FileManager.default.fileExists(atPath: libraryPath.path) {
            do {
                try FileManager.default.createDirectory(atPath: libraryPath.path, withIntermediateDirectories: true)
            } catch {
                print("failed to create attachment dir")
            }
            
        }
    }
    return libraryPath
}

func getLibraryDataPath() -> String {
    var libraryPath = "";
    if let libraryDirectoryURL = FileManager.default.urls(for: .libraryDirectory, in: .userDomainMask).first {
        libraryPath = libraryDirectoryURL.appendingPathComponent("data").path
        if !FileManager.default.fileExists(atPath: libraryPath) {
            do {
                try FileManager.default.createDirectory(atPath: libraryPath, withIntermediateDirectories: true)
            } catch {
                print("failed to create data dir")
            }
            
        }
        print("Library data path: \(libraryPath)")
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
            let cfg = AppConfig(dataPath: getLibraryDataPath(), logDirective: "ghostlib=debug", devApi: DeviceApiProvider())
            host = AppHost(config: cfg)
        }
        
        RustApp.host = host
    }
    
}
