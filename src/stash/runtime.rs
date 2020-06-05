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

use std::io;
use std::path::PathBuf;

use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::zmq::ApiType;
use lnpbp::lnp::{transport, NoEncryption, Session, Unmarshall, Unmarshaller};
use lnpbp::rgb::{Genesis, Schema};
use lnpbp::TryService;

use super::Command;
use super::Config;
use crate::api::Reply;
use crate::error::{
    BootstrapError, RuntimeError, ServiceError, ServiceErrorDomain, ServiceErrorSource,
};

use super::index::{BtreeIndex, Index};
#[cfg(not(store_hammersbald))] // Default store
use super::storage::{DiskStorage, DiskStorageConfig, Store};

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
    indexer: BtreeIndex,

    /// RGB Stash data storage: high-volume on-disk key-value storage with
    /// large binary blob values. Fast read, slow write, no delete db.
    /// Must be exclusive for the current service and must not be used
    /// from anywhere else. The disk storage must be locked for exclusive
    /// access.
    #[cfg(not(store_hammersbald))] // Default store
    storage: DiskStorage,
    #[cfg(all(store_hammersbald, not(any(store_disk))))]
    storage: HammersbaldStore,

    /// Unmarshaller instance used for parsing RPC request
    unmarshaller: Unmarshaller<Command>,
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

        let indexer = BtreeIndex::new();

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

        Ok(Self {
            config,
            session_rpc,
            session_pub,
            indexer,
            storage,
            unmarshaller: Command::create_unmarshaller(),
        })
    }
}

#[async_trait]
impl TryService for Runtime {
    type ErrorType = RuntimeError;

    async fn try_run_loop(mut self) -> Result<!, Self::ErrorType> {
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
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = match self.rpc_process(raw).await {
            Ok(_) => Reply::Success,
            Err(err) => Reply::Failure(format!("{}", err)),
        };
        let data = reply.encode()?;
        self.session_rpc.send_raw_message(data)?;
        Ok(())
    }

    async fn rpc_process(&mut self, raw: Vec<u8>) -> Result<(), ServiceError> {
        let message = &*self
            .unmarshaller
            .unmarshall(&raw)
            .map_err(|err| ServiceError::from_rpc(ServiceErrorSource::Stash, err))?;
        match message {
            Command::AddGenesis(genesis) => self.rpc_add_genesis(genesis).await,
            Command::AddSchema(schema) => self.rpc_add_schema(schema).await,
            _ => unimplemented!(),
        }
        .map_err(|err| ServiceError {
            domain: err,
            service: ServiceErrorSource::Stash,
        })
    }

    async fn rpc_add_schema(&mut self, schema: &Schema) -> Result<(), ServiceErrorDomain> {
        self.storage.add_schema(schema)?;
        Ok(())
    }

    async fn rpc_add_genesis(&mut self, genesis: &Genesis) -> Result<(), ServiceErrorDomain> {
        self.storage.add_genesis(genesis)?;
        Ok(())
    }
}

pub async fn main_with_config(config: Config) -> Result<(), BootstrapError> {
    let mut context = zmq::Context::new();
    let runtime = Runtime::init(config, &mut context)?;
    runtime.run_or_panic("Stashd runtime").await
}
