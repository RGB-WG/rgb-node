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

use ::core::borrow::Borrow;
use ::core::convert::TryFrom;
use ::std::path::PathBuf;

use lnpbp::bitcoin::OutPoint;
use lnpbp::client_side_validation::Conceal;
use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::zmq::ApiType;
use lnpbp::lnp::{transport, NoEncryption, Session, Unmarshall, Unmarshaller};
use lnpbp::rgb::{AssignmentsVariant, Consignment, ContractId, Genesis, Node};
use lnpbp::TryService;

use super::cache::{Cache, FileCache, FileCacheConfig};
use super::schema::AssignmentsType;
use super::{schema, Asset, Config, IssueStructure, Processor};
use crate::api::stash::MergeRequest;
use crate::api::{
    self,
    fungible::{AcceptApi, Issue, Request, TransferApi},
    reply,
    stash::ConsignRequest,
    Reply,
};
use crate::error::{
    ApiErrorType, BootstrapError, RuntimeError, ServiceError, ServiceErrorDomain,
    ServiceErrorSource,
};

pub struct Runtime {
    /// Original configuration object
    config: Config,

    /// Request-response API session
    session_rpc: Session<NoEncryption, transport::zmq::Connection>,

    /// Publish-subscribe API session
    session_pub: Session<NoEncryption, transport::zmq::Connection>,

    /// Stash RPC client session
    stash_rpc: Session<NoEncryption, transport::zmq::Connection>,

    /// Publish-subscribe API socket
    stash_sub: Session<NoEncryption, transport::zmq::Connection>,

    /// RGB fungible assets data cache: relational database sharing the client-
    /// friendly asset information with clients
    cacher: FileCache,

    /// Processor instance: handles business logic outside of stash scope
    processor: Processor,

    /// Unmarshaller instance used for parsing RPC request
    unmarshaller: Unmarshaller<Request>,

    /// Unmarshaller instance used for parsing RPC request
    reply_unmarshaller: Unmarshaller<Reply>,
}

impl Runtime {
    /// Internal function for avoiding index-implementation specific function
    /// use and reduce number of errors. Cacher may be switched with compile
    /// configuration options and, thus, we need to make sure that the structure
    /// we use corresponds to certain trait and not specific type.
    fn cache(&self) -> &impl Cache {
        &self.cacher
    }

    pub fn init(config: Config, mut context: &mut zmq::Context) -> Result<Self, BootstrapError> {
        let processor = Processor::new()?;

        let cacher = FileCache::new(FileCacheConfig {
            data_dir: PathBuf::from(&config.cache),
            data_format: config.format,
        })
        .map_err(|err| {
            error!("{}", err);
            err
        })?;

        let session_rpc = Session::new_zmq_unencrypted(
            ApiType::Server,
            &mut context,
            config.rpc_endpoint.clone(),
            None,
        )?;

        let session_pub = Session::new_zmq_unencrypted(
            ApiType::Publish,
            &mut context,
            config.pub_endpoint.clone(),
            None,
        )?;

        let stash_rpc = Session::new_zmq_unencrypted(
            ApiType::Client,
            &mut context,
            config.stash_rpc.clone(),
            None,
        )?;

        let stash_sub = Session::new_zmq_unencrypted(
            ApiType::Subscribe,
            &mut context,
            config.stash_sub.clone(),
            None,
        )?;

        Ok(Self {
            config,
            session_rpc,
            session_pub,
            stash_rpc,
            stash_sub,
            cacher,
            processor,
            unmarshaller: Request::create_unmarshaller(),
            reply_unmarshaller: Reply::create_unmarshaller(),
        })
    }
}

#[async_trait]
impl TryService for Runtime {
    type ErrorType = RuntimeError;

    async fn try_run_loop(mut self) -> Result<!, RuntimeError> {
        debug!("Registering RGB20 schema");
        self.register_schema().await.map_err(|_| {
            error!("Unable to register RGB20 schema");
            RuntimeError::Internal("Unable to register RGB20 schema".to_string())
        })?;

        loop {
            match self.run().await {
                Ok(_) => debug!("API request processing complete"),
                Err(err) => {
                    error!("Error processing API request: {}", err);
                    Err(err)?;
                }
            }
        }
    }
}

impl Runtime {
    async fn run(&mut self) -> Result<(), RuntimeError> {
        trace!("Awaiting for ZMQ RPC requests...");
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.rpc_process(raw).await.unwrap_or_else(|err| err);
        trace!("Preparing ZMQ RPC reply: {:?}", reply);
        let data = reply.encode()?;
        trace!(
            "Sending {} bytes back to the client over ZMQ RPC",
            data.len()
        );
        self.session_rpc.send_raw_message(data)?;
        Ok(())
    }

