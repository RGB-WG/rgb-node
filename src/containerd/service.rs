// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeSet;
use std::thread;
use std::time::Duration;

use bitcoin::secp256k1::rand::random;
use bitcoin::OutPoint;
use commit_verify::ConsensusCommit;
use electrum_client::Client as ElectrumClient;
use internet2::addr::NodeAddr;
use internet2::ZmqSocketType;
use microservices::error::BootstrapError;
use microservices::esb;
use microservices::esb::{EndpointList, Error};
use microservices::node::TryService;
use psbt::Psbt;
use rgb::schema::TransitionType;
use rgb::{
    ConsignmentType, ContractConsignment, ContractId, InmemConsignment, SealEndpoint,
    StateTransfer, TransferConsignment, Validity,
};
use rgb_rpc::{ClientId, OutpointFilter, RpcMsg};

use crate::bus::{
    BusMsg, ConsignReq, CtlMsg, DaemonId, Endpoints, FinalizeTransferReq, OutpointStateReq,
    ProcessReq, Responder, ServiceBus, ServiceId, ValidityResp,
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
                ZmqSocketType::RouterConnect,
                Some(ServiceId::Rgb)
            ),
            ServiceBus::Ctl => esb::BusConfig::with_addr(
                ctl_endpoint,
                ZmqSocketType::RouterConnect,
                Some(ServiceId::Rgb)
            )
        },
        runtime,
    )
    .map_err(|_| LaunchError::BusSetupFailure)?;

    controller.run_or_panic("containerd");

    unreachable!()
}

pub struct Runtime {
    id: DaemonId,

    pub(crate) electrum: ElectrumClient,

    pub(crate) db: Db,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, BootstrapError<LaunchError>> {
        debug!("Connecting to store service at {}", config.store_endpoint);

        let db = Db::with(&config.store_endpoint)?;

        let id = random();

        let electrum = ElectrumClient::new(&config.electrum_url)
            .map_err(|_| LaunchError::ElectrumConnectivity)?;

        info!("Containerd runtime started successfully");

        Ok(Self { id, db, electrum })
    }
}

impl Responder for Runtime {}

impl esb::Handler<ServiceBus> for Runtime {
    type Request = BusMsg;
    type Error = DaemonError;

    fn identity(&self) -> ServiceId { ServiceId::Container(self.id) }

