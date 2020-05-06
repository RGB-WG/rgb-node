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

use clap::Clap;
use lnpbp::bp;
use lnpbp::internet::InetSocketAddr;
use std::{fs, io, path::PathBuf};

use crate::commands::*;
use crate::constants::*;

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
#[clap(
    name = "kaleidoscope",
    version = "0.2.0",
    author = "Dr Maxim Orlovsky <orlovsky@pandoracore.com>, Alekos Filini <alekos.filini@gmail.com>",
    about = "Kaleidoscope: RGB command-line wallet utility"
)]
pub struct Opts {
    /// Sets verbosity level; can be used multiple times to increase verbosity
    #[clap(
        global = true,
        short,
        long,
        min_values = 0,
        max_values = 4,
        parse(from_occurrences)
    )]
    pub verbose: u8,

    /// Data directory for keeping information about keyrings, assets etc
    #[clap(global=true, long, default_value=DATA_DIR, env="KALEIDOSCOPE_DATA_DIR")]
    pub data_dir: PathBuf,

    /// Electrum server RPC endpoint
    #[clap(global=true, long, default_value=ELECTRUM_ENDPOINT, env="KALEIDOSCOPE_ELECTRUM_ENDPOINT")]
    pub electrum_endpoint: InetSocketAddr,

    /// IPC connection string for bp daemon API
    #[clap(global=true, long, default_value=BPD_API_ADDR, env="KALEIDOSCOPE_BPD_API")]
    pub bpd_api: String,

    /// IPC connection string for bp daemon push notifications on transaction
    /// updates
    #[clap(global=true, long, default_value=BPD_PUSH_ADDR, env="KALEIDOSCOPE_BPD_SUBSCR")]
    pub bpd_subscr: String,

    /// Network to use
    #[clap(
        global = true,
        short,
        long,
        default_value = "signet",
        env = "KALEIDOSCOPE_NETWORK"
    )]
    pub network: bp::Network,

    #[clap(subcommand)]
    pub command: Command,
}

// We need config structure since not all of the parameters can be specified
// via environment and command-line arguments. Thus we need a config file and
// default set of configuration
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Config {
    pub verbose: u8,
    pub network: bp::Network,
    pub data_dir: PathBuf,
    pub electrum_endpoint: InetSocketAddr,
    pub bpd_api: String,
    pub bpd_subscr: String,
}

impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        let data_str = opts.data_dir.to_str().unwrap();
        let repl = shellexpand::tilde(data_str);
        let data_dir = PathBuf::from(repl.into_owned());

        Self {
            verbose: opts.verbose,
            network: opts.network,
            data_dir: data_dir,
            electrum_endpoint: opts.electrum_endpoint,
            bpd_api: opts.bpd_api,
            bpd_subscr: opts.bpd_subscr,

            ..Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            verbose: 0,
            network: bp::Network::Signet,
            data_dir: DATA_DIR
                .parse()
                .expect("Parse of DATA_DIR constant has failed"),
            electrum_endpoint: ELECTRUM_ENDPOINT
                .parse()
                .expect("Parse of ELECTRUM_ENDPOINT onstant has failed"),
            bpd_api: BPD_API_ADDR.to_string(),
            bpd_subscr: BPD_PUSH_ADDR.to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
#[non_exhaustive]
pub enum DataItem {
    Root,
    KeyringVault,
    ContractsVault,
    //    ContractGenesis(ContractId),
    FungibleSeals,
}

impl Config {
    pub fn data_path(&self, item: DataItem) -> PathBuf {
        let mut path = self.data_dir.clone();
        match item {
            DataItem::Root => (),
            DataItem::KeyringVault => path.push("vault.dat"),
            DataItem::ContractsVault => {
                path.push("contracts");
                if !path.exists() {
                    fs::create_dir_all(path.clone()).unwrap();
                }
            }
            /*
            DataItem::ContractGenesis(cmt) => {
                path = self.data_path(DataItem::ContractsVault);
                path.push(format!("{}", cmt));
                path.set_extension("rgb");
            }
             */
            DataItem::FungibleSeals => path.push("fungible_seals.json"),
        }
        path
    }

    pub fn data_reader(&self, item: DataItem) -> Result<impl io::Read, io::Error> {
        let file_name = self.data_path(item);
        let file = fs::File::open(file_name)?;
        Ok(io::BufReader::new(file))
    }

    pub fn data_writer(&self, item: DataItem) -> Result<impl io::Write, io::Error> {
        let file_name = self.data_path(item);
        let file = fs::File::create(file_name)?;
        Ok(io::BufWriter::new(file))
    }
}
