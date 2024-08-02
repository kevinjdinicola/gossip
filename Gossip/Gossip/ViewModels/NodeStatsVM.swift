//
//  NodeStatsVM.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/30/24.
//

import Foundation

@Observable
@MainActor
class NodeStatsVM: NodeStatViewModel, ObservableObject {
    
    var stats: NodeStatsData? = nil
    
    func updateStats(stats: NodeStatsData) async {
        self.stats = stats
    }
    
    
}
