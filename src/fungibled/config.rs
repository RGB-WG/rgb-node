// RGB standard library
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
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

use internet2::zmqsocket::ZmqSocketAddr;
use lnpbp::chain::Chain;
use microservices::FileFormat;

use crate::constants::*;

#[derive(Parser)]
#[clap(
    name = "fungibled",
    version = "0.1.0",
    author = "Dr Maxim Orlovsky <orlovsky@lnp-bp.org>",
    about = "RGB fungible contract daemon; part of RGB suite"
)]
pub struct Opts {
    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Data directory path
    #[clap(short, long, default_value = RGB_DATA_DIR, env = "RGB_DATA_DIR")]
    pub data_dir: String,

    /// Connection string to stash (exact format depends on used storage
    /// engine)
    #[clap(long = "cache", default_value = FUNGIBLED_CACHE, env = "RGB_FUNGIBLED_CACHE")]
    pub cache: String,

    /// Data format for cache storage (valid only if file storage is used)
    #[clap(short, long, default_value = "yaml", env = "RGB_FUNGIBLED_FORMAT")]
    pub format: FileFormat,

    /// ZMQ socket address string for REQ/REP API
    #[clap(
        long = "rpc",
        default_value = FUNGIBLED_RPC_ENDPOINT,
        env = "RGB_FUNGIBLED_RPC"
    )]
    pub rpc_endpoint: String,

    /// ZMQ socket address string for REQ/REP API
    #[clap(
        long,
        default_value = STASHD_RPC_ENDPOINT,
        env = "RGB_STASHD_RPC"
    )]
    pub stash_rpc: String,

    /// Bitcoin network to use
    #[clap(short, long, default_value = RGB_NETWORK, env = "RGB_NETWORK")]
    pub network: Chain,
}

// We need config structure since not all of the parameters can be specified
// via environment and command-line arguments. Thus we need a config file and
// default set of configuration
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    pub verbose: u8,
    pub data_dir: PathBuf,
    pub cache: String,
    pub format: FileFormat,
    pub rpc_endpoint: ZmqSocketAddr,
    pub stash_rpc: ZmqSocketAddr,
    pub network: Chain,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        let mut me = Self {
            verbose: opts.verbose,
            network: opts.network,
            ..Config::default()
        };
        me.data_dir = me.parse_param(opts.data_dir);
        me.cache = me.parse_param(opts.cache);
        me.rpc_endpoint = me.parse_param(opts.rpc_endpoint);
        me.stash_rpc = me.parse_param(opts.stash_rpc);
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
            cache: FUNGIBLED_CACHE.to_string(),
            #[cfg(feature = "serde_yaml")]
            format: FileFormat::Yaml,
            #[cfg(not(feature = "serde"))]
            format: FileFormat::StrictEncode,
            rpc_endpoint: FUNGIBLED_RPC_ENDPOINT
                .parse()
                .expect("Error in FUNGIBLED_RPC_ENDPOINT constant value"),
            stash_rpc: STASHD_RPC_ENDPOINT
                .parse()
                .expect("Error in STASHD_RPC_ENDPOINT constant value"),
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
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
            .replace("{id}", "default")
            .replace("{network}", &self.network.to_string())
            .replace("{data_dir}", self.data_dir.to_str().unwrap())
            .parse()
            .unwrap_or_else(|err| panic!("Error parsing parameter `{}`: {}", param, err))
    }
}
