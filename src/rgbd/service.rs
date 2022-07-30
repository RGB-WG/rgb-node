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
use bitcoin::OutPoint;
use internet2::addr::NodeAddr;
use internet2::ZmqSocketType;
use lnpbp::chain::Chain;
use microservices::cli::LogStyle;
use microservices::error::BootstrapError;
use microservices::esb::{ClientId, EndpointList};
use microservices::node::TryService;
use microservices::{esb, rpc};
use psbt::Psbt;
use rgb::schema::TransitionType;
use rgb::{
    Contract, ContractConsignment, ContractId, SealEndpoint, StateTransfer, TransferConsignment,
};
use rgb_rpc::{AcceptReq, ComposeReq, FailureCode, HelloReq, OutpointFilter, RpcMsg, TransferReq};
use storm::ContainerId;
use storm_ext::ExtMsg as StormMsg;
use storm_rpc::AddressedMsg;

use crate::bucketd::StashError;
use crate::bus::{
    BusMsg, ConsignReq, CtlMsg, DaemonId, Endpoints, FinalizeTransferReq, OutpointStateReq,
    ProcessReq, Responder, ServiceBus, ServiceId,
};
use crate::db::ChunkHolder;
use crate::rgbd::daemons::Daemon;
use crate::{db, Config, DaemonError, LaunchError};

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
                Some(ServiceId::stormd())
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
    .map_err(|_| LaunchError::BusSetupFailure)?;

    controller.run_or_panic("rgbd");

    unreachable!()
}

pub struct Runtime {
    /// Original configuration object
    pub(crate) config: Config,

    pub(crate) store: store_rpc::Client,

    pub(crate) bucketd_free: VecDeque<DaemonId>,
    pub(crate) bucketd_busy: BTreeSet<DaemonId>,
    pub(crate) ctl_queue: VecDeque<CtlMsg>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        debug!("Connecting to store service at {}", config.store_endpoint);

        let mut store =
            store_rpc::Client::with(&config.store_endpoint).map_err(LaunchError::from)?;

        for table in [
            db::SCHEMATA,
            db::CONTRACTS,
            db::BUNDLES,
            db::GENESIS,
            db::TRANSITIONS,
            db::ANCHORS,
            db::EXTENSIONS,
            db::ATTACHMENT_CHUNKS,
            db::ATTACHMENT_INDEX,
            db::ALU_LIBS,
            db::OUTPOINTS,
            db::NODE_CONTRACTS,
            db::TRANSITION_WITNESS,
            db::CONTRACT_TRANSITIONS,
            db::DISCLOSURES,
        ] {
            store.use_table(table.to_owned()).map_err(LaunchError::from)?;
        }

        info!("RGBd runtime started successfully");

