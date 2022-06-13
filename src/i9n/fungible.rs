// RGB standard library
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use bitcoin::util::psbt::raw::ProprietaryKey;
use bitcoin::OutPoint;
use internet2::{Session, TypedEnum, Unmarshall};
use lnpbp::chain::Chain;
use microservices::FileFormat;
use rgb::{
    seal, AtomicValue, Consignment, ContractId, Disclosure, Genesis, OutpointValue, SealEndpoint,
    PSBT_OUT_PUBKEY,
};
use rgb20::Asset;
use stens::AsciiString;
use wallet::psbt::Psbt;

use super::{Error, Runtime};
use crate::error::ServiceErrorDomain;
use crate::rpc::fungible::{AcceptReq, IssueReq, Request, TransferReq};
use crate::rpc::reply::Transfer;
use crate::rpc::{reply, Reply};

impl Runtime {
    fn command(&mut self, command: Request) -> Result<Arc<Reply>, ServiceErrorDomain> {
        let data = command.serialize();
        self.session_rpc.send_raw_message(&data)?;
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.unmarshaller.unmarshall(&*raw)?;
        Ok(reply)
    }

    pub fn issue(
        &mut self,
        chain: Chain,
        ticker: AsciiString,
        name: AsciiString,
        description: Option<String>,
        precision: u8,
        allocation: Vec<OutpointValue>,
        inflation: Vec<OutpointValue>,
        renomination: Option<OutPoint>,
        epoch: Option<OutPoint>,
    ) -> Result<Asset, Error> {
        if self.config.network != chain {
            Err(Error::WrongNetwork)?;
        }
        let command = Request::Issue(IssueReq {
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
            Reply::Asset(asset) => Ok(asset.clone()),
            Reply::Failure(failmsg) => Err(Error::Reply(failmsg.clone())),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn transfer(
        &mut self,
        contract_id: ContractId,
        inputs: BTreeSet<OutPoint>,
        payment: BTreeMap<SealEndpoint, AtomicValue>,
        change: BTreeMap<seal::Revealed, AtomicValue>,
        mut witness: Psbt,
    ) -> Result<Transfer, Error> {
        for (index, output) in &mut witness.outputs.iter_mut().enumerate() {
            if let Some(key) = output.bip32_derivation.keys().next() {
                let key = key.clone();
                output.proprietary.insert(
                    ProprietaryKey {
                        prefix: b"RGB".to_vec(),
                        subtype: PSBT_OUT_PUBKEY,
                        key: vec![],
                    },
                    key.serialize().to_vec(),
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
        trace!("{:?}", witness);

        let api = TransferReq {
            witness,
            contract_id,
            inputs,
            payment,
            change,
        };

        match &*self.command(Request::Transfer(api))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Transfer(transfer) => {
                info!("Transfer succeeded");

                Ok(transfer.clone())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn accept(
        &mut self,
        consignment: Consignment,
        reveal_outpoints: Vec<seal::Revealed>,
    ) -> Result<(), Error> {
        let api = AcceptReq {
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

    pub fn validate(&mut self, consignment: Consignment) -> Result<rgb::validation::Status, Error> {
        match &*self.command(Request::Validate(consignment))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::ValidationStatus(status) => {
                info!("Validation succeeded");
                Ok(status.clone())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn enclose(&mut self, disclosure: Disclosure) -> Result<(), Error> {
        match &*self.command(Request::Enclose(disclosure))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Success => {
                info!("Enclose command succeeded");
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
            Reply::AssetAllocations(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn outpoint_assets(
        &mut self,
        outpoint: OutPoint,
    ) -> Result<BTreeMap<ContractId, Vec<AtomicValue>>, Error> {
        match &*self.command(Request::Assets(outpoint))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::OutpointAssets(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn export_asset(&mut self, asset_id: ContractId) -> Result<Genesis, Error> {
        match &*self.command(Request::ExportAsset(asset_id))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Genesis(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn import_asset(&mut self, genesis: Genesis) -> Result<Asset, Error> {
        match &*self.command(Request::ImportAsset(genesis))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Asset(asset) => {
                info!("Asset import succeeded");
                Ok(asset.clone())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn list_assets(&mut self, data_format: FileFormat) -> Result<reply::SyncFormat, Error> {
        match &*self.command(Request::Sync(data_format))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Sync(response) => Ok(response.clone()),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}
