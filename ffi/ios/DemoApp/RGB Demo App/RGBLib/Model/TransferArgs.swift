//
//  TransferArgs.swift
//  RGB Demo App
//
//  Created by Jason van den Berg on 2020/07/11.
//  Copyright © 2020 LNP/BP Standards Association. All rights reserved.
//

import Foundation

struct TransferArgs: Arguments {
    let inputs: [String]
    let allocate: [CoinAllocation]
    let invoice: String
    let prototype_psbt: String
    let fee: UInt
    let change: String
    let consignment_file: String
    let transaction_file: String
}
