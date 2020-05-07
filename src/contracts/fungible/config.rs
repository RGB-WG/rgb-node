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

const FUNGIBLED_CACHE: &'static str = "rgb-cache.sqlite";
const FUNGIBLED_SOCKET_REP: &'static str = "tcp://0.0.0.0:13801";
const FUNGIBLED_SOCKET_PUB: &'static str = "tcp://0.0.0.0:13901";
const FUNGIBLED_STASHD_REQ: &'static str = "tcp://0.0.0.0:13000";
const FUNGIBLED_STASHD_SUB: &'static str = "tcp://0.0.0.0:13300";

#[derive(Clap)]
#[clap(
    name = "fungibled",
    version = "0.1.0",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>",
    about = "RGB fungible contract daemon; part of RGB suite"
)]
pub struct Opts {
    /// Path and name of the configuration file
    #[clap(short = "c", long = "config", default_value = "fungibled.toml")]
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
    #[clap(short = "s", long = "stash", default_value = FUNGIBLED_CACHE, env = "RGB_FUNGIBLED_CACHE")]
    pub cache: String,

    /// ZMQ socket address string for REQ/REP API
    #[clap(
        long = "socket-rep",
        default_value = FUNGIBLED_SOCKET_REP,
        env = "RGB_FUNGIBLED_SOCKET_REP",
        parse(try_from_str)
    )]
    pub socket_rep: String,

    /// ZMQ socket address string for PUB/SUb API
    #[clap(
        long = "socket-pub",
        default_value = FUNGIBLED_SOCKET_PUB,
        env = "RGB_FUNGIBLED_SOCKET_PUB",
        parse(try_from_str)
    )]
    pub socket_pub: String,

    /// ZMQ socket address string for REQ/REP API
    #[clap(
        long = "socket-req",
        default_value = FUNGIBLED_STASHD_REQ,
        env = "RGB_STASHD_STASHD_REQ",
        parse(try_from_str)
    )]
    pub stash_req: String,

    /// ZMQ socket address string for PUB/SUb API
    #[clap(
        long = "socket-req",
        default_value = FUNGIBLED_STASHD_SUB,
        env = "RGB_STASHD_API_SUB",
        parse(try_from_str)
    )]
    pub stash_sub: String,
}

// We need config structure since not all of the parameters can be specified
// via environment and command-line arguments. Thus we need a config file and
// default set of configuration
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Config {
    pub verbose: u8,
    pub cache: String,
    pub socket_rep: String,
    pub socket_pub: String,
    pub stash_req: String,
    pub stash_sub: String,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Self {
            verbose: opts.verbose,
            cache: opts.cache,
            socket_rep: opts.socket_rep,
            socket_pub: opts.socket_pub,
            stash_req: opts.stash_req,
            stash_sub: opts.stash_sub,
            ..Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            cache: FUNGIBLED_CACHE.to_string(),
            socket_rep: FUNGIBLED_SOCKET_REP.to_string(),
            socket_pub: FUNGIBLED_SOCKET_PUB.to_string(),
            stash_req: FUNGIBLED_STASHD_REQ.to_string(),
            stash_sub: FUNGIBLED_STASHD_SUB.to_string(),
        }
    }
}
