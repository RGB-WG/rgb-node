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

use super::{fungible, Runtime};
use crate::BootstrapError;
use clap::Clap;

use lnpbp::bp;
use lnpbp::lnp::transport::zmq::SocketLocator;

use crate::constants::*;

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
#[clap(
    name = "rgb-cli",
    version = "0.0.1",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>",
    about = "RGB node command-line interface; part of Lightning network protocol suite"
)]
pub struct Opts {
    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(
        global = true,
        short = "v",
        long = "verbose",
        min_values = 0,
        max_values = 4,
        parse(from_occurrences)
    )]
    pub verbose: u8,

    /// RPC endpoint of contracts service
    #[clap(short, long, default_value = FUNGIBLED_RPC_ENDPOINT)]
    endpoint: SocketLocator,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,

    /// Bitcoin network to use
    #[clap(short, long, default_value = RGB_NETWORK, env = "RGB_NETWORK")]
    pub network: bp::Network,
}

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
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
#[display_from(Debug)]
pub struct Config {
    pub verbose: u8,
    pub endpoint: SocketLocator,
    pub network: bp::Network,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Self {
            verbose: opts.verbose,
            endpoint: opts.endpoint,
            network: opts.network,
            ..Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            endpoint: FUNGIBLED_RPC_ENDPOINT
                .parse()
                .expect("Broken FUNGIBLED_RPC_ENDPOINT value"),
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
        }
    }
}

impl Command {
    pub fn exec(self, runtime: &Runtime) -> Result<(), BootstrapError> {
        match self {
            Command::Fungible { subcommand, .. } => subcommand.exec(runtime),
        }
    }
}