    async fn rpc_process(&mut self, raw: Vec<u8>) -> Result<Reply, Reply> {
        trace!("Got {} bytes over ZMQ RPC: {:?}", raw.len(), raw);
        let message = &*self
            .unmarshaller
            .unmarshall(&raw)
            .map_err(|err| ServiceError::from_rpc(ServiceErrorSource::Stash, err))?;
        debug!("Received ZMQ RPC request: {:?}", message);
        Ok(match message {
            Request::Issue(issue) => self.rpc_issue(issue).await,
            Request::Transfer(transfer) => self.rpc_transfer(transfer).await,
            Request::Validate(consignment) => self.rpc_validate(consignment).await,
            Request::Accept(accept) => self.rpc_accept(accept).await,
            Request::Forget(outpoint) => self.rpc_forget(outpoint).await,
            Request::ImportAsset(genesis) => self.rpc_import_asset(genesis).await,
            Request::ExportAsset(asset_id) => self.rpc_export_asset(asset_id).await,
            Request::Sync => self.rpc_sync().await,
        }
        .map_err(|err| ServiceError::contract(err, "fungible"))?)
    }

    async fn rpc_issue(&mut self, issue: &Issue) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ISSUE {}", issue);

        let issue_structure = match issue.inflatable {
            None => IssueStructure::SingleIssue,
            Some(ref seal_spec) => IssueStructure::MultipleIssues {
                max_supply: issue.supply.ok_or(ServiceErrorDomain::Api(
                    ApiErrorType::MissedArgument {
                        request: "Issue".to_string(),
                        argument: "supply".to_string(),
                    },
                ))?,
                reissue_control: seal_spec.clone(),
            },
        };

        let (asset, genesis) = self.processor.issue(
            self.config.network,
            issue.ticker.clone(),
            issue.title.clone(),
            issue.description.clone(),
            issue_structure,
            issue.allocate.clone(),
            issue.precision,
            vec![],
            issue.dust_limit,
        )?;

        self.import_asset(asset, genesis).await?;

        // TODO: Send push request to client informing about cache update

