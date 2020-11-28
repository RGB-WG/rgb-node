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

use clap::{ArgEnum, Clap};
use lnpbp::bp;
use std::path::PathBuf;

use serde::Deserialize;

use crate::constants::*;
use crate::DataFormat;

#[derive(Clap)]
#[clap(
    name = "rgbd",
    version = "0.1.0",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>",
    about = "RGB main daemon; part of RGB suite"
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

    /// ZMQ socket address string for PUB/SUB API of fungibled
    #[clap(
        long = "fungible-pub",
        default_value = FUNGIBLED_PUB_ENDPOINT,
        env = "RGB_FUNGIBLED_PUB"
    )]
    pub fungible_pub_endpoint: String,

    /// ZMQ socket address string for REQ/REP API of stashd
    #[clap(
        long = "stash-rpc",
        default_value = STASHD_RPC_ENDPOINT,
        env = "RGB_STASHD_RPC"
    )]
    pub stash_rpc_endpoint: String,

    /// ZMQ socket address string for PUB/SUB API of stashd
    #[clap(
        long = "stash-pub",
        default_value = STASHD_PUB_ENDPOINT,
        env = "RGB_STASHD_PUB"
    )]
    pub stash_pub_endpoint: String,

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
    pub format: DataFormat,

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

    /// LNP socket address string for P2P API of stashd
    #[clap(
        long = "bind",
        default_value = STASHD_P2P_ENDPOINT,
        env = "RGB_STASHD_BIND"
    )]
    pub p2p_endpoint: String,

    /// Run services as threads instead of daemons
    #[clap(short, long)]
    pub threaded: bool,

    /// Bitcoin network to use
    #[clap(short, long, default_value = RGB_NETWORK, env = "RGB_NETWORK")]
    pub network: bp::Chain,

    /// Electrum server to use to fecth Bitcoin transactions
    #[clap(
        long = "electrum",
        default_value = DEFAULT_ELECTRUM_ENDPOINT,
        env = "RGB_ELECTRUM_SERVER"
    )]
    pub electrum_server: String,
}

#[derive(Clap, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
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
    pub network: bp::Chain,
    pub verbose: u8,
    pub fungible_rpc_endpoint: String,
    pub fungible_pub_endpoint: String,
    pub stash_rpc_endpoint: String,
    pub stash_pub_endpoint: String,
    pub cache: String,
    pub format: DataFormat,
    pub stash: String,
    pub index: String,
    pub p2p_endpoint: String,
    pub electrum_server: String,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Self {
            data_dir: opts.data_dir.into(),
            bin_dir: opts.bin_dir.into(),
            threaded: opts.threaded,
            network: opts.network,
            contracts: opts.contracts,
            fungible_rpc_endpoint: opts.fungible_rpc_endpoint,
            fungible_pub_endpoint: opts.fungible_pub_endpoint,
            stash_rpc_endpoint: opts.stash_rpc_endpoint,
            stash_pub_endpoint: opts.stash_pub_endpoint,
            cache: opts.cache,
            format: opts.format,
            stash: opts.stash,
            index: opts.index,
            p2p_endpoint: opts.p2p_endpoint,
            verbose: opts.verbose,
            electrum_server: opts.electrum_server,
        }
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
            fungible_rpc_endpoint: FUNGIBLED_RPC_ENDPOINT.to_string(),
            fungible_pub_endpoint: FUNGIBLED_PUB_ENDPOINT.to_string(),
            stash_rpc_endpoint: STASHD_RPC_ENDPOINT.to_string(),
            stash_pub_endpoint: STASHD_PUB_ENDPOINT.to_string(),
            cache: FUNGIBLED_CACHE.to_string(),
            format: DataFormat::Yaml,
            stash: STASHD_STASH.to_string(),
            index: STASHD_INDEX.to_string(),
            p2p_endpoint: STASHD_P2P_ENDPOINT.to_string(),
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
            fungible_pub_endpoint: FUNGIBLED_PUB_ENDPOINT.to_string(),
            stash_rpc_endpoint: STASHD_RPC_ENDPOINT.to_string(),
            stash_pub_endpoint: STASHD_PUB_ENDPOINT.to_string(),
            cache: FUNGIBLED_CACHE.to_string(),
            format: DataFormat::Yaml,
            stash: STASHD_STASH.to_string(),
            index: STASHD_INDEX.to_string(),
            p2p_endpoint: STASHD_P2P_ENDPOINT.to_string(),
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
