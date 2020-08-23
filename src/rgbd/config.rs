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

use clap::derive::ArgEnum;
use clap::Clap;
use lnpbp::bp;
use std::path::PathBuf;

use serde::Deserialize;

use crate::constants::*;

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

    /// Run services as threads instead of daemons
    #[clap(short, long)]
    pub threaded: bool,

    /// Bitcoin network to use
    #[clap(short, long, default_value = RGB_NETWORK, env = "RGB_NETWORK")]
    pub network: bp::Network,
}

#[derive(Clap, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Deserialize)]
#[display_from(Debug)]
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
#[display_from(Debug)]
pub struct Config {
    pub data_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub threaded: bool,
    pub contracts: Vec<ContractName>,
    pub network: bp::Network,
    pub verbose: u8,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Self {
            data_dir: opts.data_dir.into(),
            bin_dir: opts.bin_dir.into(),
            threaded: opts.threaded,
            network: opts.network,
            contracts: opts.contracts,
            verbose: opts.verbose,
            ..Config::default()
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
            network: RGB_NETWORK
                .parse()
                .expect("Error in RGB_NETWORK constant value"),
            verbose: 0,
        }
    }
}
