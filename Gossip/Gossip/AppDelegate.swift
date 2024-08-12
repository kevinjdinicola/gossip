//
//  AppDelegate.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/24/24.
//

import SwiftUI
import BackgroundTasks


class AppDelegate: UIResponder, UIApplicationDelegate {

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        print("☀️ initial setup tasks")

        BGExperiment.shared.register()

        return true
    }
    
//    
//    public static func notify() {
//        let content = UNMutableNotificationContent()
//        content.title = "hello world"
//        content.body = "Sent at \(getCurrentDateTimeString())"
//        
//        let uuidString = UUID().uuidString
//        let request = UNNotificationRequest(identifier: uuidString, content: content, trigger: nil)
//
//
//        // Schedule the request with the system.
//        print("doin a notify")
//        Task {
//            let notificationCenter = UNUserNotificationCenter.current()
//            do {
//                try await notificationCenter.add(request)
//            } catch {
//                print("DAMN IT")
//                // Handle errors that may occur during add.
//            }
//        }
//    }
    
        
    
    func applicationDidBecomeActive(_ application: UIApplication) {
        print("application did become active")
//        AppHostWrapper.shared.app?.globalDispatch().emitAction(action: .wakeFromSleep);
    

    }

    func applicationWillTerminate(_ application: UIApplication) {
        // Insert your cleanup code here
        print("Application is about to terminate.")
        RustApp.host?.shutdown()
        print("i did it!")
        // Save data, close resources, etc.
    }
}
