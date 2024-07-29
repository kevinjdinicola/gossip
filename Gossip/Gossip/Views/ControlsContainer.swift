//
//  ControlsContainer.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/26/24.
//

import SwiftUI

struct ControlsContainer: View {
    var body: some View {
        VStack {
            WideButton(text: "boom", action: {
                RustApp.host?.setResetFlag();
                RustApp.host?.shutdown();
                exit(0);
            })
        }.padding(10)
    }
}

#Preview {
    ControlsContainer()
}
