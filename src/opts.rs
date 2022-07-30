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
use lnpbp::chain::Chain;
use store_rpc::STORED_RPC_ENDPOINT;

#[cfg(any(target_os = "linux"))]
pub const RGB_NODE_DATA_DIR: &str = "~/.rgb_node";
#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub const RGB_NODE_DATA_DIR: &str = "~/.rgb_node";
#[cfg(target_os = "macos")]
pub const RGB_NODE_DATA_DIR: &str = "~/Library/Application Support/RGB Node";
#[cfg(target_os = "windows")]
pub const RGB_NODE_DATA_DIR: &str = "~\\AppData\\Local\\RGB Node";
#[cfg(target_os = "ios")]
pub const RGB_NODE_DATA_DIR: &str = "~/Documents";
#[cfg(target_os = "android")]
pub const RGB_NODE_DATA_DIR: &str = ".";

pub const RGB_NODE_CTL_ENDPOINT: &str = "{data_dir}/ctl";

pub const RGB_NODE_CONFIG: &str = "{data_dir}/rgb_node.toml";

/// Command-line arguments
#[derive(Parser)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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

    /// ZMQ socket for connecting storage daemon.
    #[clap(
        short = 'S',
        long = "store",
        global = true,
        env = "STORED_RPC_ENDPOINT",
        default_value = STORED_RPC_ENDPOINT,
        value_hint = ValueHint::FilePath
    )]
    pub store_endpoint: ServiceAddr,

    /// ZMQ socket for internal service bus.
    ///
    /// A user needs to specify this socket usually if it likes to distribute daemons
    /// over different server instances. In this case all daemons within the same node
    /// must use the same socket address.
    ///
    /// Socket can be either TCP address in form of `<ipv4 | ipv6>:<port>` – or a path
    /// to an IPC file.
    ///
    /// Defaults to `ctl` file inside `--data-dir` directory, unless `--threaded-daemons`
    /// is specified; in that cases uses in-memory communication protocol.
    #[clap(
        short = 'X',
        long = "ctl",
        global = true,
        env = "RGB_NODE_CTL_ENDPOINT",
        default_value = RGB_NODE_CTL_ENDPOINT,
        value_hint = ValueHint::FilePath
    )]
    pub ctl_endpoint: ServiceAddr,

    /// Blockchain to use
    #[clap(
        short = 'n',
        long,
        global = true,
        alias = "network",
        default_value = "signet",
        env = "RGB_NODE_NETWORK"
    )]
    pub chain: Chain,

    /// Electrum server to use.
    #[clap(
        long,
        global = true,
        default_value("pandora.network"),
        env = "RGB_NODE_ELECTRUM_SERVER",
        value_hint = ValueHint::Hostname
    )]
    pub electrum_server: String,

    /// Customize Electrum server port number. By default the wallet will use port
    /// matching the selected network.
    #[clap(long, global = true, env = "RGB_NODE_ELECTRUM_PORT")]
    pub electrum_port: Option<u16>,
}

#[cfg(feature = "server")]
impl Opts {
    pub fn process<'s>(&'s mut self, other: impl IntoIterator<Item = &'s mut ServiceAddr>) {
        let mut services = vec![&mut self.ctl_endpoint, &mut self.store_endpoint];
        services.extend(other);
        microservices::shell::shell_setup(self.verbose, services, &mut self.data_dir, &[(
            "{chain}",
            self.chain.to_string(),
        )]);
    }
}
