// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::Slice32;
use bitcoin::hashes::{sha256t, Hash};
use commit_verify::TaggedHash;
use internet2::{CreateUnmarshaller, Unmarshaller, ZmqSocketType};
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{EndpointList, Error};
use microservices::node::TryService;
use rgb::Node;
use rgb_rpc::{ClientId, RpcMsg};
use storm_app::AppMsg as StormMsg;
use strict_encoding::StrictEncode;

use crate::bus::{BusMsg, CtlMsg, Endpoints, Responder, ServiceBus, ServiceId};
use crate::{Config, DaemonError, LaunchError};

const DB_TABLE_SCHEMATA: &str = "schemata";
const DB_TABLE_BUNDLES: &str = "bundles";
const DB_TABLE_GENESIS: &str = "genesis";
const DB_TABLE_TRANSITIONS: &str = "transitions";
const DB_TABLE_ANCHORS: &str = "transitions";
const DB_TABLE_EXTENSIONS: &str = "extensions";
const DB_TABLE_ATTACHMENT_CHUNKS: &str = "chunks";
const DB_TABLE_ATTACHMENT_INDEX: &str = "attachments";
const DB_TABLE_ALU_LIBS: &str = "alu";

pub fn run(config: Config) -> Result<(), BootstrapError<LaunchError>> {
    let storm_endpoint = config.storm_endpoint.clone();
    let rpc_endpoint = config.rpc_endpoint.clone();
    let ctl_endpoint = config.ctl_endpoint.clone();
    let runtime = Runtime::init(config)?;

    debug!("Connecting to service buses {}, {}, {}", storm_endpoint, rpc_endpoint, ctl_endpoint);
    let controller = esb::Controller::with(
        map! {
            ServiceBus::Storm => esb::BusConfig::with_addr(
                storm_endpoint,
                ZmqSocketType::RouterConnect,
                Some(ServiceId::Storm)
            ),
            ServiceBus::Rpc => esb::BusConfig::with_addr(
                rpc_endpoint,
                ZmqSocketType::RouterBind,
                None
            ),
            ServiceBus::Ctl => esb::BusConfig::with_addr(
                ctl_endpoint,
                ZmqSocketType::RouterBind,
                None
            )
        },
        runtime,
    )
    .map_err(|_| LaunchError::NoLnpdConnection)?;

    controller.run_or_panic("rgbd");

    unreachable!()
}

pub struct Runtime {
    /// Original configuration object
    pub(crate) config: Config,

    pub(crate) store: store_rpc::Client,

    /// Unmarshaller instance used for parsing RPC request
    pub(crate) unmarshaller: Unmarshaller<BusMsg>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        debug!("Connecting to store service at {}", config.store_endpoint);

        let mut store =
            store_rpc::Client::with(&config.store_endpoint).map_err(LaunchError::from)?;

        for table in [
            DB_TABLE_SCHEMATA,
            DB_TABLE_BUNDLES,
            DB_TABLE_GENESIS,
            DB_TABLE_TRANSITIONS,
            DB_TABLE_ANCHORS,
            DB_TABLE_EXTENSIONS,
            DB_TABLE_ATTACHMENT_CHUNKS,
            DB_TABLE_ATTACHMENT_INDEX,
            DB_TABLE_ALU_LIBS,
        ] {
            store.use_table(table.to_owned()).map_err(LaunchError::from)?;
        }

        info!("RGBd runtime started successfully");

