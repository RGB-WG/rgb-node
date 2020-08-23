//
//  ViewController.swift
//  RGB Demo App
//
//  Created by Maxim Orlovsky on 6/10/20.
//  Copyright © 2020 LNP/BP Standards Association. All rights reserved.
//

import UIKit

class ViewController: UIViewController {

    override func viewDidLoad() {
        super.viewDidLoad()

        let s = "{}"
        let cs = (s as NSString).utf8String
        let buffer = UnsafeMutablePointer<Int8>(mutating: cs)
        start_rgb(buffer)
    }
}

