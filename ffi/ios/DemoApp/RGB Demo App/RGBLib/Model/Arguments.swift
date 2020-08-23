//
//  Arguments.swift
//  RGB Demo App
//
//  Created by Jason van den Berg on 2020/07/11.
//  Copyright © 2020 LNP/BP Standards Association. All rights reserved.
//

import Foundation

/// Allows argument structs to be converted to a json formatted string
protocol Arguments: Codable {}

extension Arguments {
    func toJson() throws -> NSString {
        let jsonData = try JSONEncoder().encode(self)
        return String(data: jsonData, encoding: .utf8)! as NSString
    }
}
