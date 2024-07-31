//
//  Dirs.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/30/24.
//

import Foundation

func getUniqueTempDir() -> URL {
    let path = NSURL.fileURL(withPathComponents: [ NSTemporaryDirectory(), UUID().uuidString])!;
    do {
        try FileManager.default.createDirectory(at: path, withIntermediateDirectories: true)
    } catch {
        print("error obtaining unique temporary directory")
    }
    
    return path
}
