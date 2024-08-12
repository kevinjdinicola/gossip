//
//  BGExperiment.swift
//  Gossip
//
//  Created by Kevin Dinicola on 8/9/24.
//

import Foundation
import BackgroundTasks

class BGExperiment {
    
    static let shared = BGExperiment()
    
    func register() {
        print("registering bg task")
        BGTaskScheduler.shared.register(forTaskWithIdentifier: "com.dini.gossip.mainrefresh", using: nil) { task in
            self.handleAppRefresh(task: task as! BGAppRefreshTask)
        }
        createOrAppendToFile(text: "\n\n---\n\n", to: "example.txt")
        scheduleAppRefresh()
    }
    
    func scheduleAppRefresh() {
        let request = BGAppRefreshTaskRequest(identifier: "com.dini.gossip.mainrefresh")
        // Fetch no earlier than 15 minutes from now5
        let executeMinsFromNow: Double = 15.0
        
        
        let now = Date()
        let scheduledExecution = Date(timeIntervalSinceNow: executeMinsFromNow * 60)
        createOrAppendToFile(text: "\(formatToISO8601(date: now)),\(formatToISO8601(date: scheduledExecution)),", to: "example.txt")
        
        request.earliestBeginDate = scheduledExecution
             
        do {
            print("task scheduled")
           try BGTaskScheduler.shared.submit(request)
        } catch {
           print("Could not schedule app refresh: \(error)")
        }
    }
    
    func formatToISO8601(date: Date) -> String {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter.string(from: date)
    }

    
    func handleAppRefresh(task: BGAppRefreshTask) {
        print("scheduled task executing")
        let now = Date()
        createOrAppendToFile(text: "\(formatToISO8601(date: now))\n", to: "example.txt")
        
        // Schedule a new refresh task.
        scheduleAppRefresh()
        
        task.expirationHandler = {}
        
        task.setTaskCompleted(success: true)
        
    }
    
    func getFileData() -> String {
        return readFileAndPrint(fileName: "example.txt")
    }
    
    func readFileAndPrint(fileName: String) ->String {
        let fileURL = getDocumentsDirectory().appendingPathComponent(fileName)
        
        do {
            let content = try String(contentsOf: fileURL, encoding: .utf8)
            return content
        } catch {
            print("Failed to read from file: \(error)")
            return ""
        }
    }
    
    func getDocumentsDirectory() -> URL {
        return FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    }
    
    func createOrAppendToFile(text: String, to fileName: String) {
        let fileURL = getDocumentsDirectory().appendingPathComponent(fileName)
        
        do {
            if FileManager.default.fileExists(atPath: fileURL.path) {
                // If the file exists, append the text
                let fileHandle = try FileHandle(forWritingTo: fileURL)
                fileHandle.seekToEndOfFile() // Move to the end of the file
                if let data = text.data(using: .utf8) {
                    fileHandle.write(data)
                    fileHandle.closeFile()
                }
            } else {
                // If the file doesn't exist, create it and write the text
                try text.write(to: fileURL, atomically: true, encoding: .utf8)
            }
//            print("Successfully written to file at \(fileURL)")
        } catch {
            print("Failed to write to file: \(error)")
        }
    

    }
}
