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

//! RPC connections from clients organized into a reactor thread.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::convert::Infallible;
use std::error::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};

use amplify::confinement::SmallVec;
use bpwallet::Network;
use crossbeam_channel::Sender;
use netservices::Direction;
use netservices::remotes::DisconnectReason;
use netservices::service::{ServiceCommand, ServiceController};
use reactor::Timestamp;
use rgbrpc::{ClientInfo, Failure, RemoteAddr, RgbRpcReq, RgbRpcResp, Session, Status};
use strict_encoding::DecodeError;

use crate::{Broker2Dispatch, ReqId};

// TODO: Make this configuration parameter
const MAX_CLIENTS: usize = 0xFFFF;
const NAME: &str = "dispatcher";

#[derive(Clone, Debug)]
pub enum Dispatch2Broker {
    Send(ReqId, RgbRpcResp),
}

pub struct Displatcher {
    network: Network,
    broker: Sender<(ReqId, Broker2Dispatch)>,
    actions: VecDeque<ServiceCommand<SocketAddr, RgbRpcResp>>,
    clients: HashMap<SocketAddr, ClientInfo>,
    requests: BTreeMap<ReqId, SocketAddr>,
}

impl Displatcher {
    pub fn new(network: Network, broker: Sender<(ReqId, Broker2Dispatch)>) -> Self {
        Self {
            network,
            broker,
            actions: none!(),
            clients: none!(),
            requests: none!(),
        }
    }
}

impl ServiceController<RemoteAddr, Session, TcpListener, Dispatch2Broker> for Displatcher {
    type InFrame = RgbRpcReq;
    type OutFrame = RgbRpcResp;

    fn should_accept(&mut self, _remote: &RemoteAddr, _time: Timestamp) -> bool {
        // For now, we just do not allow more than 64k connections.
        // In a future, we may also filter out known clients doing spam and DDoS attacks
        self.clients.len() < MAX_CLIENTS
    }

    fn establish_session(
        &mut self,
        _remote: RemoteAddr,
        connection: TcpStream,
        _time: Timestamp,
    ) -> Result<Session, impl Error> {
        Result::<_, Infallible>::Ok(connection)
    }

    fn on_listening(&mut self, local: SocketAddr) {
        log::info!(target: NAME, "Listening on {local}");
    }

    fn on_established(
        &mut self,
        addr: SocketAddr,
        _remote: RemoteAddr,
        direction: Direction,
        time: Timestamp,
    ) {
        debug_assert_eq!(direction, Direction::Inbound);
        if self
            .clients
            .insert(addr, ClientInfo {
                agent: None,
                connected: time.as_millis(),
                last_seen: time.as_millis(),
            })
            .is_some()
        {
            panic!("Client {addr} already connected!");
        };
    }

    fn on_disconnected(&mut self, remote: SocketAddr, _: Direction, reason: &DisconnectReason) {
        let client = self.clients.remove(&remote).unwrap_or_else(|| {
            panic!("Client at {remote} got disconnected but not found in the provider list");
        });
        log::warn!(target: NAME, "Client at {remote} got disconnected due to {reason} ({})", client.agent.map(|a| a.to_string()).unwrap_or_default());
    }

    fn on_command(&mut self, cmd: Dispatch2Broker) {
        match cmd {
            Dispatch2Broker::Send(req_id, response) => {
                let remote = self.requests.remove(&req_id).unwrap_or_else(|| {
                    panic!("Unmatched reply to non-existing request {req_id}");
                });
                self.send_response(remote, response);
            }
        }
    }

    fn on_frame(&mut self, remote: SocketAddr, req: RgbRpcReq) {
        log::trace!(target: NAME, "Processing `{req}`");

        let client = self.clients.get_mut(&remote).expect("must be known");
        client.last_seen = Timestamp::now().as_millis();

        match req {
            // TODO: Check that networks match
            RgbRpcReq::Ping(noise) => self.send_response(remote, RgbRpcResp::Pong(noise)),
            RgbRpcReq::Status => self.send_response(
                remote,
                RgbRpcResp::Status(Status {
                    clients: SmallVec::from_iter_checked(self.clients.values().cloned()),
                }),
            ),
            RgbRpcReq::State(contract_id) => {
                self.request_broker(remote, Broker2Dispatch::ContractState(contract_id));
            }
            _ => todo!(),
        }
    }

    fn on_frame_unparsable(&mut self, remote: SocketAddr, err: &DecodeError) {
        log::error!(target: NAME, "Disconnecting client {remote} due to unparsable frame: {err}");
        self.actions.push_back(ServiceCommand::Disconnect(remote))
    }
}

impl Iterator for Displatcher {
    type Item = ServiceCommand<SocketAddr, RgbRpcResp>;

    fn next(&mut self) -> Option<Self::Item> { self.actions.pop_front() }
}

impl Displatcher {
    pub fn send_response(&mut self, remote: SocketAddr, response: RgbRpcResp) {
        log::trace!(target: NAME, "Sending `{response}` to {remote}");
        self.actions
            .push_back(ServiceCommand::Send(remote, response));
    }

    pub fn request_broker(&mut self, remote: SocketAddr, request: Broker2Dispatch) {
        let req_id = self
            .requests
            .last_key_value()
            .map(|(id, _)| *id)
            .unwrap_or_default()
            + 1;
        self.requests.insert(req_id, remote);
        if let Err(err) = self.broker.send((req_id, request)) {
            log::error!(target: NAME, "Broker thread channel is dead: {err}");
            self.send_response(
                remote,
                RgbRpcResp::Failure(Failure::internal_error("Broker thread channel is dead")),
            );
        }
    }
}
