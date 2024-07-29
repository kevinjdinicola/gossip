//
//  AppDelegate.swift
//  Gossip
//
//  Created by Kevin Dinicola on 7/24/24.
//

import SwiftUI


class AppDelegate: UIResponder, UIApplicationDelegate {


    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Perform initial setup tasks here.
        print("app setup tasks")
        
        return true
    }
    
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
