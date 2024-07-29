//
//  DataExtension.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/27/24.
//

import Foundation

extension DisplayMessage {
    init(id: UInt32, text: String, isSelf: Bool) {
        self.id = id
        self.text = text
        self.isSelf = isSelf
        self.payload = nil
    
    }
}
