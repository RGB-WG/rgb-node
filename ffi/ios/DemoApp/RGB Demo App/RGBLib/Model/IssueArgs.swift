//
//  IssueArgs.swift
//  RGB Demo App
//
//  Created by Jason van den Berg on 2020/07/11.
//  Copyright © 2020 LNP/BP Standards Association. All rights reserved.
//

import Foundation

struct IssueArgs: Arguments {
    let network: String
    let ticker: String
    let name: String
    let description: String?
    let issueStructure: String
    let allocations: [CoinAllocation]
    let precision: UInt
    let pruneSeals: [SealSpec]
    let dustLimit: UInt
}

struct CoinAllocation: Codable {
    let coins: UInt64
    let vout: UInt
    let txid: String
}

struct SealSpec: Codable {
    let vout: UInt
    let txid: String
}
