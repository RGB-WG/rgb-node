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
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use bitcoin::consensus::Decodable;
use bitcoin::hashes::hex::FromHex;
use bitcoin::util::psbt::{self, PartiallySignedTransaction};
use bitcoin::{OutPoint, Transaction, TxIn};

use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::data_format::DataFormat;
use lnpbp::rgb::prelude::*;

use super::{Error, OutputFormat, Runtime};
use crate::api::fungible::{Issue, TransferApi};
use crate::api::{reply, Reply};
use crate::fungible::{Asset, Invoice, Outcoincealed, Outcoins, OutpointDescriptor};
use crate::util::file::ReadWrite;

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

    /// Create an invoice
    Invoice(InvoiceCli),

    /// Do a transfer of some requested asset to another party
    Transfer(TransferCli),

    /// Accepts an incoming payment
    Accept {
        /// Consignment file
        consignment: PathBuf,
    },
}

#[derive(Clap, Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct InvoiceCli {
    /// Assets
    pub asset: ContractId,

    /// Amount
    pub amount: f32,

    /// Receive assets to a given bitcoin address or UTXO
    pub outpoint: OutpointDescriptor,
}

#[derive(Clap, Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct TransferCli {
    /// Use custom commitment output for generated witness transaction
    #[clap(long)]
    pub commit_txout: Option<Output>,

    /// Read partially-signed transaction prototype
    #[clap(short, long)]
    pub prototype: Option<PathBuf>,

    /// Asset inputs
    #[clap(short = "i", long, min_values = 1)]
    pub inputs: Vec<OutPoint>,

    /// Adds custom (non-RGB) output(s) to generated witness transaction
    #[clap(long)]
    pub txout: Vec<Output>,

    /// Adds custom (non-RGB) input(s) to generated witness transaction
    #[clap(long)]
    pub txin: Vec<Input>,

    /// Fee (in satoshis), if the PSBT transaction prototype is not provided
    #[clap(short, long)]
    pub fee: Option<u64>,

    /// Adds additional asset allocations; MUST use transaction inputs
    /// controlled by the local party
    #[clap(short, long)]
    pub allocate: Vec<Outcoins>,

    /// Amount
    pub amount: f32,

    /// Assets
    pub contract_id: ContractId,

    /// Receiver
    #[clap(parse(try_from_str=bp::blind::OutpointHash::from_hex))]
    pub receiver: bp::blind::OutpointHash,

    /// Change output
    pub change: Option<OutPoint>,

    /// File to save consignment to
    pub consignment: PathBuf,

    /// File to save updated partially-signed bitcoin transaction to
    pub transaction: PathBuf,
    // / Invoice to pay
    //pub invoice: fungible::Invoice,
}

impl Command {
    pub fn exec(self, runtime: Runtime) -> Result<(), Error> {
        match self {
            Command::List { format, long } => self.exec_list(runtime, format, long),
            Command::Import { ref asset } => self.exec_import(runtime, asset.clone()),
            Command::Export { asset } => self.exec_export(runtime, asset),
            Command::Invoice(invoice) => invoice.exec(runtime),
            Command::Issue(issue) => issue.exec(runtime),
            Command::Transfer(transfer) => transfer.exec(runtime),
            _ => unimplemented!(),
        }
    }

    fn exec_list(
        &self,
        mut runtime: Runtime,
        output_format: OutputFormat,
        long: bool,
    ) -> Result<(), Error> {
        match &*runtime.list()? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
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
                            "id" => a.id().to_bech32_string(),
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
                eprintln!(
                    "Unexpected server error; probably you connecting with outdated client version"
                );
            }
        }
        Ok(())
    }

    fn exec_import(&self, mut runtime: Runtime, genesis: Genesis) -> Result<(), Error> {
        info!("Importing asset ...");

        match &*runtime.import(genesis)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Success => {
                eprintln!("Asset successfully imported");
            }
            _ => {
                eprintln!(
                    "Unexpected server error; probably you connecting with outdated client version"
                );
            }
        }
        Ok(())
    }

    fn exec_export(&self, mut runtime: Runtime, asset_id: ContractId) -> Result<(), Error> {
        info!("Exporting asset ...");

        match &*runtime.export(asset_id)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Genesis(genesis) => {
                eprintln!("Asset successfully exported. Use this information for sharing:");
                println!("{}", genesis);
            }
            _ => {
                eprintln!(
                    "Unexpected server error; probably you connecting with outdated client version"
                );
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

        /*let (asset, genesis) = match reply {

        };

        debug!("Asset information:\n {:?}\n", asset);
        trace!("Genesis contract:\n {:?}\n", genesis);

        eprintln!("Asset successfully issued. Use this information for sharing:");
        println!("{}", genesis);*/

        Ok(())
    }
}

impl InvoiceCli {
    pub fn exec(self, _: Runtime) -> Result<(), Error> {
        info!("Generating invoice ...");
        debug!("{}", self.clone());

        let invoice = Invoice {
            contract_id: self.asset,
            outpoint: self.outpoint.into(),
            amount: self.amount,
        };

        println!("{}", invoice);

        Ok(())
    }
}

impl TransferCli {
    pub fn exec(self, mut runtime: Runtime) -> Result<(), Error> {
        info!("Transferring asset ...");
        debug!("{}", self.clone());

        let psbt = match self.prototype {
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
        match &*reply {
            Reply::Failure(failure) => {
                eprintln!("Transfer failed: {}", failure);
            }
            Reply::Transfer(transfer) => {
                transfer.consignment.write_file(self.consignment.clone())?;
                println!(
                    "Transfer succeeded, consignment data are written to {:?}",
                    self.consignment
                );
            }
            _ => (),
        }

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
