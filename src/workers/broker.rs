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
#[cfg(feature = "embedded")]
use std::thread::JoinHandle;
use std::time::Duration;

use amplify::IoError;
use bpstd::seals::TxoSeal;
#[cfg(feature = "embedded")]
use crossbeam_channel::Sender;
use crossbeam_channel::{Receiver, select};
use microservices::UThread;
#[cfg(feature = "server")]
use netservices::{NetAccept, service};
use rgb::{Pile, Stockpile};
use rgbrpc::{Failure, RgbRpcReq, RgbRpcResp};

#[cfg(feature = "server")]
use crate::Dispatcher;
#[cfg(feature = "embedded")]
use crate::services::{AsyncClient, AsyncDispatcher};
use crate::services::{Reader2Broker, ReaderMsg, ReaderService, Request2Reader, WriterService};
use crate::{Config, ReqId};

pub struct Broker<Sp: Stockpile>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Pile<Seal = TxoSeal> + Send,
{
    #[cfg(feature = "embedded")]
    rpc_thread: UThread<AsyncDispatcher>,
    #[cfg(feature = "embedded")]
    rpc_tx: Sender<(ReqId, RgbRpcResp)>,

    #[cfg(not(feature = "embedded"))]
    rpc_thread: service::Runtime<(ReqId, RgbRpcResp)>,
    rpc_rx: Receiver<(ReqId, RgbRpcReq)>,

    reader_rx: Receiver<Reader2Broker>,
    reader_thread: UThread<ReaderService>,
    writer_thread: UThread<WriterService<Sp>>,
}

impl<Sp> Broker<Sp>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Pile<Seal = TxoSeal> + Send,
{
    const NAME: &'static str = "broker";

    #[cfg(feature = "embedded")]
    pub fn start_embedded(conf: Config, stockpile: Sp) -> Result<Self, BrokerError> {
        Self::start_inner(conf, stockpile)
    }

    #[cfg(not(feature = "embedded"))]
    pub fn run_standalone(conf: Config, stockpile: Sp) -> Result<(), BrokerError> {
        let me = Self::start_inner(conf, stockpile)?;
        me.run_internal()
    }

    #[cfg(feature = "embedded")]
    pub fn run(self) -> io::Result<JoinHandle<Result<(), BrokerError>>> {
        std::thread::Builder::new()
            .name(s!("broker"))
            .spawn(move || self.run_internal())
    }

    #[cfg(feature = "embedded")]
    pub fn client(&self) -> AsyncClient { AsyncClient::new(self.rpc_thread.sender()) }

    fn start_inner(conf: Config, stockpile: Sp) -> Result<Self, BrokerError> {
        const TIMEOUT: Option<Duration> = Some(Duration::from_secs(60 * 10));

        log::info!(target: Self::NAME, "Starting contracts reader thread...");
        let (reader_tx, reader_rx) = crossbeam_channel::unbounded::<Reader2Broker>();
        let reader = ReaderService::new(reader_tx);
        let reader_thread = UThread::new(reader, TIMEOUT);

        log::info!(target: Self::NAME, "Starting contracts writer thread...");
        let writer =
            WriterService::new(conf.network, &conf.data_dir, stockpile, reader_thread.sender());
        let writer_thread = UThread::new(writer, TIMEOUT);

        log::info!(target: Self::NAME, "Starting the dispatcher thread...");
        let (rpc_tx, rpc_rx) = crossbeam_channel::unbounded::<(ReqId, RgbRpcReq)>();
        #[cfg(feature = "embedded")]
        let (rpc_thread, rpc_tx) = {
            let (rpc_tx2, rpc_rx2) = crossbeam_channel::unbounded::<(ReqId, RgbRpcResp)>();
            let dispatcher = AsyncDispatcher::new(rpc_tx, rpc_rx2);
            let thread = UThread::new(dispatcher, TIMEOUT);
            (thread, rpc_tx2)
        };
        #[cfg(not(feature = "embedded"))]
        let rpc_thread = {
            let controller = Dispatcher::new(conf.network, rpc_tx);
            let listen = conf.rpc.iter().map(|addr| {
                NetAccept::bind(addr)
                    .unwrap_or_else(|err| panic!("unable to bind to {addr}: {err}"))
            });
            let rpc_runtime = service::Runtime::new(conf.rpc[0], controller, listen)
                .map_err(|err| BrokerError::Dispatcher(err.into()))?;
            rpc_runtime
        };

        log::info!(target: Self::NAME, "Launch completed successfully");
        Ok(Self {
            rpc_thread,
            rpc_rx,
            #[cfg(feature = "embedded")]
            rpc_tx,
            reader_rx,
            reader_thread,
            writer_thread,
        })
    }

