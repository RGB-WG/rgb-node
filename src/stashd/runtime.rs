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

use bp::dbc::Anchor;
use std::collections::BTreeSet;
use std::path::PathBuf;

use internet2::zmqsocket::ZmqType;
use internet2::{
    session, transport, CreateUnmarshaller, PlainTranscoder, Session,
    TypedEnum, Unmarshall, Unmarshaller,
};
use microservices::node::TryService;
use rgb::{
    Consignment, ContractId, Disclosure, Genesis, Node, NodeId, Schema,
    SchemaId, Stash,
};
use wallet::resolvers::ElectrumTxResolver;

use super::index::{BTreeIndex, Index};
#[cfg(not(store_hammersbald))] // Default store
use super::storage::{DiskStorage, DiskStorageConfig, Store};
use super::Config;
use crate::error::{
    BootstrapError, RuntimeError, ServiceError, ServiceErrorDomain,
    ServiceErrorSource,
};
use crate::rpc::stash::{AcceptRequest, Request, TransferRequest};
use crate::rpc::{reply, Reply};
use crate::stashd::index::BTreeIndexConfig;
use crate::util::ToBech32Data;

pub struct Runtime {
    /// Original configuration object
    config: Config,

    /// Request-response API socket
    rpc_server: session::Raw<PlainTranscoder, transport::zmqsocket::Connection>,

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
}

impl Runtime {
    /// Internal function for avoiding index-implementation specific function
    /// use and reduce number of errors. Indexer may be switched with compile
    /// configuration options and, thus, we need to make sure that the sturcture
    /// we use corresponds to certain trait and not specific type.
    fn indexer(&self) -> &impl Index {
        &self.indexer
    }

    pub fn storage(&self) -> &impl Store {
        &self.storage
    }

    pub fn init(config: Config) -> Result<Self, BootstrapError> {
        #[cfg(not(store_hammersbald))] // Default store
        let storage = DiskStorage::new(DiskStorageConfig {
            data_dir: PathBuf::from(config.stash.clone()),
        })?;

        let indexer = BTreeIndex::new(BTreeIndexConfig {
            index_dir: PathBuf::from(config.index.clone()),
            data_format: config.format,
        })?;

        let session_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Rep,
            &config.rpc_endpoint,
            None,
            None,
        )?;

        Ok(Self {
            config,
            rpc_server: session_rpc,
            indexer,
            storage,
            unmarshaller: Request::create_unmarshaller(),
        })
    }
}

impl TryService for Runtime {
    type ErrorType = RuntimeError;

