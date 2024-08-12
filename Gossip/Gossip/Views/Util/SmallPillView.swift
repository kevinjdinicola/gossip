//
//  SmallPillView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/8/24.
//

import SwiftUI


struct SmallPillModifier<S: ShapeStyle>: ViewModifier {
    
    var style: S
    
    func body(content: Content) -> some View {
        content
            .font(.footnote)
            .bold()
            .padding(.vertical, 5)
            .padding(.horizontal, 10)
            .background(style, in: RoundedRectangle(cornerRadius: 25.0))
    }
}

extension View {
    func smallPillButton<S>(_ style: S) -> some View where S : ShapeStyle {
        self.modifier(SmallPillModifier(style: style))
    }
}

#Preview {
    Text("hi")
        .smallPillButton(.gray)
        .foregroundStyle(.white)
}
