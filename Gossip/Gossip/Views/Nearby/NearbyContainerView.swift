//
//  NearbyContainerView.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/24/24.
//

import SwiftUI


struct NearbyContainerView: View {
    
    @StateObject
    var model: GlobalVM
    
    @State
    private var isOn = false
    
    @State
    var composingMessage: String = "";
    @State
    var attachments: [(String, Data)] = []
    
    @State
    var status: String = ""
    
    @State
    var statusDebounceTimer:Timer?
    
    func shouldShowChat() -> Bool {
        switch model.conState {
        case .connected(_):
            return true
        case .reconnecting:
            return true
        case .disconnected:
            return true
        default:
            return false
        }
    }
    
    var body: some View {
        NavigationSplitView {
            List {
                //                Section {
                //                    HStack {
                //                        Toggle("Scanning", isOn: $isOn)
                //                    }
                //
                switch model.conState {
                case .offline:
                    Section {
                        Button("Scan Nearby") {
                            Task {
                                try await GossipApp.global?.startScanning()
                            }
                        }.foregroundStyle(Color.accentColor)
                    }
                case .searching:
                    Section {
                        HStack {
                            Text("Scanning...")
                                .italic()
                                .foregroundStyle(.gray)
                            Spacer()
                            ProgressView().progressViewStyle(CircularProgressViewStyle())
                        }
                        Button("Cancel") {
                        }.foregroundStyle(Color.red)
                    }
                case .connected(let peerCount):
                    Section {
                        HStack {
                            Text("Connected")
                                .foregroundStyle(.green)
                            Spacer()
                            Label(
                                title: { Text(String(peerCount)).bold() },
                                icon: { Image(systemName: "point.3.connected.trianglepath.dotted") }
                            )
                            .padding(2)
                            .padding(.horizontal, 7)
                            .background(.green)
                            .foregroundStyle(.white)
                            .clipShape(RoundedRectangle(cornerRadius: 20))
                        }
                        Toggle(isOn: $isOn, label: {
                            Text("Discoverable")
                        })
                    }
                case .reconnecting:
                    Section {
                        HStack {
                            Text("Reconnecting...")
                                .italic()
                                .foregroundStyle(.gray)
                            Spacer()
                            ProgressView().progressViewStyle(CircularProgressViewStyle())
                        }
                        Button("Cancel") {
                        }.foregroundStyle(Color.red)
                    }
                case .disconnected:
                    Section {
                        Text("Disconnected")
                            .foregroundStyle(.gray)
                    }
                case .invalid:
                    Section {
                        Text("❌ Invalid State")
                            .foregroundStyle(.gray)
                    }
                }

                Section {
                    
                    HStack {
                        Text("Status")
                            .padding(.trailing, 10)
                        TextField("Whats up?", text:$status)
                        
                    }
                    NavigationLink(destination: {
                        NearbyPersonDetailsView(pk: model.ownPk)
                    }, label: {
                        Text("Bio")
                            .padding(.trailing, 10)
                    })
                    
                    if shouldShowChat() {
                        NavigationLink("Chat", destination: {
                            MessageListView(messages: model.messages, composingMessage: $composingMessage, attachments: $attachments) {
                                Task {
                                    var attachmentDirStr: String? = nil
                                    if attachments.count > 0 {
                                        let attachmentDir = getUniqueTempDir()
                                        for (i,a) in attachments {
                                            let path = attachmentDir.appendingPathComponent("\(i).png", conformingTo: .png)
                                            try a.write(to: path, options: .atomic)
                                        }
                                        attachmentDirStr = attachmentDir.path()
                                    }
                                    
                                    await GossipApp.global?.sendMessage(text:composingMessage, payloadDir: attachmentDirStr);
                                    composingMessage = ""
                                    attachments = []
                                }
                            }
                            .navigationTitle("Chat")
                        })
                    }
                    
                    
                }
                if shouldShowChat() {
                    Section {
                        
                        ForEach(model.identities, id: \.pk) { iden in
                            NavigationLink(destination: {
                                NearbyPersonDetailsView(pk: iden.pk)
                            }, label: {
                                NearbyPersonRow(data: iden)
                            })
                            
                        }
                    }
                    Section {
                        Button(action: {
                            Task {
                                try await GossipApp.global?.leaveNearbyGroup()
                            }
                        }, label: {
                            Text("Leave Group")
                                .foregroundStyle(.red)
                        })
                    }
                }
                

                Section {
                    WideIdView(wideId: model.docData.docId)
                }
                
                
            }
            .navigationTitle("Nearby")
        } detail: {
            Text("Nothing selected")
        }
        .onChange(of: isOn) {
            Task {
                try await GossipApp.global?.setBroadcasting(shouldBroadcast:isOn)
            }
        }
        .onChange(of: model.isBroadcasting) {
            isOn = model.isBroadcasting
        }
        .onChange(of: model.status.text) {
            status = model.status.text;
        }
        .onChange(of: status) {
            statusDebounceTimer?.invalidate()
            statusDebounceTimer = Timer.scheduledTimer(withTimeInterval: 1.5, repeats: false) { _ in
                Task {
                    await GossipApp.global?.setStatus(status: status)
                }
            }
            
        }
        
    }
}

#Preview {
    BlobCache.shared.setLocalImage("crow", for: WideId(1))
    var model = GlobalVM()

    model.isBroadcasting = true
    model.messages = [
        DisplayMessage(id: 0, text: "caw", isSelf: true),
        DisplayMessage(id: 1, text: "caw!!!!", isSelf: false),
    ]
    model.status = Status(text: "Whats up?")
    model.identities = [
        nearbyProfileDummy(),
        nearbyProfileDummy(),
        nearbyProfileDummy()
    ]
    return NearbyContainerView(model: model, status: "Whats up?")
}
