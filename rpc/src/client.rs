// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::fs;
use std::path::PathBuf;

use internet2::addr::ServiceAddr;
use internet2::session::LocalSession;
use internet2::{
    CreateUnmarshaller, SendRecvMessage, TypedEnum, Unmarshall, Unmarshaller, ZmqSocketType,
};
use microservices::rpc::ServerError;
use microservices::ZMQ_CONTEXT;

use crate::messages::BusMsg;
use crate::{FailureCode, RpcMsg};

/// Final configuration resulting from data contained in config file environment
/// variables and command-line options. For security reasons node key is kept
/// separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    /// ZMQ socket for RPC API
    pub rpc_endpoint: ServiceAddr,

    /// Data location
    pub data_dir: PathBuf,

    /// Verbosity level
    pub verbose: u8,
}

pub struct Client {
    config: Config,
    // TODO: Replace with RpcSession once its implementation is completed
    session_rpc: LocalSession,
    unmarshaller: Unmarshaller<BusMsg>,
}

impl Client {
    pub fn with(config: Config) -> Result<Self, ServerError<FailureCode>> {
        debug!("Initializing runtime");
        trace!("Connecting to RGB daemon at {}", config.rpc_endpoint);
        let session_rpc = LocalSession::connect(
            ZmqSocketType::RouterConnect,
            &config.rpc_endpoint,
            None,
            None,
            &ZMQ_CONTEXT,
        )?;
        Ok(Self {
            config,
            session_rpc,
            unmarshaller: BusMsg::create_unmarshaller(),
        })
    }

    pub fn request(&mut self, request: RpcMsg) -> Result<RpcMsg, ServerError<FailureCode>> {
        trace!("Sending request to the server: {:?}", request);
        let data = BusMsg::from(request).serialize();
        trace!("Raw request data ({} bytes): {:02X?}", data.len(), data);
        self.session_rpc.send_raw_message(&data)?;
        trace!("Awaiting reply");
        let raw = self.session_rpc.recv_raw_message()?;
        trace!("Got reply ({} bytes), parsing: {:02X?}", raw.len(), raw);
        let reply = self.unmarshaller.unmarshall(raw.as_slice())?;
        trace!("Reply: {:?}", reply);
        match &*reply {
            BusMsg::Rpc(rpc) => Ok(rpc.clone()),
        }
    }
}

impl Config {
    pub fn process(&mut self) {
        self.data_dir =
            PathBuf::from(shellexpand::tilde(&self.data_dir.display().to_string()).to_string());

        let me = self.clone();
        let mut data_dir = self.data_dir.to_string_lossy().into_owned();
        self.process_dir(&mut data_dir);
        self.data_dir = PathBuf::from(data_dir);

        fs::create_dir_all(&self.data_dir).expect("Unable to access data directory");

        for dir in vec![&mut self.rpc_endpoint] {
            if let ServiceAddr::Ipc(ref mut path) = dir {
                me.process_dir(path);
            }
        }
    }

    pub fn process_dir(&self, path: &mut String) {
        *path = path.replace("{data_dir}", &self.data_dir.to_string_lossy());
        *path = shellexpand::tilde(path).to_string();
    }
}
