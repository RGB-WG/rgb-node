// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use core::fmt::Display;
use core::str::FromStr;
use std::path::PathBuf;

use lnpbp::bp;
use lnpbp::lnp::transport::zmqsocket::ZmqSocketAddr;

use super::{fungible, stash, Error, Runtime};
use crate::constants::*;

#[derive(Clap, Clone, Debug, Display)]
#[display(Debug)]
#[clap(
    name = "rgb-cli",
    version = "0.1.0-beta.2",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>",
    about = "RGB node command-line interface; part of Lightning network protocol suite"
)]
pub struct Opts {
    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Data directory path
    #[clap(short, long, default_value = RGB_DATA_DIR, env = "RGB_DATA_DIR")]
    pub data_dir: String,

    /// RPC endpoint of contracts service
    #[clap(short, long, default_value = FUNGIBLED_RPC_ENDPOINT)]
    pub fungible_endpoint: String,

    /// RPC endpoint of contracts service
    #[clap(short, long, default_value = STASHD_RPC_ENDPOINT)]
    pub stash_endpoint: String,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,

    /// Bitcoin network to use
    #[clap(short, long, default_value = RGB_NETWORK, env = "RGB_NETWORK")]
    pub network: bp::Chain,
}

#[derive(Clap, Clone, Debug, Display)]
#[display(Debug)]
pub enum Command {
    Schema {
        /// Subcommand specifying particular operation
        #[clap(subcommand)]
        subcommand: stash::SchemaCommand,
    },

    Genesis {
        /// Subcommand specifying particular operation
        #[clap(subcommand)]
        subcommand: stash::GenesisCommand,
    },

    /// Operations on fungible RGB assets (RGB-20 standard)
    Fungible {
        /// Subcommand specifying particular operation
        #[clap(subcommand)]
        subcommand: fungible::Command,
    },
}

// We need config structure since not all of the parameters can be specified
// via environment and command-line arguments. Thus we need a config file and
// default set of configuration
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    pub verbose: u8,
    pub data_dir: PathBuf,
    pub fungible_endpoint: ZmqSocketAddr,
    pub stash_endpoint: ZmqSocketAddr,
    pub network: bp::Chain,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        let mut me = Self {
            verbose: opts.verbose,
            network: opts.network,
            ..Config::default()
        };
        me.data_dir = me.parse_param(opts.data_dir);
        me.fungible_endpoint = me.parse_param(opts.fungible_endpoint);
        me.stash_endpoint = me.parse_param(opts.stash_endpoint);
        me
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            data_dir: RGB_DATA_DIR
                .parse()
                .expect("Error in RGB_DATA_DIR constant value"),
            fungible_endpoint: FUNGIBLED_RPC_ENDPOINT
                .parse()
                .expect("Broken FUNGIBLED_RPC_ENDPOINT value"),
            stash_endpoint: STASHD_RPC_ENDPOINT
                .parse()
                .expect("Broken STASHD_RPC_ENDPOINT value"),
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
        }
    }
}

impl Command {
    pub fn exec(self, runtime: Runtime) -> Result<(), Error> {
        match self {
            Command::Fungible { subcommand } => subcommand.exec(runtime),
            Command::Schema { subcommand } => subcommand.exec(runtime),
            Command::Genesis { subcommand } => subcommand.exec(runtime),
        }
    }
}

impl Config {
    pub fn parse_param<T>(&self, param: String) -> T
    where
        T: FromStr,
        T::Err: Display,
    {
        param
            .replace("{network}", &self.network.to_string())
            .replace("{data_dir}", self.data_dir.to_str().unwrap())
            .parse()
            .unwrap_or_else(|err| {
                panic!("Error parsing parameter `{}`: {}", param, err)
            })
    }
}
