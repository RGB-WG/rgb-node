//
//  StartRgbArgs.swift
//  RGB Demo App
//
//  Created by Jason van den Berg on 2020/07/11.
//  Copyright © 2020 LNP/BP Standards Association. All rights reserved.
//

import Foundation

struct StartRgbArgs: Arguments {
    let network: String
    let stashEndpoint: String
    let contractEndpoints: [String: String]
    let threaded: Bool
    let datadir: String
}
