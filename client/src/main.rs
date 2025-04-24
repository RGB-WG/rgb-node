// RGB Node: sovereign smart contracts backend
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
// Written in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2024 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2025 RGB Consortium, Switzerland. All rights reserved.
// Copyright (C) 2020-2025 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

//! Command-line interface to BP Node

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

mod args;
mod client;
mod command;

use bpwallet::cli::{ExecError, LogLevel};
use clap::Parser;
use rgbrpc::Response;

pub use crate::args::{Args, Command};
use crate::client::BpClient;

fn main() -> Result<(), ExecError> {
    let args = Args::parse();
    LogLevel::from_verbosity_flag_count(args.verbose).apply();
    trace!("Command-line arguments: {:#?}", &args);

    let client = BpClient::new(args.remote, cb)?;

    args.command.exec(client)
}

fn cb(reply: Response) {
    match reply {
        Response::Failure(failure) => {
            println!("Failure: {failure}");
        }
        Response::Pong(_noise) => {}
        Response::Status(status) => {
            println!("{}", serde_yaml::to_string(&status).unwrap());
        }
    }
}
