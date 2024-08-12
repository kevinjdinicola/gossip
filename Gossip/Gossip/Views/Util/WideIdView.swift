//
//  WideIdView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/11/24.
//

import SwiftUI

struct WideIdView: View {
    
    var wideId: WideId?
    
    var body: some View {
        Text(self.wideId != nil ? wideidToString(wideId: self.wideId!) : "")
            .foregroundStyle(.gray)
            .font(.system(.caption, design: .monospaced))
    }
}

#Preview {
    WideIdView(wideId: WideId(20))
}
