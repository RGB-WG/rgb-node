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

use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::util::psbt::{raw::Key, PartiallySignedTransaction};
use bitcoin::OutPoint;

use lnpbp::bitcoin;
use lnpbp::bp::blind::OutpointReveal;
use lnpbp::client_side_validation::Conceal;
use lnpbp::data_format::DataFormat;
use lnpbp::rgb::prelude::*;

use super::{Error, OutputFormat, Runtime};
use crate::api::fungible::{AcceptApi, Issue, TransferApi};
use crate::api::{reply, Reply};
use crate::fungible::{Asset, Invoice, Outcoincealed, Outcoins, Outpoint};
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

        /// Locally-controlled outpoint (specified when the invoice was created)
        outpoint: OutPoint,

        /// Outpoint blinding factor (generated when the invoice was created)
        blinding_factor: u32,
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
    pub outpoint: OutPoint,
}

#[derive(Clap, Clone, PartialEq, Debug, Display)]
#[display_from(Debug)]
pub struct TransferCli {
    /// Asset inputs
    #[clap(short = "i", long = "input", min_values = 1)]
    pub inputs: Vec<OutPoint>,

    /// Adds additional asset allocations; MUST use transaction inputs
    /// controlled by the local party
    #[clap(short, long)]
    pub allocate: Vec<Outcoins>,

    /// Invoice to pay
    pub invoice: Invoice,

    /// Read partially-signed transaction prototype
    pub prototype: PathBuf,

    /// Fee (in satoshis)
    pub fee: u64,

    /// Change output
    pub change: OutPoint,

    /// File to save consignment to
    pub consignment: PathBuf,

    /// File to save updated partially-signed bitcoin transaction to
    pub transaction: PathBuf,
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
            Command::Accept {
                ref consignment,
                outpoint,
                blinding_factor,
            } => self.exec_accept(runtime, consignment.clone(), outpoint, blinding_factor),
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

    fn exec_accept(
        &self,
        mut runtime: Runtime,
        filename: PathBuf,
        outpoint: OutPoint,
        blinding_factor: u32,
    ) -> Result<(), Error> {
        info!("Accepting asset transfer...");

        debug!("Reading consignment from file {:?}", &filename);
        let consignment = Consignment::read_file(filename.clone()).map_err(|err| {
            Error::InputFileFormatError(format!("{:?}", filename), format!("{}", err))
        })?;

        let api = if let Some(outpoint_hash) = consignment.endpoints.get(0) {
            let outpoint_reveal = OutpointReveal {
                blinding: blinding_factor,
                txid: outpoint.txid,
                vout: outpoint.vout as u16,
            };
            if outpoint_reveal.conceal() != *outpoint_hash {
                eprintln!("The provided outpoint and blinding factors does not match outpoint from the consignment");
                Err(Error::DataInconsistency)?
            }
            AcceptApi {
                consignment,
                reveal_outpoints: vec![outpoint_reveal],
            }
        } else {
            eprintln!("Currently, this command-line tool is unable to accept consignments containing more than a single locally-controlled output point");
            Err(Error::UnsupportedFunctionality)?
        };

        match &*runtime.accept(api)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Success => {
                eprintln!("Asset transfer successfully accepted.");
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

        let outpoint_reveal = OutpointReveal::from(self.outpoint);
        let invoice = Invoice {
            contract_id: self.asset,
            outpoint: Outpoint::BlindedUtxo(outpoint_reveal.conceal()),
            amount: self.amount,
        };

        eprint!("Invoice: ");
        println!("{}", invoice);
        eprint!("Outpoint blinding factor: ");
        println!("{}", outpoint_reveal.blinding);

        Ok(())
    }
}

impl TransferCli {
    #[allow(unreachable_code)]
    pub fn exec(self, mut runtime: Runtime) -> Result<(), Error> {
        info!("Transferring asset ...");
        debug!("{}", self.clone());

        let seal_confidential = match self.invoice.outpoint {
            Outpoint::BlindedUtxo(outpoint_hash) => outpoint_hash,
            Outpoint::Address(_address) => {
                // To do a pay-to-address, we need to spend some bitcoins,
                // which we have to take from somewhere. While payee can
                // provide us with additional input, it's not part of the
                // invoicing protocol + does not make a lot of sense, since
                // the same input can be simply used by Utxo scheme
                unimplemented!();
                SealDefinition::WitnessVout {
                    vout: 0,
                    blinding: 0,
                }
                .conceal()
            }
        };

        let pubkey_key = Key {
            type_value: 0xFC,
            key: PSBT_PUBKEY_KEY.to_vec(),
        };
        let fee_key = Key {
            type_value: 0xFC,
            key: PSBT_FEE_KEY.to_vec(),
        };

        debug!(
            "Reading partially-signed transaction from file {:?}",
            self.prototype
        );
        let filepath = format!("{:?}", &self.prototype);
        let file = fs::File::open(self.prototype)
            .map_err(|_| Error::InputFileIoError(format!("{:?}", filepath)))?;
        let mut psbt = PartiallySignedTransaction::consensus_decode(file).map_err(|err| {
            Error::InputFileFormatError(format!("{:?}", filepath), format!("{}", err))
        })?;

        psbt.global
            .unknown
            .insert(fee_key, self.fee.to_be_bytes().to_vec());
        for output in &mut psbt.outputs {
            output.unknown.insert(
                pubkey_key.clone(),
                output.hd_keypaths.keys().next().unwrap().to_bytes(),
            );
        }
        trace!("{:?}", psbt);

        let api = TransferApi {
            psbt,
            contract_id: self.invoice.contract_id,
            inputs: self.inputs,
            ours: self.allocate,
            theirs: vec![Outcoincealed {
                coins: self.invoice.amount,
                seal_confidential,
            }],
            change: self.change,
        };

        // TODO: Do tx output reorg for deterministic ordering

        let reply = runtime.transfer(api)?;
        info!("Reply: {}", reply);
        match &*reply {
            Reply::Failure(failure) => {
                eprintln!("Transfer failed: {}", failure);
            }
            Reply::Transfer(transfer) => {
                transfer.consignment.write_file(self.consignment.clone())?;
                let out_file = fs::File::create(&self.transaction)
                    .expect("can't create output transaction file");
                transfer.psbt.consensus_encode(out_file)?;
                println!(
                    "Transfer succeeded, consignment data are written to {:?}, partially signed witness transaction to {:?}",
                    self.consignment, self.transaction
                );
            }
            _ => (),
        }

        Ok(())
    }
}
