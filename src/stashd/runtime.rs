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

use core::convert::TryFrom;
use futures::TryFutureExt;
use std::path::PathBuf;

use lnpbp::api::{Error, Multipart};
use lnpbp::service::*;

use super::Config;
use crate::BootstrapError;

use super::index::{BtreeIndex, Index};
#[cfg(not(store_hammersbald))] // Default store
use super::storage::{DiskStorage, DiskStorageConfig, Store};

pub struct Runtime {
    /// Original configuration object
    config: Config,

    /// Request-response API socket
    api_rep: zmq::Socket,

    /// Publish-subscribe API socket
    api_pub: zmq::Socket,

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
    #[cfg(and(store_hammersbald, not(any(store_disk))))]
    storage: HammersbaldStore,
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

    pub fn init(config: Config) -> Result<Self, BootstrapError> {
        #[cfg(not(store_hammersbald))] // Default store
        let storage = DiskStorage::new(DiskStorageConfig {
            data_dir: PathBuf::from(config.stash.clone()),
        })?;

        let indexer = BtreeIndex::new()?;

        let api_rep = context
            .socket(zmq::REP)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;
        api_rep
            .connect(&config.socket_req)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;

        let api_pub = context
            .socket(zmq::PUB)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;
        api_pub
            .connect(&config.socket_sub)
            .map_err(|e| BootstrapError::SubscriptionError(e))?;

        Ok(Self {
            config,
            api_rep,
            api_pub,
            indexer,
            storage,
        })
    }
}

#[async_trait]
impl TryService for Runtime {
    async fn run_loop(self) -> ! {
        loop {
            self.run().await?
        }
    }

    type ErrorType = Error;

    async fn try_run_loop(self) -> Result<!, Self::ErrorType> {
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
    fn run(&self) -> Result<(), Error> {
        let req: Multipart = self
            .subscriber
            .recv_multipart(0)
            .map_err(|err| Error::SocketError(err))?
            .into_iter()
            .map(zmq::Message::from)
            .collect();
        trace!("New API request");

        trace!("Received API request {:x?}, processing ... ", req[0]);
        let reply = self
            .proc_command(req)
            .inspect_err(|err| error!("Error processing request: {}", err))
            .await
            .unwrap_or(Reply::Failure);

        trace!(
            "Received response from command processor: `{}`; replying to client",
            reply
        );
        self.subscriber
            .send_multipart(Multipart::from(Reply::Success), 0)?;
        debug!("Sent reply {}", Reply::Success);

        Ok(())
    }

    async fn proc_command(&mut self, req: Multipart) -> Result<Reply, Error> {
        use Request::*;

        let command = Request::try_from(req)?;

        match command {
            Query(query) => self.command_query(query).await,
            _ => Err(Error::UnknownCommand),
        }
    }

    async fn command_query(&mut self, query: Query) -> Result<Reply, Error> {
        debug!("Got QUERY {}", query);

        // TODO: Do query processing

        Ok(Reply::Success)
    }
}
