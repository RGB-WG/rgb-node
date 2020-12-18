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

use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use lnpbp::bitcoin::consensus::encode::{deserialize, Encodable};
use lnpbp::bitcoin::util::psbt::PartiallySignedTransaction;
use lnpbp::bitcoin::OutPoint;
use lnpbp::bp;
use lnpbp::bp::psbt::ProprietaryKeyMap;
use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::{Session, Unmarshall};
use lnpbp::rgb::{
    AtomicValue, Consignment, ContractId, Genesis, PSBT_OUT_PUBKEY,
};

use super::{Error, Runtime};
use crate::api::{
    fungible::AcceptApi, fungible::Issue, fungible::Request,
    fungible::TransferApi, reply, Reply,
};
use crate::error::ServiceErrorDomain;
use crate::fungible::{
    ConsealCoins, Invoice, Outpoint, OutpointCoins, SealCoins,
};
use crate::util::file::ReadWrite;
use crate::DataFormat;

impl Runtime {
    fn command(
        &mut self,
        command: Request,
    ) -> Result<Arc<Reply>, ServiceErrorDomain> {
        let data = command.encode()?;
        self.session_rpc.send_raw_message(&data)?;
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.unmarshaller.unmarshall(&raw)?;
        Ok(reply)
    }

    pub fn issue(
        &mut self,
        chain: bp::Chain,
        ticker: String,
        name: String,
        description: Option<String>,
        precision: u8,
        allocation: Vec<OutpointCoins>,
        inflation: Vec<OutpointCoins>,
        renomination: Option<OutPoint>,
        epoch: Option<OutPoint>,
    ) -> Result<(), Error> {
        if self.config.network != chain {
            Err(Error::WrongNetwork)?;
        }
        let command = Request::Issue(Issue {
            ticker,
            name,
            description,
            precision,
            allocation,
            inflation,
            renomination,
            epoch,
        });
        match &*self.command(command)? {
            Reply::Success => Ok(()),
            Reply::Failure(failmsg) => Err(Error::Reply(failmsg.clone())),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn transfer(
        &mut self,
        inputs: Vec<OutPoint>,
        allocate: Vec<SealCoins>,
        invoice: Invoice,
        prototype_psbt: String,
        consignment_file: String,
        transaction_file: String,
    ) -> Result<(), Error> {
        let seal_confidential = match invoice.outpoint {
            Outpoint::BlindedUtxo(outpoint_hash) => outpoint_hash,
            Outpoint::Address(_address) => unimplemented!(),
        };

        let psbt_bytes = base64::decode(&prototype_psbt)?;
        let mut psbt: PartiallySignedTransaction = deserialize(&psbt_bytes)?;

        for (index, output) in &mut psbt.outputs.iter_mut().enumerate() {
            if let Some(key) = output.hd_keypaths.keys().next() {
                let key = key.clone();
                output.insert_proprietary_key(
                    b"RGB".to_vec(),
                    PSBT_OUT_PUBKEY,
                    vec![],
                    &key.key,
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

        let api = TransferApi {
            psbt,
            contract_id: invoice.contract_id,
            inputs,
            ours: allocate,
            theirs: vec![ConsealCoins {
                coins: invoice.amount,
                seal_confidential,
            }],
        };

        match &*self.command(Request::Transfer(api))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Transfer(transfer) => {
                transfer
                    .consignment
                    .write_file(PathBuf::from(&consignment_file))?;
                let out_file = File::create(&transaction_file)
                    .expect("can't create output transaction file");
                transfer.psbt.consensus_encode(out_file)?;
                info!(
                    "Transfer succeeded, consignment data are written to {:?}, partially signed witness transaction to {:?}",
                    consignment_file, transaction_file
                );

                Ok(())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn accept(
        &mut self,
        consignment: Consignment,
        reveal_outpoints: Vec<bp::blind::OutpointReveal>,
    ) -> Result<(), Error> {
        let api = AcceptApi {
            consignment,
            reveal_outpoints,
        };

        match &*self.command(Request::Accept(api))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Success => {
                info!("Accept command succeeded");
                Ok(())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn validate(&mut self, consignment: Consignment) -> Result<(), Error> {
        match &*self.command(Request::Validate(consignment))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Success => {
                info!("Validation succeeded");
                Ok(())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn asset_allocations(
        &mut self,
        contract_id: ContractId,
    ) -> Result<BTreeMap<OutPoint, Vec<AtomicValue>>, Error> {
        match &*self.command(Request::Allocations(contract_id))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Allocations(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn outpoint_assets(
        &mut self,
        outpoint: OutPoint,
    ) -> Result<BTreeMap<ContractId, Vec<AtomicValue>>, Error> {
        match &*self.command(Request::Assets(outpoint))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Assets(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn export_asset(
        &mut self,
        asset_id: ContractId,
    ) -> Result<Genesis, Error> {
        match &*self.command(Request::ExportAsset(asset_id))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Genesis(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn import_asset(&mut self, genesis: Genesis) -> Result<(), Error> {
        match &*self.command(Request::ImportAsset(genesis))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Success => {
                info!("Asset import succeeded");
                Ok(())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn list_assets(
        &mut self,
        data_format: DataFormat,
    ) -> Result<reply::SyncFormat, Error> {
        match &*self.command(Request::Sync(data_format))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Sync(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}
