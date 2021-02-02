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

use clap::{AppSettings, ArgEnum, Clap};
#[cfg(feature = "serde")]
use serde::Deserialize;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use internet2::ZmqSocketAddr;
use lnpbp::Chain;
use microservices::FileFormat;

use crate::constants::*;

#[derive(Clap)]
#[clap(
    name = "rgbd",
    bin_name = "rgbd",
    author,
    version,
    setting = AppSettings::ColoredHelp
)]
pub struct Opts {
    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Path to the directory containing daemon executables
    #[clap(short, long, default_value = RGB_BIN_DIR, env = "RGB_BIN_DIR")]
    pub bin_dir: String,

    /// Data directory path
    #[clap(short, long, default_value = RGB_DATA_DIR, env = "RGB_DATA_DIR")]
    pub data_dir: String,

    /// Contract daemons to launch
    #[clap(arg_enum, long = "contract", default_value = RGB_CONTRACTS, env = "RGB_CONTRACTS")]
    pub contracts: Vec<ContractName>,

    /// ZMQ socket address string for REQ/REP API of fungibled
    #[clap(
        long = "fungible-rpc",
        default_value = FUNGIBLED_RPC_ENDPOINT,
        env = "RGB_FUNGIBLED_RPC"
    )]
    pub fungible_rpc_endpoint: String,

    /// ZMQ socket address string for REQ/REP API of stashd
    #[clap(
        long = "stash-rpc",
        default_value = STASHD_RPC_ENDPOINT,
        env = "RGB_STASHD_RPC"
    )]
    pub stash_rpc_endpoint: String,

    /// Connection string to fungibled cache (exact format depends on used
    /// storage engine)
    #[clap(
        short = 'c',
        long = "cache",
        default_value = FUNGIBLED_CACHE,
        env = "RGB_FUNGIBLED_CACHE"
    )]
    pub cache: String,

    /// Data format for fungibled cache storage (valid only if file storage is
    /// used)
    #[clap(short, long, default_value = "yaml", env = "RGB_FUNGIBLED_FORMAT")]
    pub format: FileFormat,

    /// Connection string to stashd stash (exact format depends on used storage
    /// engine)
    #[clap(
        short,
        long,
        default_value = STASHD_STASH,
        env = "RGB_STASHD_STASH"
    )]
    pub stash: String,

    /// Connection string to indexing service
    #[clap(
        short,
        long,
        default_value = STASHD_INDEX,
        env = "RGB_STASHD_INDEX"
    )]
    pub index: String,

    /// Run services as threads instead of daemons
    #[clap(short, long)]
    pub threaded: bool,

    /// Bitcoin network to use
    #[clap(short, long, default_value = RGB_NETWORK, env = "RGB_NETWORK")]
    pub network: Chain,

    /// Electrum server to use to fecth Bitcoin transactions
    #[clap(
        long = "electrum",
        default_value = DEFAULT_ELECTRUM_ENDPOINT,
        env = "RGB_ELECTRUM_SERVER"
    )]
    pub electrum_server: String,
}

#[derive(
    Clap, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display,
)]
#[display(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize,),
    serde(crate = "serde_crate")
)]
#[repr(u8)]
#[non_exhaustive]
pub enum ContractName {
    Fungible,
    Collectible,
    Identity,
}

impl ContractName {
    pub fn daemon_name(&self) -> &str {
        match self {
            ContractName::Fungible => "fungibled",
            ContractName::Collectible => "collectibled",
            ContractName::Identity => "identityd",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    pub data_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub threaded: bool,
    pub contracts: Vec<ContractName>,
    pub network: Chain,
    pub verbose: u8,
    pub fungible_rpc_endpoint: ZmqSocketAddr,
    pub stash_rpc_endpoint: ZmqSocketAddr,
    pub cache: String,
    pub format: FileFormat,
    pub stash: String,
    pub index: String,
    pub electrum_server: String,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        let mut me = Self {
            threaded: opts.threaded,
            network: opts.network,
            contracts: opts.contracts,
            format: opts.format,
            verbose: opts.verbose,
            electrum_server: opts.electrum_server,
            ..Default::default()
        };
        me.bin_dir = me.parse_param(opts.bin_dir);
        me.data_dir = me.parse_param(opts.data_dir);
        me.cache = me.parse_param(opts.cache);
        me.stash = me.parse_param(opts.stash);
        me.index = me.parse_param(opts.index);
        me.fungible_rpc_endpoint = me.parse_param(opts.fungible_rpc_endpoint);
        me.stash_rpc_endpoint = me.parse_param(opts.stash_rpc_endpoint);
        me
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: RGB_DATA_DIR
                .parse()
                .expect("Error in RGB_DATA_DIR constant value"),
            bin_dir: RGB_BIN_DIR
                .parse()
                .expect("Error in RGB_BIN_DIR constant value"),
            threaded: false,
            contracts: vec![ContractName::from_str(RGB_CONTRACTS, false)
                .expect("Error in RGB_CONTRACTS constant value")],
            fungible_rpc_endpoint: FUNGIBLED_RPC_ENDPOINT
                .parse()
                .expect("Error in FUNGIBLED_RPC_ENDPOINT value"),
            stash_rpc_endpoint: STASHD_RPC_ENDPOINT
                .parse()
                .expect("Error in STASHD_RPC_ENDPOINT value"),
            cache: FUNGIBLED_CACHE.to_string(),
            #[cfg(feature = "serde_yaml")]
            format: FileFormat::Yaml,
            #[cfg(not(feature = "serde_yaml"))]
            format: FileFormat::StrictEncode,
            stash: STASHD_STASH.to_string(),
            index: STASHD_INDEX.to_string(),
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
            verbose: 0,
            electrum_server: DEFAULT_ELECTRUM_ENDPOINT
                .parse()
                .expect("Error in DEFAULT_ELECTRUM_ENDPOINT constant value"),
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            data_dir: RGB_DATA_DIR
                .parse()
                .expect("Error in RGB_DATA_DIR constant value"),
            bin_dir: RGB_BIN_DIR
                .parse()
                .expect("Error in RGB_BIN_DIR constant value"),
            threaded: false,
            contracts: vec![ContractName::from_str(RGB_CONTRACTS, false)
                .expect("Error in RGB_CONTRACTS constant value")],
            fungible_rpc_endpoint: FUNGIBLED_RPC_ENDPOINT.to_string(),
            stash_rpc_endpoint: STASHD_RPC_ENDPOINT.to_string(),
            cache: FUNGIBLED_CACHE.to_string(),
            #[cfg(feature = "serde_yaml")]
            format: FileFormat::Yaml,
            #[cfg(not(feature = "serde_yaml"))]
            format: FileFormat::StrictEncode,
            stash: STASHD_STASH.to_string(),
            index: STASHD_INDEX.to_string(),
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
            verbose: 0,
            electrum_server: DEFAULT_ELECTRUM_ENDPOINT
                .parse()
                .expect("Error in DEFAULT_ELECTRUM_ENDPOINT constant value"),
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
            .unwrap_or_else(|err| {
                panic!("Error parsing parameter `{}`: {}", param, err)
            })
    }
}