        Ok(Self {
            config,
            store,
            bucketd_free: empty!(),
            bucketd_busy: empty!(),
            ctl_queue: empty!(),
        })
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::rgbd() }

    fn handle(
        &mut self,
        endpoints: &mut EndpointList<ServiceBus>,
        bus_id: ServiceBus,
        source: ServiceId,
        request: Self::Request,
    ) -> Result<(), Self::Error> {
        match (bus_id, request, source) {
            (ServiceBus::Storm, BusMsg::Storm(msg), service_id)
                if service_id == ServiceId::stormd() =>
            {
                self.handle_storm(endpoints, msg)
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
        endpoints: &mut Endpoints,
        message: StormMsg,
    ) -> Result<(), DaemonError> {
        match message {
            StormMsg::ContainerAnnouncement(AddressedMsg { remote_id, data }) => {
                self.send_storm(
                    endpoints,
                    StormMsg::RetrieveContainer(AddressedMsg {
                        remote_id,
                        data: data.id,
                    }),
                )?;
            }

            StormMsg::RetrieveContainer(AddressedMsg { remote_id, data }) => {
                self.send_storm(
                    endpoints,
                    StormMsg::SendContainer(AddressedMsg { remote_id, data }),
                )?;
            }

            // We receive this message when we asked storm daemon to download announced container
            // and the container got downloaded
            StormMsg::ContainerRetrieved(container_id) => {
                self.process_transfer(endpoints, container_id)?;
            }

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
            RpcMsg::ConsignContract(ComposeReq {
                contract_id,
                include,
                outpoints,
            }) => {
                self.consign_contract(endpoints, client_id, contract_id, include, outpoints)?;
            }
            RpcMsg::ConsignTransfer(ComposeReq {
                contract_id,
                include,
                outpoints,
            }) => {
                self.consign_transfer(endpoints, client_id, contract_id, include, outpoints)?;
            }
            RpcMsg::GetContractState(contract_id) => {
                self.get_contract_state(endpoints, client_id, contract_id)?;
            }
            RpcMsg::GetOutpointState(outpoints) => {
                self.outpoint_transitions(endpoints, client_id, outpoints)?;
            }
            RpcMsg::ConsumeContract(AcceptReq {
                consignment: contract,
                force,
            }) => {
                self.accept_contract(endpoints, client_id, contract, force)?;
            }
            RpcMsg::ConsumeTransfer(AcceptReq {
                consignment: transfer,
                force,
            }) => {
                self.accept_transfer(endpoints, client_id, transfer, force)?;
            }

            RpcMsg::Transfer(TransferReq {
                consignment,
                endseals,
                psbt,
                beneficiary,
            }) => {
                self.complete_transfer(
                    endpoints,
                    client_id,
                    consignment,
                    endseals,
                    psbt,
                    beneficiary,
                )?;
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
                if let ServiceId::Bucket(daemon_id) = source {
                    self.bucketd_busy.remove(&daemon_id);
                    self.bucketd_free.push_back(daemon_id);
                    self.pick_task(endpoints)?;
                }
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
            service_id if service_id == ServiceId::rgbd() => {
                error!("{}", "Unexpected another RGBd instance connection".err());
            }
            ServiceId::Bucket(daemon_id) => {
                self.bucketd_free.push_back(daemon_id);
                info!(
                    "Bucket daemon {} is registered; total {} container processors are known",
                    daemon_id,
                    self.bucketd_free.len() + self.bucketd_busy.len()
                );
            }
            _ => {
                // Ignoring the rest of daemon/client types
            }
        }

        Ok(())
    }

    fn pick_task(&mut self, endpoints: &mut Endpoints) -> Result<bool, esb::Error<ServiceId>> {
        if self.ctl_queue.is_empty() {
            return Ok(true);
        }

        let (service, daemon_id) = match self.bucketd_free.front() {
            Some(damon_id) => (ServiceId::Bucket(*damon_id), *damon_id),
            None => return Ok(false),
        };

        let msg = match self.ctl_queue.pop_front() {
            None => return Ok(true),
            Some(req) => req,
        };

        debug!("Assigning task {} to {}", msg, service);

        self.send_ctl(endpoints, service, msg)?;
        self.bucketd_free.pop_front();
        self.bucketd_busy.insert(daemon_id);
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
                RpcMsg::Progress(s!("Task forwarded to bucket daemon")),
            );
            return Ok(());
        }

        let _handle = self.launch_daemon(Daemon::Bucketd, self.config.clone())?;
        let _ = self.send_rpc(
            endpoints,
            client_id,
            RpcMsg::Progress(s!("A new bucket daemon instance is started")),
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
        let ids = self.store.ids(db::CONTRACTS)?;
        let ids = ids
            .into_iter()
            .map(|id| ContractId::from_inner(Hash::from_inner(id.into_inner())))
            .collect();
        let _ = self.send_rpc(endpoints, client_id, RpcMsg::ContractIds(ids));
        Ok(())
    }

    fn consign_contract(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract_id: ContractId,
        include: BTreeSet<TransitionType>,
        outpoints: OutpointFilter,
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

    fn consign_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract_id: ContractId,
        include: BTreeSet<TransitionType>,
        outpoints: OutpointFilter,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::ConsignTranfer(ConsignReq {
            client_id,
            contract_id,
            include,
            outpoints,
            _phantom: TransferConsignment,
        }));
        self.pick_or_start(endpoints, client_id)
    }

    fn get_contract_state(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract_id: ContractId,
    ) -> Result<(), DaemonError> {
        let msg = match self.store.retrieve(db::CONTRACTS, contract_id)? {
            Some(state) => RpcMsg::ContractState(ChunkHolder::unbox(state)),
            None => DaemonError::from(StashError::StateAbsent(contract_id)).into(),
        };
        let _ = self.send_rpc(endpoints, client_id, msg);
        Ok(())
    }

    fn outpoint_transitions(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        outpoints: BTreeSet<OutPoint>,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::OutpointState(OutpointStateReq {
            client_id,
            outpoints,
        }));
        self.pick_or_start(endpoints, client_id)
    }

    fn accept_contract(
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

    fn accept_transfer(
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

    fn process_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        container_id: ContainerId,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::ProcessTransferContainer(container_id));
        if !self.pick_task(endpoints)? {
            self.launch_daemon(Daemon::Bucketd, self.config.clone())?;
        }
        Ok(())
    }

    fn complete_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        consignment: StateTransfer,
        endseals: Vec<SealEndpoint>,
        psbt: Psbt,
        beneficiary: Option<NodeAddr>,
    ) -> Result<(), DaemonError> {
        self.ctl_queue.push_back(CtlMsg::FinalizeTransfer(FinalizeTransferReq {
            client_id,
            consignment,
            endseals,
            psbt,
            beneficiary,
        }));
        self.pick_or_start(endpoints, client_id)
    }
}
