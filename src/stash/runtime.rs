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

use std::path::PathBuf;

use lnpbp::bitcoin::{Transaction, Txid};
use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::zmq::ApiType;
use lnpbp::lnp::{transport, NoEncryption, Session, Unmarshall, Unmarshaller};
use lnpbp::rgb::{
    validation, Anchor, Assignments, Consignment, ContractId, Genesis, Node, NodeId, Schema,
    SchemaId, Stash, Validity,
};

use super::electrum::ElectrumTxResolver;
use super::index::{BTreeIndex, Index};
#[cfg(not(store_hammersbald))] // Default store
use super::storage::{DiskStorage, DiskStorageConfig, Store};
use super::Config;
use crate::api::stash::{ConsignRequest, MergeRequest, Request};
use crate::api::{reply, Reply};
use crate::error::{
    BootstrapError, RuntimeError, ServiceError, ServiceErrorDomain, ServiceErrorSource,
};
use crate::service::TryService;
use crate::stash::index::BTreeIndexConfig;

pub struct Runtime {
    /// Original configuration object
    config: Config,

    /// Request-response API socket
    session_rpc: Session<NoEncryption, transport::zmq::Connection>,

    /// Publish-subscribe API socket
    session_pub: Session<NoEncryption, transport::zmq::Connection>,

    /// RGB Index: fast, mostly in-memory key-value indexing service.
    /// Must be exclusive for the current service
    // Here we use default indexer. When other indexers will be implemented,
    // they will be compile-time switched with `--cfg` options like
    // `--cfg "index_memcached"`
    pub(super) indexer: BTreeIndex,

    /// RGB Stash data storage: high-volume on-disk key-value storage with
    /// large binary blob values. Fast read, slow write, no delete db.
    /// Must be exclusive for the current service and must not be used
    /// from anywhere else. The disk storage must be locked for exclusive
    /// access.
    #[cfg(not(store_hammersbald))] // Default store
    pub(super) storage: DiskStorage,
    #[cfg(all(store_hammersbald, not(any(store_disk))))]
    pub(super) storage: HammersbaldStore,

    /// Unmarshaller instance used for parsing RPC request
    unmarshaller: Unmarshaller<Request>,

    /// Electrum client handle to fetch transactions
    electrum: ElectrumTxResolver,
}

impl Runtime {
    /// Internal function for avoiding index-implementation specific function
    /// use and reduce number of errors. Indexer may be switched with compile
    /// configuration options and, thus, we need to make sure that the sturcture
    /// we use corresponds to certain trait and not specific type.
    fn indexer(&self) -> &impl Index {
        &self.indexer
    }

    fn storage(&self) -> &impl Store {
        &self.storage
    }

    pub fn init(config: Config, mut context: &mut zmq::Context) -> Result<Self, BootstrapError> {
        #[cfg(not(store_hammersbald))] // Default store
        let storage = DiskStorage::new(DiskStorageConfig {
            data_dir: PathBuf::from(config.stash.clone()),
        })?;

        let indexer = BTreeIndex::load(BTreeIndexConfig {
            index_file: PathBuf::from(config.index.clone()),
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

        let electrum = ElectrumTxResolver::new(&config.electrum_server)?;

        Ok(Self {
            config,
            session_rpc,
            session_pub,
            indexer,
            storage,
            unmarshaller: Request::create_unmarshaller(),
            electrum,
        })
    }
}

#[async_trait]
impl TryService for Runtime {
    type ErrorType = RuntimeError;

    async fn try_run_loop(mut self) -> Result<(), Self::ErrorType> {
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
            Request::ListSchemata() => self.rpc_list_schemata().await,
            Request::ListGeneses() => self.rpc_list_geneses().await,
            Request::AddGenesis(genesis) => self.rpc_add_genesis(genesis).await,
            Request::AddSchema(schema) => self.rpc_add_schema(schema).await,
            Request::ReadGenesis(contract_id) => self.rpc_read_genesis(contract_id).await,
            Request::ReadSchema(schema_id) => self.rpc_read_schema(schema_id).await,
            Request::Consign(consign) => self.rpc_consign(consign).await,
            Request::Validate(consign) => self.rpc_validate(consign).await,
            Request::Merge(merge) => self.rpc_merge(merge).await,
            Request::Forget(removal_list) => self.rpc_forget(removal_list).await,
            _ => unimplemented!(),
        }
        .map_err(|err| ServiceError {
            domain: err,
            service: ServiceErrorSource::Stash,
        })?)
    }

