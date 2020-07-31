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

use ::std::sync::Arc;
use std::fs::File;
use std::path::PathBuf;

use lnpbp::bitcoin::consensus::encode::{deserialize, Encodable};
use lnpbp::bitcoin::util::psbt::{raw::Key, PartiallySignedTransaction};
use lnpbp::bitcoin::OutPoint;

use lnpbp::bp;
use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::Unmarshall;
use lnpbp::rgb::{Amount, PSBT_FEE_KEY, PSBT_PUBKEY_KEY};

use super::{Error, Runtime};
use crate::api::{fungible::Issue, fungible::Request, fungible::TransferApi, reply, Reply};
use crate::error::ServiceErrorDomain;
use crate::fungible::{Invoice, IssueStructure, Outcoincealed, Outcoins, Outpoint};
use crate::util::file::ReadWrite;
use crate::util::SealSpec;

impl Runtime {
    fn command(&mut self, command: Request) -> Result<Arc<Reply>, ServiceErrorDomain> {
        let data = command.encode()?;
        self.session_rpc.send_raw_message(data)?;
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.unmarshaller.unmarshall(&raw)?;
        Ok(reply)
    }

    pub fn issue(
        &mut self,
        _network: bp::Network,
        ticker: String,
        title: String,
        description: Option<String>,
        issue_structure: IssueStructure,
        allocate: Vec<Outcoins>,
        precision: u8,
        _prune_seals: Vec<SealSpec>,
        dust_limit: Option<Amount>,
    ) -> Result<(), Error> {
        // TODO: Make sure we use the same network
        let (supply, inflatable) = match issue_structure {
            IssueStructure::SingleIssue => (None, None),
            IssueStructure::MultipleIssues {
                max_supply,
                reissue_control,
            } => (Some(max_supply), Some(reissue_control)),
        };
        let command = Request::Issue(Issue {
            ticker,
            title,
            description,
            supply,
            inflatable,
            precision,
            dust_limit,
            allocate,
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
        allocate: Vec<Outcoins>,
        invoice: Invoice,
        prototype_psbt: String,
        fee: u64,
        change: OutPoint,
        consignment_file: String,
        transaction_file: String,
    ) -> Result<(), Error> {
        let seal_confidential = match invoice.outpoint {
            Outpoint::BlindedUtxo(outpoint_hash) => outpoint_hash,
            Outpoint::Address(_address) => unimplemented!(),
        };

        let pubkey_key = Key {
            type_value: 0xFC,
            key: PSBT_PUBKEY_KEY.to_vec(),
        };
        let fee_key = Key {
            type_value: 0xFC,
            key: PSBT_FEE_KEY.to_vec(),
        };

        let psbt_bytes = base64::decode(&prototype_psbt)?;
        let mut psbt: PartiallySignedTransaction = deserialize(&psbt_bytes)?;

        psbt.global
            .unknown
            .insert(fee_key, fee.to_be_bytes().to_vec());
        for output in &mut psbt.outputs {
            output.unknown.insert(
                pubkey_key.clone(),
                output.hd_keypaths.keys().next().unwrap().to_bytes(),
            );
        }
        // trace!("{:?}", psbt);

        let api = TransferApi {
            psbt,
            contract_id: invoice.contract_id,
            inputs,
            ours: allocate,
            theirs: vec![Outcoincealed {
                coins: invoice.amount,
                seal_confidential,
            }],
            change,
        };

        // TODO: Do tx output reorg for deterministic ordering

        match &*self.command(Request::Transfer(api))? {
            Reply::Failure(failure) => Err(Error::Reply(failure.clone())),
            Reply::Transfer(transfer) => {
                transfer
                    .consignment
                    .write_file(PathBuf::from(&consignment_file))?;
                let out_file =
                    File::create(&transaction_file).expect("can't create output transaction file");
                transfer.psbt.consensus_encode(out_file)?;
                println!(
                    "Transfer succeeded, consignment data are written to {:?}, partially signed witness transaction to {:?}",
                    consignment_file, transaction_file
                );

                Ok(())
            }
            _ => Err(Error::UnexpectedResponse),
        }
    }

    pub fn sync(&mut self) -> Result<reply::SyncFormat, Error> {
        match &*self.command(Request::Sync)? {
            Reply::Sync(data) => Ok(data.clone()),
            Reply::Failure(failmsg) => Err(Error::Reply(failmsg.clone())),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}
