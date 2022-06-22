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

use amplify::Wrapper;
use bitcoin::hashes::Hash;
use internet2::ZmqSocketType;
use lnpbp::chain::Chain;
use microservices::cli::LogStyle;
use microservices::error::BootstrapError;
use microservices::esb::EndpointList;
use microservices::node::TryService;
use microservices::{esb, rpc};
use rgb::schema::TransitionType;
use rgb::{Contract, ContractConsignment, ContractId, StateTransfer};
use rgb_rpc::{AcceptReq, ClientId, ContractReq, FailureCode, HelloReq, OutpointSelection, RpcMsg};
use storm_ext::ExtMsg as StormMsg;

use crate::bus::{
    BusMsg, ConsignReq, CtlMsg, DaemonId, Endpoints, ProcessReq, Responder, ServiceBus, ServiceId,
};
use crate::containerd::StashError;
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
            RpcMsg::Hello(HelloReq {
                user_agent,
                network,
            }) => {
                self.accept_client(endpoints, client_id, user_agent, network)?;
            }
            RpcMsg::ListContracts => {
                self.list_contracts(endpoints, client_id)?;
            }
            RpcMsg::GetContract(ContractReq {
                contract_id,
                include,
                outpoints,
            }) => {
                self.get_contract(endpoints, client_id, contract_id, include, outpoints)?;
            }
            RpcMsg::GetContractState(contract_id) => {
                self.get_contract_state(endpoints, client_id, contract_id)?;
            }
            RpcMsg::AcceptContract(AcceptReq {
                consignment: contract,
                force,
            }) => {
                self.process_contract(endpoints, client_id, contract, force)?;
            }
            RpcMsg::AcceptTransfer(AcceptReq {
                consignment: transfer,
                force,
            }) => {
                self.process_transfer(endpoints, client_id, transfer, force)?;
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
            CtlMsg::Hello => {
                self.accept_daemon(source)?;
                self.pick_task(endpoints)?;
            }
            CtlMsg::Validity(_) | CtlMsg::ProcessingFailed | CtlMsg::ProcessingComplete => {
                self.pick_task(endpoints)?;
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
    fn accept_daemon(&mut self, source: ServiceId) -> Result<(), esb::Error<ServiceId>> {
        info!("{} daemon is {}", source.ended(), "connected".ended());

        match source {
            ServiceId::Rgb => {
                error!("{}", "Unexpected another RGBd instance connection".err());
            }
            ServiceId::Container(daemon_id) => {
                self.containerd_free.push_back(daemon_id);
                info!(
                    "Container daemon {} is registered; total {} container processors are known",
                    daemon_id,
                    self.containerd_free.len() + self.containerd_busy.len()
                );
            }
            _ => {
                // Ignoring the rest of daemon/client types
            }
        }

        Ok(())
    }

    fn pick_task(&mut self, endpoints: &mut Endpoints) -> Result<bool, esb::Error<ServiceId>> {
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

    fn pick_or_start(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
    ) -> Result<(), DaemonError> {
        if self.pick_task(endpoints)? {
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

    fn accept_client(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        user_agent: String,
        network: Chain,
    ) -> Result<(), DaemonError> {
        info!("Accepting new client with id {} ({})", client_id, user_agent);
        let msg = match self.config.chain == network {
            true => RpcMsg::success(),
            false => rpc::Failure {
                code: rpc::FailureCode::Other(FailureCode::ChainMismatch),
                info: s!("wrong network"),
            }
            .into(),
        };
        let _ = self.send_rpc(endpoints, client_id, msg);
        Ok(())
    }

    fn list_contracts(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
    ) -> Result<(), DaemonError> {
        let ids = self.db.store.ids(Db::CONTRACTS.to_owned())?;
        let ids = ids
            .into_iter()
            .map(|id| ContractId::from_inner(Hash::from_inner(id.into_inner())))
            .collect();
        let _ = self.send_rpc(endpoints, client_id, RpcMsg::ContractIds(ids));
        Ok(())
    }

    fn get_contract(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract_id: ContractId,
        include: BTreeSet<TransitionType>,
        outpoints: OutpointSelection,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::ConsignContract(ConsignReq {
            client_id,
            contract_id,
            include,
            outpoints,
            _phantom: ContractConsignment,
        }));
        self.pick_or_start(endpoints, client_id)
    }

    fn get_contract_state(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract_id: ContractId,
    ) -> Result<(), DaemonError> {
        let msg = match self.db.retrieve(Db::CONTRACTS, contract_id)? {
            Some(state) => RpcMsg::ContractState(state),
            None => DaemonError::from(StashError::GenesisAbsent).into(),
        };
        let _ = self.send_rpc(endpoints, client_id, msg);
        Ok(())
    }

    fn process_contract(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract: Contract,
        force: bool,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::ProcessContract(ProcessReq {
            client_id,
            consignment: contract,
            force,
        }));
        self.pick_or_start(endpoints, client_id)
    }

    fn process_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        transfer: StateTransfer,
        force: bool,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::ProcessTransfer(ProcessReq {
            client_id,
            consignment: transfer,
            force,
        }));
        self.pick_or_start(endpoints, client_id)
    }
}
