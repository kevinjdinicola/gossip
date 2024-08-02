//
//  ControlsContainer.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/26/24.
//

import SwiftUI

struct ControlsContainer: View {
    var body: some View {
        List {
            Section {
                WideButton(text: "Delete All Data", backgroundColor: .red, action: {
                    RustApp.host?.setResetFlag();
                    RustApp.host?.shutdown();
                    exit(0);
                })
                WideButton(text: "Sync peers", backgroundColor: .blue, action: {
                    Task {
                        try await GossipApp.global?.startSync()
                    }
                });
            }
            Section {
                NodeStatsView()
            }

        }

    }
    
    func printContentsOfDirectory(atPath path: String, printoffset: Int) {
        let fileManager = FileManager.default
        
        // Check if the path exists and is a directory
        var isDirectory: ObjCBool = false
        guard fileManager.fileExists(atPath: path, isDirectory: &isDirectory), isDirectory.boolValue else {
            print("The path \(path) does not exist or is not a directory.")
            return
        }
        
        // Recursively print contents
        do {
            let contents = try fileManager.contentsOfDirectory(atPath: path)
            for content in contents {
                let fullPath: String = (path as NSString).appendingPathComponent(content)
                
                print(fullPath.dropFirst(printoffset))
                
                var isDirectory: ObjCBool = false
                if fileManager.fileExists(atPath: fullPath, isDirectory: &isDirectory), isDirectory.boolValue {
                    printContentsOfDirectory(atPath: fullPath, printoffset: printoffset)
                }
            }
        } catch {
            print("Error reading contents of directory: \(error)")
        }
    }

}

#Preview {
    ControlsContainer()
}
