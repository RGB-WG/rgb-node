//
//  RGB.swift
//  RGB Demo App
//
//  Created by Jason van den Berg on 2020/07/11.
//  Copyright © 2020 LNP/BP Standards Association. All rights reserved.
//

import Foundation

enum RuntimeError: Error {
    case start
    case issue
    case transfer
}

class Runtime {
    private var runtime: CResult
    
    init(_ args: StartRgbArgs) throws {
        let cs = (try args.toJson()).utf8String
        let buffer = UnsafeMutablePointer<Int8>(mutating: cs)
        runtime = start_rgb(buffer)
        
        guard runtime.result.rawValue == 0 else {
            throw RuntimeError.start
        }
    }
    
    func issueAsset(_ args: IssueArgs) throws {
        let cs = (try args.toJson()).utf8String
        let buffer = UnsafeMutablePointer<Int8>(mutating: cs)
        
        try withUnsafePointer(to: &runtime.inner) {ptr in
            guard issue(ptr, buffer).result.rawValue == 0 else {
                throw RuntimeError.issue
            }
        }
    }
    
    func transferAsset(_ args: TransferArgs) throws {
        let cs = (try args.toJson()).utf8String
        let buffer = UnsafeMutablePointer<Int8>(mutating: cs)
        
        try withUnsafePointer(to: &runtime.inner) {ptr in
            guard transfer(ptr, buffer).result.rawValue == 0 else {
                throw RuntimeError.transfer
            }
        }
    }
    
    deinit {
        //TODO free runtime memory with ffi destroy function when available
    }
}
