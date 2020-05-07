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

use clap::Clap;
use std::net::SocketAddr;

use lnpbp::internet::{InetAddr, InetSocketAddr};

const STASHD_STASH: &'static str = "/var/lib/rgb/stash/main";
const STASHD_INDEX: &'static str = "";
const STASHD_SOCKET_REP: &'static str = "tcp://0.0.0.0:13000";
const STASHD_SOCKET_PUB: &'static str = "tcp://0.0.0.0:13300";

#[derive(Clap)]
#[clap(
    name = "stashd",
    version = "0.1.0",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>",
    about = "RGB stashd: daemon managing RGB smart contract stash; part of RGB suite"
)]
pub struct Opts {
    /// Path and name of the configuration file
    #[clap(short = "c", long = "config", default_value = "stashd.toml")]
    pub config: String,

    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(
        short = "v",
        long = "verbose",
        min_values = 0,
        max_values = 4,
        parse(from_occurrences)
    )]
    pub verbose: u8,

    /// Connection string to stash (exact format depends on used storage engine)
    #[clap(short = "s", long = "stash", default_value = STASHD_STASH, env = "RGB_STASHD_STASH")]
    pub stash: String,

    /// Connection string to indexing service
    #[clap(short = "i", long = "index", default_value = STASHD_INDEX, env = "RGB_STASHD_INDEX")]
    pub index: String,

    /// ZMQ socket address string for REQ/REP API
    #[clap(
        long = "socket-req",
        default_value = STASHD_SOCKET_REP,
        env = "RGB_STASHD_API_REQ",
        parse(try_from_str)
    )]
    pub socket_rep: String,

    /// ZMQ socket address string for PUB/SUb API
    #[clap(
        long = "socket-req",
        default_value = STASHD_SOCKET_PUB,
        env = "RGB_STASHD_API_SUB",
        parse(try_from_str)
    )]
    pub socket_pub: String,
}

// We need config structure since not all of the parameters can be specified
// via environment and command-line arguments. Thus we need a config file and
// default set of configuration
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Config {
    pub verbose: u8,
    pub stash: String,
    pub index: String,
    pub socket_rep: String,
    pub socket_pub: String,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Self {
            verbose: opts.verbose,
            stash: opts.stash,
            index: opts.index,
            socket_rep: opts.socket_rep,
            socket_pub: opts.socket_pub,
            ..Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            stash: STASHD_STASH.to_string(),
            index: STASHD_INDEX.to_string(),
            socket_rep: STASHD_SOCKET_REP.to_string(),
            socket_pub: STASHD_SOCKET_PUB.to_string(),
        }
    }
}
