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
use lnpbp::TryService;

use super::cache::{Cache, FileCache, FileCacheConfig};
use super::{Command, Config, Processor};
use crate::api::{fungible::Issue, Reply};
use crate::error::{
    ApiErrorType, BootstrapError, RuntimeError, ServiceError, ServiceErrorDomain,
    ServiceErrorSource,
};
use crate::fungible::IssueStructure;

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
    unmarshaller: Unmarshaller<Command>,
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
            unmarshaller: Command::create_unmarshaller(),
        })
    }
}

#[async_trait]
impl TryService for Runtime {
    type ErrorType = RuntimeError;

    async fn try_run_loop(mut self) -> Result<!, RuntimeError> {
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
        let mut cursor = io::Cursor::new(vec![]);
        reply.encode(&mut cursor)?;
        let data = cursor.into_inner();
        self.session_rpc.send_raw_message(data)?;
        Ok(())
    }

    async fn rpc_process(&mut self, raw: Vec<u8>) -> Result<(), ServiceError> {
        let mut cursor = io::Cursor::new(raw);
        let message = &*self
            .unmarshaller
            .unmarshall(&mut cursor)
            .map_err(|err| ServiceError::from_rpc(ServiceErrorSource::Stash, err))?;
        match message {
            Command::Issue(issue) => self.rpc_issue(issue).await,
        }
        .map_err(|err| ServiceError::contract(err, "fungible"))
    }

    async fn rpc_issue(&mut self, issue: &Issue) -> Result<(), ServiceErrorDomain> {
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

        let (asset, _genesis) = self.processor.issue(
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

        // TODO: Save asset and genesis by sending a message to stashd
        self.cacher.add_asset(asset)?;

        // TODO: Send push request to client informing about cache update

        Ok(())
    }
}