    fn run_internal(mut self) -> Result<(), BrokerError> {
        loop {
            select! {
                recv(self.rpc_rx) -> msg => {
                    match msg {
                        Ok((req_id, msg)) => {
                            log::trace!(target: Self::NAME, "Received a RPC message {msg} with #{req_id} from a client");
                            self.proc_rpc_msg(req_id, msg).expect("unable to send a message");
                        },
                        Err(err) => {
                            log::error!(target: Self::NAME, "Error receiving RPC message: {err}");
                        }
                    }
                },
                recv(self.reader_rx) -> msg => {
                    match msg {
                        Ok(msg) => {
                            log::trace!(target: Self::NAME, "Received a message {} with #{} from reader", msg.as_reply(), msg.req_id());
                            self.proc_reader_msg(msg).expect("unable to send a message");
                        },
                        Err(err) => {
                            log::error!(target: Self::NAME, "Error receiving reader message: {err}");
                        }
                    }
                }
            }
        }
        // TODO: Provide a control channel to the broker so we can terminate it
        /*
        self.rpc_thread
            .join()
            .map_err(|_| BrokerError::Thread("Dispatcher"))?;
        self.reader_thread
            .join()
            .map_err(|_| BrokerError::Thread("Contracts reader"))?;
        self.writer_thread
            .join()
            .map_err(|_| BrokerError::Thread("Contracts writer"))?;
        Ok(())
         */
    }

    fn proc_rpc_msg(&mut self, req_id: ReqId, msg: RgbRpcReq) -> io::Result<()> {
        log::debug!(target: Self::NAME, "Received an RPC message #{req_id} {msg}");
        match msg {
            RgbRpcReq::Wallets => {
                self.send_reader_req(req_id, Request2Reader::ListWallets(req_id));
            }
            RgbRpcReq::Wallet(wallet_id) => {
                self.send_reader_req(req_id, Request2Reader::ReadWallet(req_id, wallet_id));
            }
            RgbRpcReq::Contracts => {
                self.send_reader_req(req_id, Request2Reader::ListContracts(req_id));
            }
            RgbRpcReq::Contract(contract_id) => {
                self.send_reader_req(req_id, Request2Reader::ReadContract(req_id, contract_id));
            }
            _ => todo!(),
        }
        Ok(())
    }

    fn proc_reader_msg(&mut self, resp: Reader2Broker) -> io::Result<()> {
        log::debug!(target: Self::NAME, "Received reply from reader for an RPC request #{}", resp.req_id());
        let req_id = resp.req_id();
        match resp.into_reply() {
            ReaderMsg::Contracts(contracts) => {
                self.send_rpc_resp(req_id, RgbRpcResp::Contracts(contracts));
            }
            ReaderMsg::ContractState(contract_id, state) => {
                self.send_rpc_resp(req_id, RgbRpcResp::ContractState(contract_id, state));
            }
            ReaderMsg::ContractNotFound(id) => {
                self.send_rpc_resp(req_id, RgbRpcResp::Failure(Failure::not_found(id)));
            }
            ReaderMsg::Wallets(wallets) => {
                self.send_rpc_resp(req_id, RgbRpcResp::Wallets(wallets));
            }
            ReaderMsg::WalletState(wallet_id, state) => {
                self.send_rpc_resp(req_id, RgbRpcResp::WalletState(wallet_id, state));
            }
            ReaderMsg::WalletNotFount(id) => {
                self.send_rpc_resp(req_id, RgbRpcResp::Failure(Failure::not_found(id)));
            }
        }
        Ok(())
    }

    fn send_reader_req(&mut self, req_id: ReqId, req: Request2Reader) {
        log::trace!(target: Self::NAME, "Sending request #{req_id} `{req}` to reader");
        if let Err(err) = self.reader_thread.sender().try_send(req) {
            log::error!(target: Self::NAME, "Unable to send a request to the reader thread: {err}");
            self.send_rpc_resp(
                req_id,
                RgbRpcResp::Failure(Failure::internal_error("broken reader service")),
            );
        }
    }

    fn send_rpc_resp(&mut self, req_id: ReqId, response: RgbRpcResp) {
        log::trace!(target: Self::NAME, "Sending RPC response #{req_id} `{response}` back to dispatcher");
        if let Err(err) = {
            #[cfg(not(feature = "embedded"))]
            {
                self.rpc_thread.cmd((req_id, response))
            }
            #[cfg(feature = "embedded")]
            {
                self.rpc_tx.send((req_id, response))
            }
        } {
            log::error!(target: Self::NAME, "Channel to the dispatcher thread is broken: {err}");
            panic!("The channel to the dispatcher thread is broken. Unable to proceed, exiting.");
        };
    }
}

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum BrokerError {
    /// unable to initialize dispatcher.
    ///
    /// {0}
    Dispatcher(IoError),

    /// unable to initialize importing service.
    ///
    /// {0}
    Import(IoError),

    /// unable to initialize block importing service.
    ///
    /// {0}
    Importer(IoError),

    /// unable to create a thread for {0}.
    Thread(&'static str),
}
