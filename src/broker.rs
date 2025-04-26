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
use microservices::UThread;
use netservices::{NetAccept, service};
use rgb::{ContractId, Pile, Stockpile};
use rgbrpc::{ContractReply, Response};

use crate::rpc::RpcCmd;
use crate::services::{ContractsReader, ContractsWriter, ReaderReq, ReaderResp, ReplyMsg};
use crate::{Config, ReqId, RpcController};

#[derive(Debug, Display)]
#[display(lowercase)]
pub enum BrokerRpcMsg {
    ContractState(ContractId),
}

pub struct Broker<Sp: Stockpile>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Send,
    <Sp::Pile as Pile>::Seal: Send,
{
    rpc_runtime: service::Runtime<RpcCmd>,
    rpc_rx: Receiver<(ReqId, BrokerRpcMsg)>,

    reader_rx: Receiver<ReaderResp<<Sp::Pile as Pile>::Seal>>,
    reader_thread: UThread<ContractsReader<<Sp::Pile as Pile>::Seal>>,
    writer_thread: UThread<ContractsWriter<Sp>>,
}

impl<Sp> Broker<Sp>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Send,
    <Sp::Pile as Pile>::Seal: Send,
{
    pub fn start(conf: Config, stockpile: Sp) -> Result<Self, BrokerError> {
        const TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 10));

        log::info!("Starting contracts reader thread...");
        let (reader_tx, reader_rx) =
            crossbeam_channel::unbounded::<ReaderResp<<Sp::Pile as Pile>::Seal>>();
        let reader = ContractsReader::new(reader_tx);
        let reader_thread = UThread::new(reader, TIMEOUT);

        log::info!("Starting contracts writer thread...");
        let writer = ContractsWriter::new(stockpile, reader_thread.sender());
        let writer_thread = UThread::new(writer, TIMEOUT);

        log::info!("Starting the RPC server thread...");
        let (rpc_tx, rpc_rx) = crossbeam_channel::unbounded::<(ReqId, BrokerRpcMsg)>();
        let controller = RpcController::new(conf.network, rpc_tx.clone());
        let listen = conf.rpc.iter().map(|addr| {
            NetAccept::bind(addr).unwrap_or_else(|err| panic!("unable to bind to {addr}: {err}"))
        });
        let rpc_runtime = service::Runtime::new(conf.rpc[0], controller, listen)
            .map_err(|err| BrokerError::Rpc(err.into()))?;

        log::info!("Launch completed successfully");
        Ok(Self { rpc_runtime, rpc_rx, reader_rx, reader_thread, writer_thread })
    }

    pub fn run(mut self) -> Result<(), BrokerError> {
        select! {
            recv(self.rpc_rx) -> msg => {
                match msg {
                    Ok((req_id, msg)) => { self.proc_rpc_msg(req_id, msg).expect("unable to send a message"); },
                    Err(err) => {
                        log::error!("Error receiving RPC message: {err}");
                    }
                }
            },
            recv(self.reader_rx) -> msg => {
                match msg {
                    Ok(msg) => { self.proc_reader_msg(msg).expect("unable to send a message"); },
                    Err(err) => {
                        log::error!("Error receiving reader message: {err}");
                    }
                }
            }
        }

        self.rpc_runtime
            .join()
            .map_err(|_| BrokerError::Thread("RPC server"))?;
        self.reader_thread
            .join()
            .map_err(|_| BrokerError::Thread("Contracts reader"))?;
        self.writer_thread
            .join()
            .map_err(|_| BrokerError::Thread("Contracts writer"))?;
        Ok(())
    }

    fn proc_rpc_msg(&mut self, req_id: ReqId, msg: BrokerRpcMsg) -> io::Result<()> {
        log::debug!("Received an RPC message: {msg}");
        match msg {
            BrokerRpcMsg::ContractState(contract_id) => {
                if let Err(err) = self
                    .reader_thread
                    .sender()
                    .try_send(ReaderReq::ReadState(req_id, contract_id))
                {
                    log::error!("Unable to send a request to the reader thread: {err}");
                    self.send_rpc_resp(req_id, Response::NotFound(contract_id));
                }
            }
        }
        Ok(())
    }

    fn proc_reader_msg(&mut self, resp: ReaderResp<<Sp::Pile as Pile>::Seal>) -> io::Result<()> {
        log::debug!("Received reply from a reader for an RPC request {}", resp.req_id());
        match (resp.req_id(), resp.into_reply()) {
            (req_id, ReplyMsg::State(contract_id, state)) => {
                // TODO: Serialize state
                self.send_rpc_resp(
                    req_id,
                    Response::State(ContractReply { contract_id, data: none!() }),
                );
            }
            (req_id, ReplyMsg::NotFound(id)) => {
                self.send_rpc_resp(req_id, Response::NotFound(id));
            }
        }
        Ok(())
    }

    fn send_rpc_resp(&mut self, req_id: ReqId, response: Response) {
        if let Err(err) = self.rpc_runtime.cmd(RpcCmd::Send(req_id, response)) {
            log::error!("Channel to the RPC thread is broken: {err}");
            panic!("The channel to the RPC thread is broken. Unable to proceed, exiting.");
        };
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
