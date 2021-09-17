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

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::util::psbt::raw::ProprietaryKey;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::OutPoint;
use bp::seals::{OutpointHash, OutpointReveal};
use commit_verify::CommitConceal;
use microservices::FileFormat;
use rgb::{
    AllocatedValue, Genesis, ContractId, AtomicValue, Consignment, Disclosure,
    PSBT_OUT_PUBKEY, SealEndpoint
};
use rgb20::Asset;
use strict_encoding::strict_deserialize;

use super::{Error, OutputFormat, Runtime};
use crate::rpc::fungible::{AcceptReq, IssueReq, TransferReq};
use crate::rpc::{reply, Reply};
use crate::util::file::ReadWrite;

#[derive(Clap, Clone, Debug, Display)]
#[display(Debug)]
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
        /// Bech32 representation of the asset ID (contract id of the asset
        /// genesis)
        #[clap(parse(try_from_str = ContractId::from_bech32_str))]
        asset: ContractId,
    },

    /// Creates a new asset
    Issue(IssueReq),

    /// Creates a blinded version of a given bitcoin transaction outpoint
    Blind {
        /// Original outpoint in `txid:vout` format
        outpoint: OutPoint,
    },

    /// Do a transfer of some requested asset to another party
    Transfer(TransferCli),

    /// Do a transfer of some requested asset to another party
    Validate {
        /// Consignment file
        consignment: PathBuf,
    },

    /// Accepts an incoming payment
    Accept {
        /// Consignment file
        consignment: PathBuf,

        /// Locally-controlled outpoint (specified when the invoice was
        /// created)
        outpoint: OutPoint,

        /// Outpoint blinding factor (generated when the invoice was created)
        blinding_factor: u64,
    },

    /// Adds data from some disclosure to the stash & asset information cache
    Enclose {
        /// Path to disclosure file
        disclosure: PathBuf,
    },

    Forget {
        /// Bitcoin transaction output that was spent and which data
        /// has to be forgotten
        outpoint: OutPoint,
    },
}

#[derive(Clap, Clone, PartialEq, Debug, Display)]
#[display(Debug)]
pub struct TransferCli {
    /// Asset inputs
    #[clap(short = 'i', long = "input", min_values = 1)]
    pub inputs: Vec<OutPoint>,

    /// Adds additional asset allocations; MUST use transaction inputs
    /// controlled by the local party
    #[clap(short, long)]
    pub allocate: Vec<AllocatedValue>,

    /// Whom to pay
    pub receiver: OutpointHash,

    /// Amount to pay, in atomic (non-float) units
    pub amount: AtomicValue,

    /// Which asset to use for the payment
    pub asset: ContractId,

    /// Read partially-signed transaction prototype
    pub prototype: PathBuf,

    /// File to save consignment to
    pub consignment: PathBuf,

    /// File to save disclosure to.
    ///
    /// Disclosures are used to allocate the change and other assets which were
    /// on the same output but were not transferred. To see the change and
    /// those assets you will have to accept disclosure lately with special
    /// `enclose` command.
    pub disclosure: PathBuf,

    /// File to save updated partially-signed bitcoin transaction to
    pub transaction: PathBuf,
}

impl Command {
    pub fn exec(self, runtime: Runtime) -> Result<(), Error> {
        match self {
            Command::List { format, long } => {
                self.exec_list(runtime, format, long)
            }
            Command::Import { ref asset } => {
                self.exec_import(runtime, asset.clone())
            }
            Command::Export { asset } => self.exec_export(runtime, asset),
            Command::Blind { outpoint } => {
                info!("Blinding outpoint ...");
                let outpoint_reveal = OutpointReveal::from(outpoint);
                eprint!("Blinded outpoint: ");
                println!("{}", outpoint_reveal.commit_conceal());
                eprint!("Outpoint blinding secret: ");
                println!("{}", outpoint_reveal.blinding);
                Ok(())
            }
            Command::Issue(issue) => issue.exec(runtime),
            Command::Transfer(transfer) => transfer.exec(runtime),
            Command::Validate { ref consignment } => {
                self.exec_validate(runtime, consignment.clone())
            }
            Command::Accept {
                ref consignment,
                outpoint,
                blinding_factor,
            } => self.exec_accept(
                runtime,
                consignment.clone(),
                outpoint,
                blinding_factor,
            ),
            Command::Enclose { ref disclosure } => {
                self.exec_enclose(runtime, disclosure.clone())
            }
            Command::Forget { outpoint } => self.exec_forget(runtime, outpoint),
        }
    }

