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

use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::transport::zmq::ApiType;
use lnpbp::lnp::{transport, NoEncryption, Session, Unmarshall, Unmarshaller};
use lnpbp::rgb::{ContractId, Genesis};

use super::{Config, Error};
use crate::api::fungible::{AcceptApi, Issue, Request, TransferApi};
use crate::api::Reply;
use crate::error::{BootstrapError, ServiceErrorDomain};

pub struct Runtime {
    config: Config,
    session_rpc: Session<NoEncryption, transport::zmq::Connection>,
    unmarshaller: Unmarshaller<Reply>,
}

impl Runtime {
    pub async fn init(config: Config) -> Result<Self, BootstrapError> {
        let mut context = zmq::Context::new();
        let session_rpc = Session::new_zmq_unencrypted(
            ApiType::Client,
            &mut context,
            config.endpoint.clone(),
            None,
        )?;
        Ok(Self {
            config,
            session_rpc,
            unmarshaller: Reply::create_unmarshaller(),
        })
    }

    fn command(&mut self, command: Request) -> Result<Arc<Reply>, ServiceErrorDomain> {
        let data = command.encode()?;
        self.session_rpc.send_raw_message(data)?;
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.unmarshaller.unmarshall(&raw)?;
        Ok(reply)
    }

    #[inline]
    pub fn list(&mut self) -> Result<Arc<Reply>, Error> {
        Ok(self.command(Request::Sync)?)
    }

    #[inline]
    pub fn import(&mut self, genesis: Genesis) -> Result<Arc<Reply>, Error> {
        Ok(self.command(Request::ImportAsset(genesis))?)
    }

    #[inline]
    pub fn export(&mut self, asset_id: ContractId) -> Result<Arc<Reply>, Error> {
        Ok(self.command(Request::ExportAsset(asset_id))?)
    }

    #[inline]
    pub fn issue(&mut self, issue: Issue) -> Result<Arc<Reply>, Error> {
        Ok(self.command(Request::Issue(issue))?)
    }

    #[inline]
    pub fn transfer(&mut self, transfer: TransferApi) -> Result<Arc<Reply>, Error> {
        Ok(self.command(Request::Transfer(transfer))?)
    }

    #[inline]
    pub fn accept(&mut self, accept: AcceptApi) -> Result<Arc<Reply>, Error> {
        Ok(self.command(Request::Accept(accept))?)
    }
}
