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
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use bitcoin::consensus::Decodable;
use bitcoin::hashes::hex::FromHex;
use bitcoin::util::psbt::{self, PartiallySignedTransaction};
use bitcoin::{OutPoint, Transaction, TxIn};

use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::rgb::prelude::*;

use super::{Error, OutputFormat, Runtime};
use crate::api::fungible::{Issue, TransferApi};
use crate::api::{reply, Reply};
use crate::fungible::{Asset, Outcoincealed, Outcoins};

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Lists all known assets
    List {
        /// Format for information output
        #[clap(short, long, arg_enum, default_value = "yaml")]
        format: OutputFormat,

        /// List all asset details
        #[clap(short, long)]
        long: bool,
    },

    Import {
        /// Bech32 representation of the asset genesis
        asset: Genesis,
    },

    Export {
        /// Bech32 representation of the asset ID (contract id of the asset genesis)
        asset: ContractId,
    },

    /// Creates a new asset
    Issue(Issue),

    /// Transfers some asset to another party
    Transfer(TransferCli),
}

#[derive(Clap, Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct TransferCli {
    /// Use custom commitment output for generated witness transaction
    #[clap(long)]
    pub commit_txout: Option<Output>,

    /// Read pastially-signed transaction prototype
    #[clap(short, long)]
    pub psbt: Option<PathBuf>,

    /// Asset inputs
    #[clap(short = "i", long, min_values = 1)]
    pub inputs: Vec<OutPoint>,

    /// Adds output(s) to generated witness transaction
    #[clap(long)]
    pub txout: Vec<Output>,

    /// Adds input(s) to generated witness transaction
    #[clap(long)]
    pub txin: Vec<Input>,

    /// Adds additional asset allocations; MUST use transaction inputs
    /// controlled by the local party
    #[clap(short, long)]
    pub allocate: Vec<Outcoins>,

    /// Saves witness transaction to a file instead of publishing it
    #[clap(short, long)]
    pub transaction: Option<PathBuf>,

    /// Saves consignment data to a file
    #[clap(long)]
    pub consignment: Option<PathBuf>,

    /// Amount
    pub amount: f32,

    /// Assets
    #[clap(parse(try_from_str=ContractId::from_hex))]
    pub contract_id: ContractId,

    /// Receiver
    #[clap(parse(try_from_str=bp::blind::OutpointHash::from_hex))]
    pub receiver: bp::blind::OutpointHash,

    /// Change output
    pub change: Option<OutPoint>,
    // / Invoice to pay
    //pub invoice: fungible::Invoice,
}

impl Command {
    pub fn exec(self, runtime: Runtime) -> Result<(), Error> {
        match self {
            Command::List { format, long } => self.exec_list(runtime, format, long),
            Command::Issue(issue) => issue.exec(runtime),
            Command::Transfer(transfer) => transfer.exec(runtime),
            _ => unimplemented!(),
        }
    }

    fn exec_list(
        self,
        mut runtime: Runtime,
        output_format: OutputFormat,
        long: bool,
    ) -> Result<(), Error> {
        match runtime.list() {
            Ok(reply) => match reply.borrow() {
                Reply::Sync(reply::SyncFormat(input_format, data)) => {
                    let assets: Vec<Asset> = match input_format {
                        DataFormat::Yaml => serde_yaml::from_slice(&data)?,
                        DataFormat::Json => serde_json::from_slice(&data)?,
                        DataFormat::Toml => toml::from_slice(&data)?,
                        DataFormat::StrictEncode => unimplemented!(),
                    };
                    let short: Vec<HashMap<&str, String>> = assets
                        .iter()
                        .map(|a| {
                            map! {
                                "id" => format!("{}", a.id()).to_string(),
                                "ticker" => a.ticker().clone(),
                                "name" => a.name().clone()
                            }
                        })
                        .collect();
                    let long_str: String;
                    let short_str: String;
                    match output_format {
                        OutputFormat::Yaml => {
                            long_str = serde_yaml::to_string(&assets)?;
                            short_str = serde_yaml::to_string(&short)?;
                        }
                        OutputFormat::Json => {
                            long_str = serde_json::to_string(&assets)?;
                            short_str = serde_json::to_string(&short)?;
                        }
                        OutputFormat::Toml => {
                            long_str = toml::to_string(&assets)?;
                            short_str = toml::to_string(&short)?;
                        }
                        _ => unimplemented!(),
                    }
                    if long {
                        println!("{}", long_str);
                    } else {
                        println!("{}", short_str);
                    }
                }

                _ => {
                    eprintln!("Unexpected server error; probably you connecting with outdated client version");
                }
            },
            Err(err) => {
                eprintln!("Server returned error: {}\n", err);
            }
        }
        Ok(())
    }
}