    fn exec_list(
        &self,
        mut runtime: Runtime,
        output_format: OutputFormat,
        long: bool,
    ) -> Result<(), Error> {
        match &*runtime.list(output_format)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Sync(reply::SyncFormat(input_format, data)) => {
                let assets: Vec<Asset> = match input_format {
                    FileFormat::Yaml => serde_yaml::from_slice(&data)?,
                    FileFormat::Json => serde_json::from_slice(&data)?,
                    FileFormat::Toml => toml::from_slice(&data)?,
                    FileFormat::StrictEncode => strict_deserialize(&data)?,
                    _ => unimplemented!(),
                };
                let short: Vec<HashMap<&str, String>> = assets
                    .iter()
                    .map(|a| {
                        map! {
                            "id" => a.id().to_string(),
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

    fn exec_import(
        &self,
        mut runtime: Runtime,
        genesis: Genesis,
    ) -> Result<(), Error> {
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

    fn exec_export(
        &self,
        mut runtime: Runtime,
        asset_id: ContractId,
    ) -> Result<(), Error> {
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

    fn exec_validate(
        &self,
        mut runtime: Runtime,
        filename: PathBuf,
    ) -> Result<(), Error> {
        info!("Validating asset transfer...");

        debug!("Reading consignment from file {:?}", &filename);
        let consignment =
            Consignment::read_file(filename.clone()).map_err(|err| {
                Error::InputFileFormatError(
                    format!("{:?}", filename),
                    format!("{}", err),
                )
            })?;
        trace!("{:#?}", consignment);

        match &*runtime.validate(consignment)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::ValidationStatus(status) => {
                eprintln!("Asset transfer validation report:\n{:#?}", status);
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
        blinding_factor: u64,
    ) -> Result<(), Error> {
        info!("Accepting asset transfer...");

        debug!("Reading consignment from file {:?}", &filename);
        let consignment =
            Consignment::read_file(filename.clone()).map_err(|err| {
                Error::InputFileFormatError(
                    format!("{:?}", filename),
                    format!("{}", err),
                )
            })?;
        trace!("{:#?}", consignment);

        let api = if let Some((_, seal_endpoint)) = consignment.endpoints.get(0)
        {
            let outpoint_reveal = OutpointReveal {
                blinding: blinding_factor,
                txid: outpoint.txid,
                vout: outpoint.vout as u32,
            };
            if outpoint_reveal.commit_conceal()
                != seal_endpoint.commit_conceal()
            {
                eprintln!(
                    "The provided outpoint and blinding factors does not match \
                    outpoint from the consignment"
                );
                Err(Error::DataInconsistency)?
            }
            AcceptReq {
                consignment,
                reveal_outpoints: vec![outpoint_reveal],
            }
        } else {
            eprintln!(
                "Currently, this command-line tool is unable to accept \
                consignments containing more than a single locally-controlled \
                output point"
            );
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

    fn exec_enclose(
        &self,
        mut runtime: Runtime,
        filename: PathBuf,
    ) -> Result<(), Error> {
        info!("Enclosing disclosure...");

        debug!("Reading disclosure from file {:?}", &filename);
        let disclosure =
            Disclosure::read_file(filename.clone()).map_err(|err| {
                Error::InputFileFormatError(
                    format!("{:?}", filename),
                    format!("{}", err),
                )
            })?;
        trace!("{:#?}", disclosure);

        match &*runtime.enclose(disclosure)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Success => {
                eprintln!("Disclosure data successfully enclosed.");
            }
            _ => {
                eprintln!(
                    "Unexpected server error; probably you connecting with \
                    outdated client version"
                );
            }
        }

        Ok(())
    }

    fn exec_forget(
        &self,
        mut runtime: Runtime,
        outpoint: OutPoint,
    ) -> Result<(), Error> {
        info!(
            "Forgetting assets allocated to specific bitcoin transaction output that was spent..."
        );

        match &*runtime.forget(outpoint)? {
            Reply::Failure(failure) => {
                eprintln!("Server returned error: {}", failure);
            }
            Reply::Success => {
                eprintln!("Assets are removed from the stash.");
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

impl IssueReq {
    pub fn exec(self, mut runtime: Runtime) -> Result<(), Error> {
        info!("Issuing asset ...");
        debug!("{}", self.clone());

        let reply = runtime.issue(self)?;
        info!("Reply: {}", reply);

        let asset = match &*reply {
            Reply::Failure(failure) => {
                eprintln!("Issue failed: {}", failure);
                return Ok(());
            }
            Reply::Asset(asset) => asset,
            _ => {
                eprintln!("Unrecognized RGB node reply");
                Err(Error::DataInconsistency)?
            }
        };

        eprintln!(
            "Asset successfully issued. Use this information for sharing:"
        );
        #[cfg(feature = "serde")]
        eprintln!(
            "Asset information:\n {}\n",
            serde_yaml::to_string(asset)
                .expect("broken asset YAML serialization")
        );
        println!("{}", asset.genesis());

        Ok(())
    }
}

impl TransferCli {
    #[allow(unreachable_code)]
    pub fn exec(self, mut runtime: Runtime) -> Result<(), Error> {
        info!("Transferring asset ...");
        debug!("{}", self.clone());

        debug!(
            "Reading partially-signed transaction from file {:?}",
            self.prototype
        );
        let filepath = format!("{:?}", &self.prototype);
        let file = fs::File::open(self.prototype)
            .map_err(|_| Error::InputFileIoError(format!("{:?}", filepath)))?;
        let mut psbt = PartiallySignedTransaction::consensus_decode(file)
            .map_err(|err| {
                Error::InputFileFormatError(
                    format!("{:?}", filepath),
                    format!("{}", err),
                )
            })?;

        for (index, output) in &mut psbt.outputs.iter_mut().enumerate() {
            if let Some(key) = output.bip32_derivation.keys().next() {
                let key = key.clone();
                output.proprietary.insert(
                    ProprietaryKey {
                        prefix: b"RGB".to_vec(),
                        subtype: PSBT_OUT_PUBKEY,
                        key: vec![],
                    },
                    key.key.serialize().to_vec(),
                );
                debug!("Output #{} commitment key will be {}", index, key);
            } else {
                warn!(
                    "No public key information found for output #{}; \
                    LNPBP1/2 commitment will be impossible.\
                    In order to allow commitment pls add known keys derivation \
                    information to PSBT output map",
                    index
                );
            }
        }
        trace!("{:?}", psbt);

        let api = TransferReq {
            witness: psbt,
            contract_id: self.asset,
            inputs: self.inputs.into_iter().collect(),
            change: self
                .allocate
                .into_iter()
                .map(|seal_coins| {
                    (seal_coins.seal_definition(), seal_coins.coins)
                })
                .collect(),
            payment: bmap! { SealEndpoint::TxOutpoint(self.receiver) => self.amount },
        };

        let reply = runtime.transfer(api)?;
        info!("Reply: {}", reply);
        match &*reply {
            Reply::Failure(failure) => {
                eprintln!("Transfer failed: {}", failure);
            }
            Reply::Transfer(transfer) => {
                transfer.disclosure.write_file(&self.disclosure)?;
                transfer.consignment.write_file(&self.consignment)?;

                let out_file = fs::File::create(&self.transaction)
                    .expect("can't create output transaction file");
                transfer.witness.consensus_encode(out_file).map_err(|err| {
                    bitcoin::consensus::encode::Error::Io(err)
                })?;

                eprintln!(
                    "Transfer succeeded, consignments and disclosure are written \
                     to {:?} and {:?}, partially signed witness transaction to {:?}",
                    self.consignment, self.disclosure, self.transaction
                );
                eprint!("Consignment data to share:");
                println!("{}", transfer.consignment);
            }
            _ => (),
        }

        Ok(())
    }
}
