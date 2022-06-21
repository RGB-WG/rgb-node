// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use bitcoin::secp256k1::rand::random;
use commit_verify::ConsensusCommit;
use internet2::{CreateUnmarshaller, Unmarshaller, ZmqSocketType};
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{EndpointList, Error};
use microservices::node::TryService;
use rgb::{ConsignmentType, InmemConsignment, Validity};
use rgb_rpc::{ClientId, RpcMsg};

use crate::bus::{
    BusMsg, CtlMsg, DaemonId, Endpoints, ProcessReq, Responder, ServiceBus, ServiceId, ValidityResp,
};
use crate::{Config, DaemonError, Db, LaunchError};

pub fn run(config: Config) -> Result<(), BootstrapError<LaunchError>> {
    let rpc_endpoint = config.rpc_endpoint.clone();
    let ctl_endpoint = config.ctl_endpoint.clone();
    let runtime = Runtime::init(config)?;

    debug!("Connecting to service buses {}, {}", rpc_endpoint, ctl_endpoint);
    let controller = esb::Controller::with(
        map! {
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

    controller.run_or_panic("containerd");

    unreachable!()
}

pub struct Runtime {
    id: DaemonId,

    /// Original configuration object
    pub(crate) config: Config,

    pub(crate) db: Db,

    /// Unmarshaller instance used for parsing RPC request
    pub(crate) unmarshaller: Unmarshaller<BusMsg>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        debug!("Connecting to store service at {}", config.store_endpoint);

        let db = Db::with(&config.store_endpoint)?;

        let id = random();

        info!("Containerd runtime started successfully");

        Ok(Self {
            id,
            config,
            db,
            unmarshaller: BusMsg::create_unmarshaller(),
        })
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::Container(self.id) }

    fn on_ready(&mut self, endpoints: &mut EndpointList<ServiceBus>) -> Result<(), Self::Error> {
        endpoints.send_to(
            ServiceBus::Ctl,
            self.identity(),
            ServiceId::Rgb,
            BusMsg::Ctl(CtlMsg::Hello),
        )?;
        Ok(())
    }

    fn handle(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        bus_id: ServiceBus,
        source: ServiceId,
        request: Self::Request,
    ) -> Result<(), Self::Error> {
        match (bus_id, request, source) {
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
    fn handle_rpc(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        message: RpcMsg,
    ) -> Result<(), DaemonError> {
        match message {
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
        _source: ServiceId,
        message: CtlMsg,
    ) -> Result<(), DaemonError> {
        match message {
            CtlMsg::ProcessContract(ProcessReq {
                client_id,
                consignment,
            }) => {
                self.handle_consignment(endpoints, client_id, consignment)?;
            }
            CtlMsg::ProcessTransfer(ProcessReq {
                client_id,
                consignment,
            }) => {
                self.handle_consignment(endpoints, client_id, consignment)?;
            }

            wrong_msg => {
                error!("Request is not supported by the CTL interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Ctl, &wrong_msg));
            }
        }

        Ok(())
    }
}

impl Runtime {
    fn handle_consignment<C: ConsignmentType>(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        consignment: InmemConsignment<C>,
    ) -> Result<(), DaemonError> {
        let id = consignment.consensus_commit();
        match self.process_consignment(consignment) {
            Err(_) => self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingFailed)?,
            Ok(status) => {
                // We ignore client reporting if it fails
                let msg = match status.validity() {
                    Validity::Valid => RpcMsg::success(),
                    Validity::UnresolvedTransactions => {
                        RpcMsg::UnresolvedTxids(status.unresolved_txids.clone())
                    }
                    Validity::Invalid => RpcMsg::Invalid(status.clone()),
                };
                let _ = self.send_rpc(endpoints, client_id, msg);
                self.send_ctl(endpoints, ServiceId::Rgb, ValidityResp {
                    client_id,
                    consignment_id: id,
                    status,
                })?
            }
        }
        Ok(())
    }
}
