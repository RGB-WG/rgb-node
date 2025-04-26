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

use std::net::SocketAddr;
use std::path::PathBuf;

use bpwallet::cli::GeneralOpts;

use crate::rgbnode::Config;

pub const RGB_NODE_CONFIG: &str = "{data_dir}/rgb-node.toml";

/// Command-line arguments
#[derive(Parser)]
#[derive(Clone, Eq, PartialEq, Debug)]
#[command(author, version, about)]
pub struct Opts {
    /// Set a verbosity level
    ///
    /// Can be used multiple times to increase verbosity.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(flatten)]
    pub general: GeneralOpts,

    /// Address(es) to listen for client RPC connections
    // Port - ASCII hex for 'SC' ("smart contracts")
    #[arg(short, long, default_value = "127.0.0.1:5343")]
    pub listen: Vec<SocketAddr>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Command {
    Init,
}

impl Opts {
    pub fn process(&mut self) { self.general.process(); }

    pub fn conf_path(&self, name: &'static str) -> PathBuf {
        let mut conf_path = self.general.base_dir();
        conf_path.push(name);
        conf_path.set_extension("toml");
        conf_path
    }

    pub fn into_config_cmd(self) -> (Config, Option<Command>) {
        let config = Config {
            data_dir: self.general.base_dir(),
            network: self.general.network,
            rpc: self.listen,
        };
        (config, self.command)
    }
}
