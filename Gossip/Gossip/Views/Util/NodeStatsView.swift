//
//  NodeStatsView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/30/24.
//

import SwiftUI

struct NodeStatsView: View {
    
    @StateObject
    var viewModel: NodeStatsVM = NodeStatsVM()
    @State
    var nodeStat: NodeStat?;

    var body: some View {
        Group {
            if let stats = viewModel.stats {
                HStack { Text("nodeid"); Spacer();        Text(stats.nodeId.toString())}
                HStack(alignment: .top) { Text("listen addrs"); Spacer();  VStack(alignment:.trailing) {
                    ForEach(stats.listenAddrs, id: \.self) { la in
                        Text(la)
                    }
                }}
                HStack { Text("relay url"); Spacer();     Text(stats.relayUrl)}
                Group {
                    ForEach(stats.connections, id: \.nodeId) { conn in
                        VStack {
                            Text("")
                            Text("")
                            HStack { Text("nodeid").bold(); Spacer();        Text(conn.nodeId.toString())}
                            Divider()
                            HStack { Text("conn type"); Spacer();     Text(conn.connType)}
                            Divider()
                            HStack { Text("last received"); Spacer(); Text(String(conn.lastReceived))}
                            Divider()
                            HStack { Text("relay info"); Spacer();    Text(conn.relayInfo)}
                            Divider()
//                            HStack { Text("has send a"); Spacer();    Text(String(conn.hasSendAddr))}
                            HStack { Text("relay info"); Spacer();    Text(conn.relayInfo)}
                            Divider()
                            HStack(alignment: .top) { Text("addrs"); Spacer();    VStack(alignment:.trailing) {
                                ForEach(conn.addrs, id: \.self) { addr in
                                    Text(addr)
                                }
                            }}
                            
                        }
                    }
                }
            } else {
                Text("no stats")
            }
        }
        .onAppear {
            nodeStat = RustApp.host?.nodeStats(viewModel: self.viewModel)
        }
    }
}

#Preview {
    List {
        NodeStatsView()
    }

}
