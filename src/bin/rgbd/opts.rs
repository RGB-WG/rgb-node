// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::path::PathBuf;

use clap::{Parser, ValueHint};
use internet2::addr::ServiceAddr;
use rgb_rpc::RGB_NODE_RPC_ENDPOINT;
use store_rpc::STORED_RPC_ENDPOINT;
use storm_app::STORM_NODE_APP_ENDPOINT;

#[cfg(any(target_os = "linux"))]
pub const RGB_NODE_DATA_DIR: &str = "~/.rgb";
#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub const RGB_NODE_DATA_DIR: &str = "~/.rgb";
#[cfg(target_os = "macos")]
pub const RGB_NODE_DATA_DIR: &str = "~/Library/Application Support/RGB Node";
#[cfg(target_os = "windows")]
pub const RGB_NODE_DATA_DIR: &str = "~\\AppData\\Local\\RGB Node";
#[cfg(target_os = "ios")]
pub const RGB_NODE_DATA_DIR: &str = "~/Documents";
#[cfg(target_os = "android")]
pub const RGB_NODE_DATA_DIR: &str = ".";

pub const RGB_NODE_CONFIG: &str = "{data_dir}/rgbd.toml";

/// Command-line arguments
#[derive(Parser)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[clap(author, version, name = "rgbd", about = "storm node managing service")]
pub struct Opts {
    /// Set verbosity level.
    ///
    /// Can be used multiple times to increase verbosity
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Data directory path.
    ///
    /// Path to the directory that contains stored data, and where ZMQ RPC
    /// socket files are located
    #[clap(
        short,
        long,
        global = true,
        default_value = RGB_NODE_DATA_DIR,
        env = "RGB_NODE_DATA_DIR",
        value_hint = ValueHint::DirPath
    )]
    pub data_dir: PathBuf,

    /// ZMQ socket for connecting Storm node message bus.
    #[clap(
        long,
        env = "STORM_NODE_APP_ENDPOINT",
        default_value = STORM_NODE_APP_ENDPOINT,
        value_hint = ValueHint::FilePath
    )]
    pub storm_endpoint: ServiceAddr,

    /// ZMQ socket for connecting Storm node message bus.
    #[clap(
        long,
        env = "STORED_RPC_ENDPOINT",
        default_value = STORED_RPC_ENDPOINT,
        value_hint = ValueHint::FilePath
    )]
    pub store_endpoint: ServiceAddr,

    /// ZMQ socket name/address for storm node RPC interface.
    ///
    /// Internal interface for control PRC protocol communications.
    #[clap(
        short = 'x',
        long,
        env = "RGB_NODE_RPC_ENDPOINT",
        value_hint = ValueHint::FilePath,
        default_value = RGB_NODE_RPC_ENDPOINT
    )]
    pub rpc_endpoint: ServiceAddr,
}
