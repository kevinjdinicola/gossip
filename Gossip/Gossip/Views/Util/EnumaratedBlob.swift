//
//  EnumaratedBlob.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/8/24.
//

import Foundation


struct EnumaratedBlob {
    var id: String
    var index: Int
    var hash: WideId
    
    public static func list(from: [WideId]) -> [EnumaratedBlob] {
        from.enumerated().map{ idx,hash in
            EnumaratedBlob(id:"\(idx)_\(hash)", index: idx, hash: hash)
        }
    }
}