        Ok(Reply::Success)
    }

    async fn rpc_transfer(&mut self, transfer: &TransferApi) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got TRANSFER {}", transfer);

        // TODO: Check inputs that they really exist and have sufficient amount of
        //       asset for the transfer operation

        let mut asset = self.cacher.asset(transfer.contract_id)?.clone();

        let transition = self.processor.transfer(
            &mut asset,
            transfer.inputs.clone(),
            transfer.ours.clone(),
            transfer.theirs.clone(),
        )?;

        let reply = self
            .consign(ConsignRequest {
                contract_id: transfer.contract_id,
                inputs: transfer.inputs.clone(),
                transition,
                // TODO: Collect blank state transitions and pass it here
                other_transition_ids: bmap![],
                outpoints: transfer
                    .theirs
                    .iter()
                    .map(|o| (o.seal_confidential))
                    .collect(),
                psbt: transfer.psbt.clone(),
            })
            .await?;

        Ok(reply)
    }

    async fn rpc_validate(
        &mut self,
        consignment: &Consignment,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got VALIDATE");
        self.validate(consignment.clone()).await?;
        Ok(Reply::Success)
    }

    async fn rpc_accept(&mut self, accept: &AcceptApi) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ACCEPT");
        Ok(self.accept(accept.clone()).await?)
    }

    async fn rpc_forget(&mut self, outpoint: &OutPoint) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got FORGET");
        Ok(self.forget(outpoint.clone()).await?)
    }

    async fn rpc_sync(&mut self) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got SYNC");
        let data = self.cacher.export()?;
        Ok(Reply::Sync(reply::SyncFormat(self.config.format, data)))
    }

    async fn rpc_import_asset(&mut self, genesis: &Genesis) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got IMPORT_ASSET");
        self.import_asset(Asset::try_from(genesis.clone())?, genesis.clone())
            .await?;
        Ok(Reply::Success)
    }

    async fn rpc_export_asset(
        &mut self,
        asset_id: &ContractId,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got EXPORT_ASSET");
        let genesis = self.export_asset(asset_id.clone()).await?;
        Ok(Reply::Genesis(genesis))
    }

    async fn register_schema(&mut self) -> Result<(), ServiceErrorDomain> {
        match self
            .stash_req_rep(api::stash::Request::AddSchema(schema::schema()))
            .await?
        {
            Reply::Success => Ok(()),
            _ => Err(ServiceErrorDomain::Api(ApiErrorType::UnexpectedReply)),
        }
    }

    async fn import_asset(
        &mut self,
        asset: Asset,
        genesis: Genesis,
    ) -> Result<bool, ServiceErrorDomain> {
        match self
            .stash_req_rep(api::stash::Request::AddGenesis(genesis))
            .await?
        {
            Reply::Success => Ok(self.cacher.add_asset(asset)?),
            _ => Err(ServiceErrorDomain::Api(ApiErrorType::UnexpectedReply)),
        }
    }

    async fn export_asset(&mut self, asset_id: ContractId) -> Result<Genesis, ServiceErrorDomain> {
        match self
            .stash_req_rep(api::stash::Request::ReadGenesis(asset_id))
            .await?
        {
            Reply::Genesis(genesis) => Ok(genesis.clone()),
            _ => Err(ServiceErrorDomain::Api(ApiErrorType::UnexpectedReply)),
        }
    }

    async fn consign(&mut self, consign_req: ConsignRequest) -> Result<Reply, ServiceErrorDomain> {
        let reply = self
            .stash_req_rep(api::stash::Request::Consign(consign_req))
            .await?;
        if let Reply::Transfer(_) = reply {
            Ok(reply)
        } else {
            Err(ServiceErrorDomain::Api(ApiErrorType::UnexpectedReply))
        }
    }

    async fn validate(&mut self, consignment: Consignment) -> Result<Reply, ServiceErrorDomain> {
        let reply = self
            .stash_req_rep(api::stash::Request::Validate(consignment))
            .await?;

        match reply {
            Reply::Success | Reply::Failure(_) => Ok(reply),
            _ => Err(ServiceErrorDomain::Api(ApiErrorType::UnexpectedReply)),
        }
    }

    async fn accept(&mut self, accept: AcceptApi) -> Result<Reply, ServiceErrorDomain> {
        let reply = self
            .stash_req_rep(api::stash::Request::Merge(MergeRequest {
                consignment: accept.consignment.clone(),
                reveal_outpoints: accept.reveal_outpoints.clone(),
            }))
            .await?;
        if let Reply::Success = reply {
            let asset_id = accept.consignment.genesis.contract_id();
            let mut asset = if self.cacher.has_asset(asset_id)? {
                self.cacher.asset(asset_id)?.clone()
            } else {
                Asset::try_from(accept.consignment.genesis)?
            };

            for (_, transition) in &accept.consignment.data {
                let set = transition.assignments_by_type(-AssignmentsType::Assets);
                for variant in set {
                    if let AssignmentsVariant::DiscreteFiniteField(set) = variant {
                        for (index, assignment) in set.into_iter().enumerate() {
                            if let Some(seal) = accept.reveal_outpoints.iter().find(|op| {
                                op.conceal() == assignment.seal_definition_confidential()
                            }) {
                                if let Some(assigned_state) = assignment.assigned_state() {
                                    asset.add_allocation(
                                        seal.clone().into(),
                                        transition.node_id(),
                                        index as u16,
                                        assigned_state.clone(),
                                    );
                                } else {
                                    Err(ServiceErrorDomain::Internal(
                                        "Consignment structure is broken".to_string(),
                                    ))?
                                }
                            }
                        }
                    }
                }
            }

            self.cacher.add_asset(asset)?;
            Ok(reply)
        } else if let Reply::Failure(_) = &reply {
            Ok(reply)
        } else {
            Err(ServiceErrorDomain::Api(ApiErrorType::UnexpectedReply))
        }
    }

    async fn forget(&mut self, outpoint: OutPoint) -> Result<Reply, ServiceErrorDomain> {
        let mut removal_list = Vec::<_>::new();
        let assets = self
            .cacher
            .assets()?
            .into_iter()
            .map(Clone::clone)
            .collect::<Vec<_>>();
        for asset in assets {
            let mut asset = asset.clone();
            for allocation in asset
                .clone()
                .allocations(&outpoint)
                .ok_or(ServiceErrorDomain::Cache)?
            {
                asset.remove_allocation(
                    outpoint,
                    allocation.node_id,
                    allocation.index,
                    allocation.amount.clone(),
                );
                removal_list.push((allocation.node_id, allocation.index));
            }
            self.cacher.add_asset(asset)?;
        }
        if removal_list.is_empty() {
            return Ok(Reply::Nothing);
        }

        let reply = self
            .stash_req_rep(api::stash::Request::Forget(removal_list))
            .await?;

        match reply {
            Reply::Success | Reply::Failure(_) => Ok(reply),
            _ => Err(ServiceErrorDomain::Api(ApiErrorType::UnexpectedReply)),
        }
    }

    async fn stash_req_rep(
        &mut self,
        request: api::stash::Request,
    ) -> Result<Reply, ServiceErrorDomain> {
        let data = request.encode()?;
        self.stash_rpc.send_raw_message(data.borrow())?;
        let raw = self.stash_rpc.recv_raw_message()?;
        let reply = &*self.reply_unmarshaller.unmarshall(&raw)?.clone();
        if let Reply::Failure(ref failmsg) = reply {
            error!("Stash daemon has returned failure code: {}", failmsg);
            Err(ServiceErrorDomain::Stash)?
        }
        Ok(reply.clone())
    }
}

pub async fn main_with_config(config: Config) -> Result<(), BootstrapError> {
    let mut context = zmq::Context::new();
    let runtime = Runtime::init(config, &mut context)?;
    runtime.run_or_panic("Fungible contract runtime").await
}
