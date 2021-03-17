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

use std::sync::Arc;

use bitcoin::OutPoint;
use internet2::zmqsocket::ZmqType;
use internet2::{
    session, transport, CreateUnmarshaller, PlainTranscoder, Session,
    TypedEnum, Unmarshall, Unmarshaller,
};
use rgb::{Consignment, ContractId, Disclosure, Genesis, SchemaId};

use super::{Config, Error};
use crate::cli::OutputFormat;
use crate::error::{BootstrapError, ServiceErrorDomain};
use crate::rpc::fungible::{self, AcceptReq, IssueReq, TransferReq};
use crate::rpc::stash;
use crate::rpc::Reply;
use microservices::FileFormat;

pub struct Runtime {
    stash_rpc: session::Raw<PlainTranscoder, transport::zmqsocket::Connection>,
    fungible_rpc:
        session::Raw<PlainTranscoder, transport::zmqsocket::Connection>,
    unmarshaller: Unmarshaller<Reply>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError> {
        let fungible_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Req,
            &config.fungible_endpoint,
            None,
            None,
        )?;
        let stash_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Req,
            &config.stash_endpoint,
            None,
            None,
        )?;
        Ok(Self {
            stash_rpc,
            fungible_rpc,
            unmarshaller: Reply::create_unmarshaller(),
        })
    }

    fn stash_command(
        &mut self,
        command: stash::Request,
    ) -> Result<Arc<Reply>, ServiceErrorDomain> {
        let data = command.serialize();
        self.stash_rpc.send_raw_message(&data)?;
        let raw = self.stash_rpc.recv_raw_message()?;
        let reply = self.unmarshaller.unmarshall(&raw)?;
        Ok(reply)
    }

    fn fungible_command(
        &mut self,
        command: fungible::Request,
    ) -> Result<Arc<Reply>, ServiceErrorDomain> {
        let data = command.serialize();
        self.fungible_rpc.send_raw_message(&data)?;
        let raw = self.fungible_rpc.recv_raw_message()?;
        let reply = self.unmarshaller.unmarshall(&raw)?;
        Ok(reply)
    }

    #[inline]
    pub fn list_schemata(&mut self) -> Result<Arc<Reply>, Error> {
        Ok(self.stash_command(stash::Request::ListSchemata())?)
    }

    #[inline]
    pub fn list_geneses(&mut self) -> Result<Arc<Reply>, Error> {
        Ok(self.stash_command(stash::Request::ListGeneses())?)
    }

    #[inline]
    pub fn schema(&mut self, schema_id: SchemaId) -> Result<Arc<Reply>, Error> {
        Ok(self.stash_command(stash::Request::ReadSchema(schema_id))?)
    }

    #[inline]
    pub fn genesis(
        &mut self,
        contract_id: ContractId,
    ) -> Result<Arc<Reply>, Error> {
        Ok(self.stash_command(stash::Request::ReadGenesis(contract_id))?)
    }

    #[inline]
    pub fn list(
        &mut self,
        output_format: OutputFormat,
    ) -> Result<Arc<Reply>, Error> {
        let data_format = match output_format {
            OutputFormat::Yaml => FileFormat::Yaml,
            OutputFormat::Json => FileFormat::Json,
            OutputFormat::Toml => FileFormat::Toml,
            OutputFormat::StrictEncode => FileFormat::StrictEncode,
            _ => unimplemented!("The provided output format is not supported for this operation")
        };
        Ok(self.fungible_command(fungible::Request::Sync(data_format))?)
    }

    #[inline]
    pub fn import(&mut self, genesis: Genesis) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::ImportAsset(genesis))?)
    }

    #[inline]
    pub fn export(
        &mut self,
        asset_id: ContractId,
    ) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::ExportAsset(asset_id))?)
    }

    #[inline]
    pub fn issue(&mut self, issue: IssueReq) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::Issue(issue))?)
    }

    #[inline]
    pub fn transfer(
        &mut self,
        transfer: TransferReq,
    ) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::Transfer(transfer))?)
    }

    #[inline]
    pub fn validate(
        &mut self,
        consignment: Consignment,
    ) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::Validate(consignment))?)
    }

    #[inline]
    pub fn accept(&mut self, accept: AcceptReq) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::Accept(accept))?)
    }

    #[inline]
    pub fn enclose(
        &mut self,
        disclosure: Disclosure,
    ) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::Enclose(disclosure))?)
    }

    #[inline]
    pub fn forget(&mut self, outpoint: OutPoint) -> Result<Arc<Reply>, Error> {
        Ok(self.fungible_command(fungible::Request::Forget(outpoint))?)
    }
}
