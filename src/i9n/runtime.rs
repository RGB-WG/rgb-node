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

use std::convert::TryFrom;

use lnpbp::lnp::transport::zmq::ApiType;
use lnpbp::lnp::{transport, NoEncryption, NodeAddr, Session};
use lnpbp::TryService;

use super::Config;
use crate::rgbd::ContractName;
use crate::BootstrapError;

pub struct Runtime {
    config: Config,
    context: zmq::Context,
    session_rpc: Session<NoEncryption, transport::zmq::Connection>,
}

impl Runtime {
    pub async fn init(config: Config) -> Result<Self, BootstrapError> {
        let mut context = zmq::Context::new();
        let session_rpc = Session::new_zmq_unencrypted(
            ApiType::Client,
            &mut context,
            config
                .contract_endpoints
                .get(&ContractName::Fungible)
                .expect("Fungible engine is not connected in the configuration")
                .clone(),
            None,
        )?;
        Ok(Self {
            config,
            context,
            session_rpc,
        })
    }
}
