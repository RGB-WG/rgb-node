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

use internet2::zmqsocket::ZmqSocketAddr;
use internet2::LocalNode;
use lnpbp::Chain;
use microservices::FileFormat;

use crate::constants::*;

#[derive(Clap)]
#[clap(
    name = "stashd",
    version = "0.1.0",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>",
    about = "RGB stashd: daemon managing RGB smart contract stash; part of RGB suite"
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
    #[clap(short, long, default_value = STASHD_STASH, env = "RGB_STASHD_STASH")]
    pub stash: String,

    /// Connection string to indexing service
    #[clap(short, long, default_value = STASHD_INDEX, env = "RGB_STASHD_INDEX")]
    pub index: String,

    /// ZMQ socket address string for REQ/REP API
    #[clap(
        long = "rpc",
        default_value = STASHD_RPC_ENDPOINT,
        env = "RGB_STASHD_RPC"
    )]
    pub rpc_endpoint: String,

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

// We need config structure since not all of the parameters can be specified
// via environment and command-line arguments. Thus we need a config file and
// default set of configuration
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    pub node_auth: LocalNode,
    pub verbose: u8,
    pub data_dir: PathBuf,
    pub stash: String,
    pub index: String,
    pub format: FileFormat,
    pub rpc_endpoint: ZmqSocketAddr,
    pub network: Chain,
    pub electrum_server: String,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        let mut me = Self {
            verbose: opts.verbose,
            network: opts.network,
            ..Config::default()
        };
        me.data_dir = me.parse_param(opts.data_dir);
        me.stash = me.parse_param(opts.stash);
        me.index = me.parse_param(opts.index);
        me.rpc_endpoint = me.parse_param(opts.rpc_endpoint);
        me.electrum_server = me.parse_param(opts.electrum_server);
        me
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_auth: LocalNode::new(),
            verbose: 0,
            data_dir: RGB_DATA_DIR
                .parse()
                .expect("Error in RGB_DATA_DIR constant value"),
            stash: STASHD_STASH.to_string(),
            index: STASHD_INDEX.to_string(),
            #[cfg(feature = "serde_yaml")]
            format: FileFormat::Yaml,
            #[cfg(not(feature = "serde"))]
            format: FileFormat::StrictEncode,
            rpc_endpoint: STASHD_RPC_ENDPOINT
                .parse()
                .expect("Error in STASHD_RPC_ENDPOINT constant value"),
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
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
            .replace("{node_id}", &self.node_auth.node_id().to_string())
            .parse()
            .unwrap_or_else(|err| {
                panic!("Error parsing parameter `{}`: {}", param, err)
            })
    }
}
