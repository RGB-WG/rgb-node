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

use std::collections::VecDeque;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;

use bpwallet::Network;
use crossbeam_channel::Sender;
use netservices::service::{ServiceCommand, ServiceController};
use netservices::{Frame, ImpossibleResource};
use reactor::Timestamp;
use rgbrpc::{RemoteAddr, RgbRpcResp, Session};

#[derive(Clone, Debug)]
pub enum Watcher2Broker {}

pub struct Watcher {
    network: Network,
    broker: Sender<Watcher2Broker>,
    actions: VecDeque<ServiceCommand<SocketAddr, RgbRpcResp>>,
}

impl Watcher {
    pub fn new(network: Network, broker: Sender<Watcher2Broker>) -> Self {
        Self { network, broker, actions: VecDeque::new() }
    }
}

impl ServiceController<RemoteAddr, Session, ImpossibleResource, Watcher2Broker> for Watcher {
    type InFrame = bpclient::rpc::Response;
    type OutFrame = bpclient::rpc::Request;

    fn should_accept(&mut self, remote: &RemoteAddr, time: Timestamp) -> bool { unreachable!() }

    fn establish_session(
        &mut self,
        remote: RemoteAddr,
        connection: Session,
        time: Timestamp,
    ) -> Result<Session, impl Error> {
        Result::<_, Infallible>::Ok(connection)
    }

    fn on_command(&mut self, cmd: Watcher2Broker) { todo!() }

    fn on_frame(&mut self, remote: SocketAddr, req: Self::InFrame) { todo!() }

    fn on_frame_unparsable(&mut self, remote: SocketAddr, err: &<Self::InFrame as Frame>::Error) {
        todo!()
    }
}