        Ok(Self {
            config,
            store,
            unmarshaller: BusMsg::create_unmarshaller(),
        })
    }

    pub(super) fn store<'a, T: 'a + sha256t::Tag>(
        &mut self,
        table: &'static str,
        key: impl TaggedHash<'a, T> + 'a,
        data: &impl StrictEncode,
    ) -> Result<(), DaemonError> {
        let slice = key.into_inner();
        let slice = slice.into_inner();
        self.store.store(table.to_owned(), Slice32::from(slice), data.strict_serialize()?)?;
        Ok(())
    }

    pub(super) fn store_h(
        &mut self,
        table: &'static str,
        key: impl Hash<Inner = [u8; 32]>,
        data: &impl StrictEncode,
    ) -> Result<(), DaemonError> {
        let slice = *key.as_inner();
        self.store.store(table.to_owned(), Slice32::from(slice), data.strict_serialize()?)?;
        Ok(())
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::Rgb }

    fn handle(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        bus_id: ServiceBus,
        source: ServiceId,
        request: Self::Request,
    ) -> Result<(), Self::Error> {
        match (bus_id, request, source) {
            (ServiceBus::Storm, BusMsg::Storm(msg), ServiceId::Storm) => {
                // TODO: Add remote peers to Strom message protocol
                self.handle_storm(endpoints, /* remote_peer, */ msg)
            }
            (ServiceBus::Rpc, BusMsg::Rpc(msg), ServiceId::Client(client_id)) => {
                self.handle_rpc(endpoints, client_id, msg)
            }
            (ServiceBus::Ctl, BusMsg::Ctl(msg), source) => self.handle_ctl(endpoints, source, msg),
            (bus, msg, _) => Err(DaemonError::wrong_esb_msg(bus, &msg)),
        }
    }

    fn handle_err(
        &mut self,
        _endpoints: &mut EndpointList<ServiceBus>,
        _error: Error<ServiceId>,
    ) -> Result<(), Self::Error> {
        // We do nothing and do not propagate error; it's already being reported
        // with `error!` macro by the controller. If we propagate error here
        // this will make whole daemon panic
        Ok(())
    }
}

impl Runtime {
    fn handle_storm(
        &mut self,
        endpoints: &mut Endpoints,
        // remote_peer: NodeAddr,
        message: StormMsg,
    ) -> Result<(), DaemonError> {
        match message {
            wrong_msg => {
                error!("Request is not supported by the Storm interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &wrong_msg));
            }
        }
        Ok(())
    }

    fn handle_rpc(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        message: RpcMsg,
    ) -> Result<(), DaemonError> {
        match message {
            RpcMsg::AddContract(contract) => {
                // TODO: Validate consignment

                info!("Registering contract {}", contract.contract_id());
                trace!("{:?}", contract);

                self.store(DB_TABLE_SCHEMATA, contract.schema.schema_id(), &contract.schema)?;
                if let Some(root_schema) = &contract.root_schema {
                    self.store(DB_TABLE_SCHEMATA, root_schema.schema_id(), root_schema)?;
                }

                // TODO: IMPORTANT: concealed data will replace explicit.
                //       do a proper merge-reveal operation
                self.store(DB_TABLE_GENESIS, contract.genesis.contract_id(), &contract.genesis)?;

                for (anchor, bundle) in &contract.anchored_bundles {
                    // TODO: IMPORTANT: concealed data will replace explicit.
                    //       do a proper merge-reveal operation
                    self.store_h(DB_TABLE_ANCHORS, anchor.txid, anchor)?;
                    let mut data = Vec::new();
                    for (transition, inputs) in bundle {
                        self.store(DB_TABLE_TRANSITIONS, transition.node_id(), transition)?;
                        // TODO: IMPORTANT: concealed data will replace explicit.
                        //       do a proper merge-reveal operation
                        data.push((transition.node_id(), inputs.clone()));
                    }
                    // TODO: IMPORTANT: concealed data will replace explicit.
                    //       do a proper merge-reveal operation
                    self.store(DB_TABLE_BUNDLES, bundle.bundle_id(), &data)?;
                }
                for extension in &contract.state_extensions {
                    self.store(DB_TABLE_EXTENSIONS, extension.node_id(), extension)?;
                }

                self.send_rpc(endpoints, client_id, RpcMsg::Success(None.into()))?;
            }
            wrong_msg => {
                error!("Request is not supported by the RPC interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &wrong_msg));
            }
        }

        Ok(())
    }

    fn handle_ctl(
        &mut self,
        endpoints: &mut Endpoints,
        source: ServiceId,
        message: CtlMsg,
    ) -> Result<(), DaemonError> {
        match message {
            wrong_msg => {
                error!("Request is not supported by the CTL interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Ctl, &wrong_msg));
            }
        }

        Ok(())
    }
}
