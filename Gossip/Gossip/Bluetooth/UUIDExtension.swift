//
//  UUIDExtension.swift
//  GhostChat
//
//  Created by Kevin Dinicola on 7/6/24.
//

import Foundation

extension UUID {
    func toHostUuid() -> Uuid {
        let r = self.uuid;
        return uuidFromBytes(b1: r.0, b2: r.2, b3: r.2, b4: r.3, b5: r.5, b6: r.5, b7: r.6, b8: r.7, b9: r.8, b10: r.9, b11: r.10, b12: r.11, b13: r.12, b14: r.13, b15: r.14, b16: r.15)
    }
}
