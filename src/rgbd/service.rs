// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::{BTreeSet, VecDeque};

use internet2::ZmqSocketType;
use microservices::cli::LogStyle;
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::EndpointList;
use microservices::node::TryService;
use rgb::{Contract, StateTransfer};
use rgb_rpc::{ClientId, RpcMsg};
use storm_ext::ExtMsg as StormMsg;

use crate::bus::{
    BusMsg, CtlMsg, DaemonId, Endpoints, ProcessReq, Responder, ServiceBus, ServiceId,
};
use crate::daemons::Daemon;
use crate::{Config, DaemonError, Db, LaunchError};

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

    pub(crate) db: Db,

    pub(crate) containerd_free: VecDeque<DaemonId>,
    pub(crate) containerd_busy: BTreeSet<DaemonId>,
    pub(crate) ctl_queue: VecDeque<CtlMsg>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        debug!("Connecting to store service at {}", config.store_endpoint);

        let db = Db::with(&config.store_endpoint)?;

        info!("RGBd runtime started successfully");

        Ok(Self {
            config,
            db,
            containerd_free: empty!(),
            containerd_busy: empty!(),
            ctl_queue: empty!(),
        })
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
        _error: esb::Error<ServiceId>,
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
        _endpoints: &mut Endpoints,
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
                self.process_contract(endpoints, contract, client_id)?;
            }
            RpcMsg::AcceptTransfer(transfer) => {
                self.process_transfer(endpoints, transfer, client_id)?;
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
            CtlMsg::Hello => self.handle_hello(endpoints, source)?,

            CtlMsg::Validity(_) => {
                self.handle_validity(endpoints, source)?;
                self.pick_consignment(endpoints)?;
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
    fn handle_hello(
        &mut self,
        endpoints: &mut Endpoints,
        source: ServiceId,
    ) -> Result<(), esb::Error<ServiceId>> {
        info!("{} daemon is {}", source.ended(), "connected".ended());

        match source {
            ServiceId::Rgb => {
                error!("{}", "Unexpected another RGBd instance connection".err());
            }
            ServiceId::Container(daemon_id) => {
                info!(
                    "Container daemon {} is registered; total {} container processors are known",
                    daemon_id,
                    self.containerd_busy.len()
                );
                self.containerd_free.push_back(daemon_id);
                self.pick_consignment(endpoints)?;
            }
            _ => {
                // Ignoring the rest of daemon/client types
            }
        }

        Ok(())
    }

    fn handle_validity(
        &mut self,
        _endpoints: &mut Endpoints,
        _source: ServiceId,
    ) -> Result<(), esb::Error<ServiceId>> {
        // Nothing to do here for now
        Ok(())
    }

    fn pick_consignment(
        &mut self,
        endpoints: &mut Endpoints,
    ) -> Result<bool, esb::Error<ServiceId>> {
        let service = match self.containerd_free.pop_front() {
            Some(damon_id) => ServiceId::Container(damon_id),
            None => return Ok(false),
        };

        let msg = match self.ctl_queue.pop_front() {
            None => return Ok(true),
            Some(req) => req,
        };

        self.send_ctl(endpoints, service, msg)?;
        Ok(true)
    }

    fn pick_or_start_containerd(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
    ) -> Result<(), DaemonError> {
        if self.pick_consignment(endpoints)? {
            let _ = self.send_rpc(
                endpoints,
                client_id,
                RpcMsg::Progress(s!("Consignment forwarded to container daemon")),
            );
            return Ok(());
        }

        let _handle = self.launch_daemon(Daemon::Containerd, self.config.clone())?;
        let _ = self.send_rpc(
            endpoints,
            client_id,
            RpcMsg::Progress(s!("A new container daemon instance is started")),
        );

        // TODO: Store daemon handlers
        Ok(())
    }

    fn process_contract(
        &mut self,
        endpoints: &mut Endpoints,
        contract: Contract,
        client_id: ClientId,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::ProcessContract(ProcessReq {
            client_id,
            consignment: contract,
        }));
        self.pick_or_start_containerd(endpoints, client_id)
    }

    fn process_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        transfer: StateTransfer,
        client_id: ClientId,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::ProcessTransfer(ProcessReq {
            client_id,
            consignment: transfer,
        }));
        self.pick_or_start_containerd(endpoints, client_id)
    }
}
