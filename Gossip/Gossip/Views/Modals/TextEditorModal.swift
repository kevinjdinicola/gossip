//
//  TextEditor.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/7/24.
//

import SwiftUI

struct TextEditorModal: View {
    
    
    var title: String
    @Binding var isPresented: Bool
    @Binding var text: String
    var done: () -> ()
    
    var body: some View {
        VStack {

            HStack {
                Button {
                    isPresented = false
                } label: {
                    Text("Cancel")
                }
                Spacer()
                Text(title).font(.title3)
                Spacer()
                Button {
                    isPresented = false
                    done()
                } label: {
                    Text("Done")
                }
                
            }
            .frame(height: 50)

            .padding(.horizontal,10)
            .background(Color(.secondarySystemBackground))
            TextEditor(text: $text)
                
        }

    }
}

#Preview {
    TextEditorModal(title: "Update Bio", isPresented: .constant(true),text: Binding.constant("hello"), done: {})
}