    fn on_ready(&mut self, endpoints: &mut EndpointList<ServiceBus>) -> Result<(), Self::Error> {
        thread::sleep(Duration::from_millis(100));
        self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::Hello)?;
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
        _endpoints: &mut Endpoints,
        _client_id: ClientId,
        message: RpcMsg,
    ) -> Result<(), DaemonError> {
        match message {
            wrong_msg => {
                error!("Request is not supported by the RPC interface");
                return Err(DaemonError::wrong_esb_msg(ServiceBus::Rpc, &wrong_msg));
            }
        }
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
                force,
            }) => {
                self.handle_consignment(endpoints, client_id, consignment, force)?;
            }
            CtlMsg::ProcessTransfer(ProcessReq {
                client_id,
                consignment,
                force,
            }) => {
                self.handle_consignment(endpoints, client_id, consignment, force)?;
            }

            CtlMsg::ConsignContract(ConsignReq {
                client_id,
                contract_id,
                include,
                outpoints,
                _phantom,
            }) => {
                self.handle_consign_contract(
                    endpoints,
                    client_id,
                    contract_id,
                    include,
                    outpoints,
                )?;
            }
            CtlMsg::ConsignTranfer(ConsignReq {
                client_id,
                contract_id,
                include,
                outpoints,
                _phantom,
            }) => {
                self.handle_consign_transfer(
                    endpoints,
                    client_id,
                    contract_id,
                    include,
                    outpoints,
                )?;
            }

            CtlMsg::OutpointState(OutpointStateReq {
                client_id,
                outpoints,
            }) => {
                self.handle_outpoint_state(endpoints, client_id, outpoints)?;
            }

            CtlMsg::FinalizeTransfer(FinalizeTransferReq {
                client_id,
                consignment,
                endseals,
                psbt,
                beneficiary,
            }) => {
                self.handle_finalize_transfer(
                    endpoints,
                    client_id,
                    consignment,
                    endseals,
                    psbt,
                    beneficiary,
                )?;
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
        force: bool,
    ) -> Result<(), DaemonError> {
        let id = consignment.consensus_commit();
        match self.process_consignment(consignment, force) {
            Err(err) => {
                let _ = self.send_rpc(endpoints, client_id, err);
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingFailed)?
            }
            Ok(status) => {
                // We ignore client reporting if it fails
                let msg = match status.validity() {
                    Validity::UnresolvedTransactions => {
                        RpcMsg::UnresolvedTxids(status.unresolved_txids.clone())
                    }
                    Validity::Invalid => RpcMsg::Invalid(status.clone()),
                    Validity::ValidExceptEndpoints if force => RpcMsg::Success(
                        s!("consumed notwithstanding non-mined endpoint transactions").into(),
                    ),
                    Validity::ValidExceptEndpoints => {
                        RpcMsg::UnresolvedTxids(status.unmined_endpoint_txids.clone())
                    }
                    Validity::Valid => RpcMsg::success(),
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

    fn handle_consign_contract(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract_id: ContractId,
        include: BTreeSet<TransitionType>,
        outpoints: OutpointFilter,
    ) -> Result<(), DaemonError> {
        match self.compose_consignment(contract_id, include, outpoints, ContractConsignment) {
            Err(err) => {
                let _ = self.send_rpc(endpoints, client_id, err);
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingFailed)?
            }
            Ok(consignment) => {
                let _ = self.send_rpc(endpoints, client_id, RpcMsg::Contract(consignment));
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingComplete)?
            }
        }
        Ok(())
    }

    fn handle_consign_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        contract_id: ContractId,
        include: BTreeSet<TransitionType>,
        outpoints: OutpointFilter,
    ) -> Result<(), DaemonError> {
        match self.compose_consignment(contract_id, include, outpoints, TransferConsignment) {
            Err(err) => {
                let _ = self.send_rpc(endpoints, client_id, err);
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingFailed)?
            }
            Ok(consignment) => {
                let _ = self.send_rpc(endpoints, client_id, RpcMsg::StateTransfer(consignment));
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingComplete)?
            }
        }
        Ok(())
    }

    fn handle_outpoint_state(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        outpoints: BTreeSet<OutPoint>,
    ) -> Result<(), DaemonError> {
        match self.outpoint_state(outpoints) {
            Err(err) => {
                let _ = self.send_rpc(endpoints, client_id, err);
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingFailed)?
            }
            Ok(transitions_info) => {
                let _ =
                    self.send_rpc(endpoints, client_id, RpcMsg::OutpointState(transitions_info));
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingComplete)?
            }
        }
        Ok(())
    }

    fn handle_finalize_transfer(
        &mut self,
        endpoints: &mut Endpoints,
        client_id: ClientId,
        consignment: StateTransfer,
        endseals: Vec<SealEndpoint>,
        psbt: Psbt,
        beneficiary: Option<NodeAddr>, // TODO: Replace with bool
    ) -> Result<(), DaemonError> {
        match self.finalize_transfer(consignment, endseals, psbt) {
            Err(err) => {
                let _ = self.send_rpc(endpoints, client_id, err);
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingFailed)?
            }
            Ok(transfer) => {
                if beneficiary.is_some() {
                    // TODO: Upload to stored database
                }
                let _ = self.send_rpc(endpoints, client_id, RpcMsg::StateTransfer(transfer));
                self.send_ctl(endpoints, ServiceId::Rgb, CtlMsg::ProcessingComplete)?
            }
        }
        Ok(())
    }
}
