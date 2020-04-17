// Kaleidoscope: RGB command-line wallet utility
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Alekos Filini <alekos.filini@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.


use std::path::PathBuf;
use clap::Clap;

use bitcoin::util::bip32::ExtendedPubKey;

use crate::constants::*;


#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
#[clap(
    name = "kaleidoscope",
    version = "0.2.0",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>, Alekos Filini <alekos.filini@gmail.com>",
    about =  "Kaleidoscope: RGB command-line wallet utility"
)]
pub struct Opts {
    /// Path and name of the configuration file
    #[clap(global = true, short = "c", long = "config", default_value = "./kaleidoscope.toml")]
    pub config: String,

    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(global = true, short = "v", long = "verbose",
      min_values = 0, max_values = 4, parse(from_occurrences))]
    pub verbose: u8,

    /// IPC connection string for bp daemon API
    #[clap(global = true, short = "w", long = "queryd-api",
      default_value = MSGBUS_PEER_API_ADDR, env="KALEIDOSCOPE_BPD_API_ADDR")]
    pub bpd_api_socket_str: String,

    /// IPC connection string for bp daemon push notifications on transaction
    /// updates
    #[clap(global = true, short = "W", long = "queryd-push",
      default_value = MSGBUS_PEER_PUSH_ADDR, env="KALEIDOSCOPE_BPD_PUSH_ADDR")]
    pub bpd_push_socket_str: String,

    #[clap(subcommand)]
    pub command: Command
}

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Creates a new wallet and stores it in WALLET_FILE; prints extended public key to STDOUT
    WalletCreate {
        /// A file which will contain the wallet; must not exist
        #[clap(default_value = WALLET_FILE)]
        file: PathBuf
    },

    /// Returns an address for a given XPUBKEY and HD path
    AddressDerive {
        /// Extended public key
        xpubkey: ExtendedPubKey,
        /// Number of account to use
        account: u32,
        /// Index to use for the address under the account
        address: u32,
    }
}


// We need config structure since not all of the parameters can be specified
// via environment and command-line arguments. Thus we need a config file and
// default set of configuration
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Config {
    pub verbose: u8,
    pub msgbus_peer_api_addr: String,
    pub msgbus_peer_sub_addr: String,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Self {
            verbose: opts.verbose,
            msgbus_peer_api_addr: opts.bpd_api_socket_str,
            msgbus_peer_sub_addr: opts.bpd_push_socket_str,

            ..Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            msgbus_peer_api_addr: MSGBUS_PEER_API_ADDR.to_string(),
            msgbus_peer_sub_addr: MSGBUS_PEER_PUSH_ADDR.to_string()
        }
    }
}