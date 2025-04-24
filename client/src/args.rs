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

use rgbrpc::RemoteAddr;

/// Command-line tool for working with store daemon
#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[command(name = "bp-cli", bin_name = "bp-cli", author, version)]
pub struct Args {
    /// Set a verbosity level
    ///
    /// Can be used multiple times to increase verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Remote address of the RGB node to connect to
    #[arg(short, long, default_value = "127.0.0.1:5343")]
    pub remote: RemoteAddr,

    /// Command to execute
    #[command(subcommand)]
    pub command: Command,
}

/// Command-line commands:
#[derive(Subcommand, Clone, PartialEq, Eq, Debug, Display)]
pub enum Command {
    #[display("ping")]
    Ping,
}
