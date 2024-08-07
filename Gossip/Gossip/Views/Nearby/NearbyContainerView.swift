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
    
    var body: some View {
        NavigationSplitView {
            List {
                Section {
                    HStack {
                        Toggle("Scanning", isOn: $isOn)
                    }

                }
                
                Section {
                    VStack {
                        if !model.debugState.foundGroup {
                            if model.isScanning {
                                HStack {
                                    Text("Scanning").font(.caption)
                                        .bold()
                                }
                            } else {
                                Text("Nothing to see here")
                                    .font(.caption)
                            }
                        }
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

                    if model.debugState.foundGroup {
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
                if model.debugState.foundGroup {
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
                    Text("docId: \(model.debugState.docId)")
                    Text("foundGroup: \(model.debugState.foundGroup)")
                }


            }
            .navigationTitle("Nearby")
        } detail: {
            Text("Nothing selected")
        }
        .onChange(of: isOn) {
            Task {
                await GossipApp.global?.setScanning(shouldScan: isOn)
            }
        }
        .onChange(of: model.isScanning) {
            isOn = model.isScanning
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
    model.debugState.foundGroup = false
    model.isScanning = true
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
