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

use std::thread;

use internet2::ZmqType;
use internet2::{
    session, transport, CreateUnmarshaller, PlainTranscoder, Unmarshaller,
};

use super::Config;
use crate::error::BootstrapError;
use crate::rgbd::{self, ContractName};
use crate::rpc::Reply;

pub struct Runtime {
    pub(super) config: Config,
    pub(super) session_rpc:
        session::Raw<PlainTranscoder, transport::zmqsocket::Connection>,
    pub(super) unmarshaller: Unmarshaller<Reply>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError> {
        // Start rgbd on a separate thread
        if config.run_embedded {
            let rgbd_opts = rgbd::Opts {
                bin_dir: s!(""), // We do not need binaries in multithread mode
                data_dir: config.data_dir.clone(),
                verbose: config.verbose,
                electrum_server: config.electrum_server.clone(),
                // TODO: Endpoint parameters are not needed in embedded mode;
                //       remove them
                // Issue #159
                contracts: config.contract_endpoints.keys().cloned().collect(),
                fungible_rpc_endpoint: config
                    .contract_endpoints
                    .get(&ContractName::Fungible)
                    .ok_or(BootstrapError::ArgParseError(s!(
                        "Fungible endpoint is unconfigured"
                    )))?
                    .to_string(),
                stash_rpc_endpoint: config.stash_rpc_endpoint.to_string(),
                network: config.network.clone(),
                threaded: true,
                ..rgbd::Opts::default()
            };

            thread::spawn(move || {
                rgbd::main_with_config(rgbd_opts.into()).unwrap();
            });
        }

        let session_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Req,
            config
                .contract_endpoints
                .get(&ContractName::Fungible)
                .as_ref()
                .expect(
                    "Fungible engine is not connected in the configuration",
                ),
            None,
            None,
        )?;
        Ok(Self {
            config,
            session_rpc,
            unmarshaller: Reply::create_unmarshaller(),
        })
    }
}
