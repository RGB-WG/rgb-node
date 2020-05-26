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

use std::io;

use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::transport::zmq::ApiType;
use lnpbp::lnp::{transport, NoEncryption, Session, Unmarshall, Unmarshaller};
use lnpbp::rgb::Genesis;

use super::Config;
use crate::api::fungible::Issue;
use crate::api::Reply;
use crate::error::{BootstrapError, ServiceErrorDomain};
use crate::fungible::{Asset, Command};

pub struct Runtime {
    config: Config,
    context: zmq::Context,
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
            context,
            session_rpc,
            unmarshaller: Reply::create_unmarshaller(),
        })
    }

    pub fn issue(&mut self, issue: Issue) -> Result<(), ServiceErrorDomain> {
        let command = Command::Issue(issue);
        let data = command.encode()?;
        self.session_rpc.send_raw_message(data)?;
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = &*self.unmarshaller.unmarshall(&raw)?;
        info!("{}", reply);
        Ok(())
    }
}