impl Issue {
    pub fn exec(self, mut runtime: Runtime) -> Result<(), Error> {
        info!("Issuing asset ...");
        debug!("{}", self.clone());

        let reply = runtime.issue(self)?;
        info!("Reply: {}", reply);
        // TODO: Wait for the information from push notification

        /*let (asset, genesis) = debug!("Asset information:\n {}\n", asset);
        trace!("Genesis contract:\n {}\n", genesis);

        let bech = bech32::encode(
            crate::RGB_BECH32_HRP_GENESIS,
            strict_encode(&genesis).to_base32(),
        )
        .unwrap();
        info!(
            "Use this string to send information about the issued asset:\n{}\n",
            bech
        );
         */

        Ok(())
    }
}

impl TransferCli {
    pub fn exec(self, mut runtime: Runtime) -> Result<(), Error> {
        info!("Transferring asset ...");
        debug!("{}", self.clone());

        let psbt = match self.psbt {
            Some(filename) => {
                debug!(
                    "Reading partially-signed transaction from file {:?}",
                    filename
                );
                let filepath = format!("{:?}", filename.clone());
                let file = fs::File::open(filename)
                    .map_err(|_| Error::InputFileIoError(format!("{:?}", filepath)))?;
                let psbt = PartiallySignedTransaction::consensus_decode(file).map_err(|err| {
                    Error::InputFileFormatError(format!("{:?}", filepath), format!("{}", err))
                })?;
                trace!("{:?}", psbt);
                psbt
            }
            None => {
                debug!("Generating transaction from arguments");
                let tx = Transaction {
                    version: 2,
                    lock_time: 0,
                    input: vec![],
                    output: vec![],
                };
                // TODO: Add addition of custom tx inputs and outputs from
                //       command-line arguments
                let psbt = PartiallySignedTransaction {
                    global: psbt::Global {
                        unsigned_tx: tx,
                        unknown: Default::default(),
                    },
                    inputs: vec![],
                    outputs: vec![],
                };
                trace!("{:?}", psbt);
                psbt
            }
        };
        let api = TransferApi {
            psbt,
            contract_id: self.contract_id,
            inputs: self.inputs,
            ours: self.allocate,
            theirs: vec![Outcoincealed {
                coins: self.amount,
                seal_confidential: self.receiver,
            }],
            change: self.change,
        };

        let reply = runtime.transfer(api)?;
        info!("Reply: {}", reply);

        // TODO: Wait for the information from push notification

        Ok(())
    }
}

// Helper data structures

mod helpers {
    use super::*;
    use core::str::FromStr;

    /// Defines information required to generate bitcoin transaction output from
    /// command-line argument
    #[derive(Clone, PartialEq, Debug, Display)]
    #[display_from(Debug)]
    pub struct Output {
        pub amount: bitcoin::Amount,
        pub lock: bp::LockScript,
    }

    impl FromStr for Output {
        type Err = String;
        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            unimplemented!()
        }
    }

    /// Defines information required to generate bitcoin transaction input from
    /// command-line argument
    #[derive(Clone, PartialEq, Debug, Display)]
    #[display_from(Debug)]
    pub struct Input {
        pub txin: TxIn,
        pub unlock: bp::LockScript,
    }

    impl FromStr for Input {
        type Err = String;
        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            unimplemented!()
        }
    }
}
pub use helpers::*;
use lnpbp::data_format::DataFormat;
