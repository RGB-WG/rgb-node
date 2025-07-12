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

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::exit;

use bpstd::Network;
use clap::ValueHint;

use crate::rgbnode::Config;

pub const RGB_NODE_NETWORK_ENV: &str = "RGB_NODE_NETWORK";
pub const RGB_NODE_CONFIG_ENV: &str = "RGB_NODE_CONFIG";
pub const RGB_NODE_DATA_DIR_ENV: &str = "RGB_NODE_DATA_DIR";

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
))]
pub const RGB_NODE_CONFIG: &str = "/etc/rgb-node.toml";
#[cfg(target_os = "macos")]
pub const RGB_NODE_CONFIG: &str = "/etc/rgb-node.toml";
#[cfg(target_os = "windows")]
pub const RGB_NODE_CONFIG: &str = "~\\AppData\\Local\\RGB Node\\RGB Node.toml";
#[cfg(target_os = "ios")]
pub const RGB_NODE_CONFIG: &str = "~/Documents/RGB Node/RGB Node.toml";
#[cfg(target_os = "android")]
pub const RGB_NODE_CONFIG: &str = "./RGB Node/RGB Node.toml";

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
))]
pub const RGB_NODE_DATA_DIR: &str = "/var/lib/rgb-node";
#[cfg(target_os = "macos")]
pub const RGB_NODE_DATA_DIR: &str = "/Library/Application Support/RGB Node";
#[cfg(target_os = "windows")]
pub const RGB_NODE_DATA_DIR: &str = "~\\AppData\\Local\\RGB Node";
#[cfg(target_os = "ios")]
pub const RGB_NODE_DATA_DIR: &str = "~/Documents/RGB Node";
#[cfg(target_os = "android")]
pub const RGB_NODE_DATA_DIR: &str = "./RGB Node";

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

    /// Location of the data directory
    #[clap(
        short,
        long,
        global = true,
        env = RGB_NODE_DATA_DIR_ENV,
        value_hint = ValueHint::DirPath
    )]
    pub data_dir: Option<PathBuf>,

    #[clap(
        short,
        long,
        global = true,
        env = RGB_NODE_CONFIG_ENV,
        value_hint = ValueHint::DirPath
    )]
    pub config: Option<PathBuf>,

    /// Bitcoin network
    #[arg(short, long, global = true, default_value = "testnet4", env = RGB_NODE_NETWORK_ENV)]
    pub network: Option<Network>,

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

#[derive(Clone, PartialEq, Eq, Debug)]
#[derive(Serialize, Deserialize)]
struct ConfigFile {
    pub data_dir: Option<PathBuf>,
    pub network: Option<Network>,
    pub rpc: Vec<SocketAddr>,
}

fn shell_expand(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref().display().to_string();
    let exp = shellexpand::full(&path).unwrap_or_else(|err| {
        eprintln!("Error expanding  path '{path}': {err}");
        exit(1);
    });
    PathBuf::from(exp.to_string())
}

impl Opts {
    const NAME: &'static str = "init";

    pub fn into_config_cmd(self) -> (Config, Option<Command>) {
        let config_given = self.config.is_some();
        let config_filename = self
            .config
            .unwrap_or_else(|| PathBuf::from(RGB_NODE_CONFIG));
        let config_filename = shell_expand(&config_filename);
        log::info!(target: Self::NAME, "Using config '{}'", config_filename.display());

        if !fs::exists(&config_filename).unwrap_or_default() && config_given {
            eprintln!("Config file '{}' does not exist", config_filename.display());
            exit(6);
        }
        let config_exists = fs::exists(&config_filename).unwrap_or_else(|err| {
            eprintln!("Config file '{}' cannot be accessed: {err}", config_filename.display());
            exit(7);
        });

        let default_data_dir = PathBuf::from(RGB_NODE_DATA_DIR);
        let default_network = Network::Testnet4;

        let config = if config_exists {
            let config_str = fs::read_to_string(&config_filename).unwrap_or_else(|err| {
                eprintln!("Config file '{}' cannot be opened: {err}", config_filename.display());
                exit(8);
            });
            let base_config = toml::from_str::<ConfigFile>(&config_str).unwrap_or_else(|err| {
                eprintln!("Config file '{}' is not valid TOML: {err}", config_filename.display());
                exit(9);
            });
            let rpc = if self.listen.is_empty() { base_config.rpc } else { self.listen };
            Config {
                data_dir: base_config
                    .data_dir
                    .or(self.data_dir)
                    .unwrap_or(default_data_dir),
                network: base_config
                    .network
                    .or(self.network)
                    .unwrap_or(default_network),
                rpc,
            }
        } else {
            log::info!(target: Self::NAME, "Config file not found; using command-line arguments and defaults");
            Config {
                data_dir: self.data_dir.unwrap_or(default_data_dir),
                network: self.network.unwrap_or(default_network),
                rpc: self.listen,
            }
        };

        (config, self.command)
    }
}