    fn try_run_loop(mut self) -> Result<(), Self::ErrorType> {
        loop {
            match self.run() {
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
    fn run(&mut self) -> Result<(), RuntimeError> {
        trace!("Awaiting for ZMQ RPC requests...");
        let raw = self.rpc_server.recv_raw_message()?;
        let reply = self.rpc_process(raw).unwrap_or_else(|err| err);
        trace!("Preparing ZMQ RPC reply: {:?}", reply);
        let data = reply.serialize();
        trace!(
            "Sending {} bytes back to the client over ZMQ RPC: {}",
            data.len(),
            data.to_bech32data()
        );
        self.rpc_server.send_raw_message(&data)?;
        Ok(())
    }

    fn rpc_process(&mut self, raw: Vec<u8>) -> Result<Reply, Reply> {
        trace!(
            "Got {} bytes over ZMQ RPC: {:?}",
            raw.len(),
            raw.to_bech32data()
        );
        let message = &*self.unmarshaller.unmarshall(&raw).map_err(|err| {
            ServiceError::from_rpc(ServiceErrorSource::Stash, err)
        })?;
        debug!("Received ZMQ RPC request: {:?}", message);
        Ok(match message {
            Request::ListSchemata() => self.rpc_list_schemata(),
            Request::ListGeneses() => self.rpc_list_geneses(),
            Request::AddGenesis(genesis) => self.rpc_add_genesis(genesis),
            Request::AddSchema(schema) => self.rpc_add_schema(schema),
            Request::ReadGenesis(contract_id) => {
                self.rpc_read_genesis(contract_id)
            }
            Request::ReadSchema(schema_id) => self.rpc_read_schema(schema_id),
            Request::ReadTransitions(_) => unimplemented!(),
            Request::Transfer(consign) => self.rpc_transfer(consign),
            Request::Validate(consign) => self.rpc_validate(consign),
            Request::Accept(merge) => self.rpc_accept(merge),
            Request::Enclose(disclosure) => self.rpc_enclose(disclosure),
            Request::Forget(removal_list) => self.rpc_forget(removal_list),
        }
        .map_err(|err| ServiceError {
            domain: err,
            service: ServiceErrorSource::Stash,
        })?)
    }

    fn rpc_list_schemata(&mut self) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got LIST_SCHEMATA");
        let ids = self.storage.schema_ids()?;
        Ok(Reply::SchemaIds(ids))
    }

    fn rpc_list_geneses(&mut self) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got LIST_GENESES");
        let ids = self.storage.contract_ids()?;
        Ok(Reply::ContractIds(ids))
    }

    fn rpc_add_schema(
        &mut self,
        schema: &Schema,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ADD_SCHEMA {}", schema);
        self.storage.add_schema(schema)?;
        Ok(Reply::Success)
    }

    fn rpc_add_genesis(
        &mut self,
        genesis: &Genesis,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ADD_GENESIS {}", genesis);
        self.storage.add_genesis(genesis)?;
        Ok(Reply::Success)
    }

    fn rpc_read_genesis(
        &mut self,
        contract_id: &ContractId,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got READ_GENESIS {}", contract_id);
        let genesis = self.storage.genesis(contract_id)?;
        Ok(Reply::Genesis(genesis))
    }

    fn rpc_read_schema(
        &mut self,
        schema_id: &SchemaId,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got READ_SCHEMA {}", schema_id);
        let schema = self.storage.schema(schema_id)?;
        Ok(Reply::Schema(schema))
    }

    fn rpc_transfer(
        &mut self,
        request: &TransferRequest,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got TRANSFER {}", request);

        let mut transitions = request.other_transitions.clone();
        transitions.insert(request.contract_id, request.transition.clone());

        // Construct anchor
        let mut psbt = request.psbt.clone();
        let (anchors, map) = Anchor::commit(
            transitions
                .iter()
                .map(|(contract_id, ts)| (*contract_id, ts.node_id()))
                .collect(),
            &mut psbt,
        )
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
                &request.endpoints,
            )
            .map_err(|_| ServiceErrorDomain::Stash)?;

        // Prepare disclosure
        let mut disclosure = Disclosure::default();
        for (index, anchor) in anchors.into_iter().enumerate() {
            let contract_ids =
                map.iter()
                    .filter_map(|(contract_id, i)| {
                        if *i == index {
                            Some(contract_id)
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>();
            let anchored_transitions = transitions
                .clone()
                .into_iter()
                .filter(|(contract_id, _)| contract_ids.contains(contract_id))
                .collect();
            disclosure
                .insert_anchored_transitions(anchor, anchored_transitions);
        }

        Ok(Reply::Transfer(reply::Transfer {
            consignment,
            disclosure,
            witness: psbt,
        }))
    }

    fn rpc_validate(
        &mut self,
        consignment: &Consignment,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got VALIDATE CONSIGNMENT");

        let schema = self
            .storage()
            .schema(&consignment.genesis.schema_id())
            .map_err(|err| ServiceErrorDomain::Storage(err.to_string()))?;

        // [VALIDATION]: Validate genesis node against the scheme
        let electrum = ElectrumTxResolver::new(&self.config.electrum_server)
            .map_err(|_| ServiceErrorDomain::Electrum)?;
        let root_schema = self.storage().schema(&schema.root_id).ok();
        let validation_status =
            consignment.validate(&schema, root_schema.as_ref(), &electrum);

        self.storage.add_genesis(&consignment.genesis)?;

        Ok(Reply::ValidationStatus(validation_status))
    }

    fn rpc_accept(
        &mut self,
        accept_req: &AcceptRequest,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ACCEPT CONSIGNMENT");

        let known_seals = &accept_req.reveal_outpoints;
        let consignment = &accept_req.consignment;

        self.accept(consignment, known_seals)
            .map_err(|_| ServiceErrorDomain::Stash)?;

        Ok(Reply::Success)
    }

    fn rpc_enclose(
        &mut self,
        disclosure: &Disclosure,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got ENCLOSE DISCLOSURE");

        self.enclose(&disclosure)
            .map_err(|_| ServiceErrorDomain::Stash)?;

        Ok(Reply::Success)
    }

    fn rpc_forget(
        &mut self,
        _removal_list: &Vec<(NodeId, u16)>,
    ) -> Result<Reply, ServiceErrorDomain> {
        debug!("Got FORGET");

        // TODO #63: Implement stash prunning: filter all transitions containing
        //       revealed outpoints from the removal_list, and if they do
        //       not have any other _known_ outpoints, remove them — and iterate
        //       over their direct ancestor in the same manner

        Ok(Reply::Success)
    }
}

pub fn main_with_config(config: Config) -> Result<(), BootstrapError> {
    let runtime = Runtime::init(config)?;
    runtime.run_or_panic("Stashd runtime");

    unreachable!()
}