    async fn rpc_list_schemata(&mut self) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got LIST_SCHEMATA");
        let ids = self.storage.schema_ids()?;
        Ok(Reply::SchemaIds(ids))
    }

    async fn rpc_list_geneses(&mut self) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got LIST_GENESES");
        let ids = self.storage.contract_ids()?;
        Ok(Reply::ContractIds(ids))
    }

    async fn rpc_add_schema(&mut self, schema: &Schema) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ADD_SCHEMA {}", schema);
        self.storage.add_schema(schema)?;
        Ok(Reply::Success)
    }

    async fn rpc_add_genesis(&mut self, genesis: &Genesis) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ADD_GENESIS {}", genesis);
        self.storage.add_genesis(genesis)?;
        Ok(Reply::Success)
    }

    async fn rpc_read_genesis(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got READ_GENESIS {}", contract_id);
        let genesis = self.storage.genesis(contract_id)?;
        Ok(Reply::Genesis(genesis))
    }

    async fn rpc_read_schema(&mut self, schema_id: &SchemaId) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got READ_SCHEMA {}", schema_id);
        let schema = self.storage.schema(schema_id)?;
        Ok(Reply::Schema(schema))
    }

    async fn rpc_consign(&mut self, request: &ConsignRequest) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got CONSIGN {}", request);

        let mut transitions = request.other_transition_ids.clone();
        transitions.insert(request.contract_id, request.transition.node_id());

        // Construct anchor
        let mut psbt = request.psbt.clone();
        let (anchors, map) = Anchor::commit(transitions, &mut psbt)
            .map_err(|err| ServiceErrorDomain::Anchor(format!("{}", err)))?;
        let anchor = anchors[*map
            .get(&request.contract_id)
            .expect("Core LNP/BP anchor commitment procedure is broken")]
        .clone();

        // Prepare consignments: extract from stash storage the required data
        // and assemble them into a consignment
        let consignment = self
            .consign(
                request.contract_id,
                &request.transition,
                Some(&anchor),
                &request.outpoints.clone(),
            )
            .map_err(|_| ServiceErrorDomain::Stash)?;

        Ok(Reply::Transfer(reply::Transfer { consignment, psbt }))
    }

    async fn rpc_validate(
        &mut self,
        consignment: &Consignment,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got VALIDATE CONSIGNMENT");

        let schema = self
            .storage()
            .schema(&consignment.genesis.schema_id())
            .map_err(|err| ServiceErrorDomain::Storage(err.to_string()))?;

        // [VALIDATION]: Validate genesis node against the scheme
        let validation_status = consignment.validate(&schema, &self.electrum);

        self.storage.add_genesis(&consignment.genesis)?;

        match validation_status.validity() {
            Validity::Valid => Ok(Reply::Success),
            Validity::UnresolvedTransactions => Ok(Reply::Failure(reply::Failure {
                code: 1,
                info: format!("{:?}", validation_status.unresolved_txids),
            })),
            Validity::Invalid => Ok(Reply::Failure(reply::Failure {
                code: 2,
                info: format!("{:?}", validation_status.failures),
            })),
        }
        // TODO: Return this type of reply when StrictEncoding will be
        //       implemented for validation::Status
        //Ok(Reply::ValidationStatus(validation_status))
    }

    async fn rpc_merge(&mut self, merge: &MergeRequest) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got MERGE CONSIGNMENT");

        let known_seals = &merge.reveal_outpoints;

        // [PRIVACY]:
        // Update transition data with the revealed state information that we
        // kept since we did an invoice (and the sender did not know).
        let reveal_known_seals = |(_, assignments): (&usize, &mut Assignments)| match assignments {
            Assignments::Declarative(_) => {}
            Assignments::DiscreteFiniteField(set) => {
                *set = set
                    .iter()
                    .map(|a| {
                        let mut a = a.clone();
                        a.reveal_seals(known_seals.iter());
                        a
                    })
                    .collect();
            }
            Assignments::CustomData(set) => {
                *set = set
                    .iter()
                    .map(|a| {
                        let mut a = a.clone();
                        a.reveal_seals(known_seals.iter());
                        a
                    })
                    .collect();
            }
        };

        for (anchor, transition) in &merge.consignment.state_transitions {
            let mut transition = transition.clone();
            transition
                .owned_rights_mut()
                .into_iter()
                .for_each(reveal_known_seals);
            // Store the transition and the anchor data in the stash
            self.storage.add_anchor(&anchor)?;
            self.storage.add_transition(&transition)?;
        }

        for extension in &merge.consignment.state_extensions {
            let mut extension = extension.clone();
            extension
                .owned_rights_mut()
                .into_iter()
                .for_each(reveal_known_seals);
            self.storage.add_extension(&extension)?;
        }

        Ok(Reply::Success)
    }

    async fn rpc_forget(
        &mut self,
        _removal_list: &Vec<(NodeId, u16)>,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got FORGET");

        // TODO: Implement stash prunning: filter all transitions containing
        //       revealed outpoints from the removal_list, and if they do
        //       not have any other _known_ outpoints, remove them — and iterate
        //       over their direct ancestor in the same manner

        Ok(Reply::Success)
    }
}

struct DummyTxResolver;

impl validation::TxResolver for DummyTxResolver {
    fn resolve(
        &self,
        _txid: &Txid,
    ) -> Result<Option<(Transaction, u64)>, validation::TxResolverError> {
        Err(validation::TxResolverError)
    }
}

pub async fn main_with_config(config: Config) -> Result<(), BootstrapError> {
    let mut context = zmq::Context::new();
    let runtime = Runtime::init(config, &mut context)?;
    runtime.run_or_panic("Stashd runtime").await;

    unreachable!()
}
