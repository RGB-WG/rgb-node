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

use std::collections::{HashMap, VecDeque};
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
use rgbrpc::{ClientInfo, RemoteAddr, Request, Response, Session, Status};
use strict_encoding::DecodeError;

use crate::BrokerRpcMsg;

// TODO: Make this configuration parameter
const MAX_CLIENTS: usize = 0xFFFF;
const NAME: &str = "rpc";

#[derive(Clone, Debug)]
pub enum RpcCmd {
    Send(SocketAddr, Response),
}

pub struct RpcController {
    network: Network,
    broker: Sender<BrokerRpcMsg>,
    actions: VecDeque<ServiceCommand<SocketAddr, Response>>,
    clients: HashMap<SocketAddr, ClientInfo>,
}

impl RpcController {
    pub fn new(network: Network, broker: Sender<BrokerRpcMsg>) -> Self {
        Self { network, broker, actions: none!(), clients: none!() }
    }
}

impl ServiceController<RemoteAddr, Session, TcpListener, RpcCmd> for RpcController {
    type InFrame = Request;
    type OutFrame = Response;

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
        log::warn!(target: NAME, "Client at {remote} got disconnected due to {reason} ({})", client.agent.map(|a| a.to_string()).unwrap_or(none!()));
    }

    fn on_command(&mut self, cmd: RpcCmd) {
        match cmd {
            RpcCmd::Send(remote, response) => self
                .actions
                .push_back(ServiceCommand::Send(remote, response)),
        }
    }

    fn on_frame(&mut self, remote: SocketAddr, req: Request) {
        log::debug!(target: NAME, "Processing `{req}`");

        let client = self.clients.get_mut(&remote).expect("must be known");
        client.last_seen = Timestamp::now().as_millis();

        let response = match req {
            // TODO: Check that networks match
            Request::Ping(noise) => Some(Response::Pong(noise)),
            Request::Status => Some(Response::Status(Status {
                clients: SmallVec::from_iter_checked(self.clients.values().cloned()),
            })),
        };
        if let Some(response) = response {
            log::debug!(target: NAME, "Sending `{response}` to {remote}");
            self.actions
                .push_back(ServiceCommand::Send(remote, response));
        }
    }

    fn on_frame_unparsable(&mut self, remote: SocketAddr, err: &DecodeError) {
        log::error!(target: NAME, "Disconnecting client {remote} due to unparsable frame: {err}");
        self.actions.push_back(ServiceCommand::Disconnect(remote))
    }
}

impl Iterator for RpcController {
    type Item = ServiceCommand<SocketAddr, Response>;

    fn next(&mut self) -> Option<Self::Item> { self.actions.pop_front() }
}
