// RGB Node: sovereign smart contracts backend
//
// SPDX-License-Identifier: Apache-2.0
//
// Designed in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
// Written in 2020-2025 by Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2020-2024 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2025 RGB Consortium, Switzerland. All rights reserved.
// Copyright (C) 2020-2025 Dr Maxim Orlovsky. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use std::io;
use std::time::Duration;

use amplify::IoError;
use crossbeam_channel::{Receiver, select};
use netservices::{NetAccept, service};

use crate::rpc::RpcCmd;
use crate::{Config, RpcController};

#[derive(Debug, Display)]
#[display(lowercase)]
pub enum BrokerRpcMsg {
    Noop,
}

pub struct Broker {
    rpc: service::Runtime<RpcCmd>,
    rpc_rx: Receiver<BrokerRpcMsg>,
}

impl Broker {
    pub fn start(conf: Config) -> Result<Self, BrokerError> {
        const TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 10));

        log::info!("Starting the RPC server thread...");
        let (rpc_tx, rpc_rx) = crossbeam_channel::unbounded::<BrokerRpcMsg>();
        let controller = RpcController::new(conf.network, rpc_tx.clone());
        let listen = conf.rpc.iter().map(|addr| {
            NetAccept::bind(addr).unwrap_or_else(|err| panic!("unable to bind to {addr}: {err}"))
        });
        let rpc = service::Runtime::new(conf.rpc[0].clone(), controller, listen)
            .map_err(|err| BrokerError::Rpc(err.into()))?;

        log::info!("Launch completed successfully");
        Ok(Self { rpc, rpc_rx })
    }

    pub fn run(mut self) -> Result<(), BrokerError> {
        select! {
            recv(self.rpc_rx) -> msg => {
                match msg {
                    Ok(msg) => { self.proc_rpc_msg(msg).expect("unable to send message"); },
                    Err(err) => {
                        log::error!("Error receiving RPC message: {err}");
                    }
                }
            }
        }

        self.rpc
            .join()
            .map_err(|_| BrokerError::Thread("RPC server"))?;
        Ok(())
    }

    pub fn proc_rpc_msg(&mut self, msg: BrokerRpcMsg) -> io::Result<()> {
        log::debug!("Received an RPC message: {msg}");
        match msg {
            BrokerRpcMsg::Noop => {}
        }
        Ok(())
    }
}

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum BrokerError {
    /// unable to initialize RPC service.
    ///
    /// {0}
    Rpc(IoError),

    /// unable to initialize importing service.
    ///
    /// {0}
    Import(IoError),

    /// unable to initialize block importing service.
    ///
    /// {0}
    Importer(IoError),

    /// unable to create thread for {0}.
    Thread(&'static str),
}
