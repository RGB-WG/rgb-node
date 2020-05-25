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

use super::Runtime;
use crate::BootstrapError;

const RGB_BIN_DIR: &'static str = "/usr/local/bin";
const RGB_CONTRACTS: &'static str = "fungible";

#[derive(Clap)]
#[clap(
    name = "rgbd",
    version = "0.1.0",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>",
    about = "RGB main daemon; part of RGB suite"
)]
pub struct Opts {
    /// Path to the directory containing daemon executables
    #[clap(long, default_value = RGB_BIN_DIR, env = "RGB_BIN_DIR")]
    pub bin_dir: PathBuf,

    /// Contract daemons to launch
    #[clap(arg_enum, long, default_value = RGB_CONTRACTS, env = "RGB_CONTRACTS")]
    pub contract: Vec<ContractName>,

    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(
        short = "v",
        long = "verbose",
        min_values = 0,
        max_values = 4,
        parse(from_occurrences)
    )]
    pub verbose: u8,

    /// Bitcoin network to use
    #[clap(default_value = "bitcoin", env = "RGB_NETWORK")]
    pub network: bp::Network,
}

#[derive(Clap, Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
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
    pub verbose: u8,
    pub bin_dir: PathBuf,
    pub contract: Vec<ContractName>,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Self {
            verbose: opts.verbose,
            bin_dir: opts.bin_dir,
            contract: opts.contract,
            ..Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            bin_dir: RGB_BIN_DIR
                .parse()
                .expect("Error in RGB_BIN_DIR constant value"),
            contract: vec![ContractName::from_str(RGB_CONTRACTS, false)
                .expect("Error in RGB_CONTRACTS constant value")],
        }
    }
}
