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

use std::collections::HashMap;
use std::convert::Infallible;
use std::ops::ControlFlow;

use crossbeam_channel::Sender;
use microservices::UService;
use rgb::{ContractId, ContractState, RgbSeal};

use crate::ReqId;

pub enum Request2Reader<Seal> {
    ReadState(ReqId, ContractId),
    UpdateState(ContractId, ContractState<Seal>),
}

pub struct Reader2Broker<Seal: RgbSeal>(ReqId, ReaderMsg<Seal>);

impl<Seal: RgbSeal> Reader2Broker<Seal> {
    pub fn req_id(&self) -> ReqId { self.0 }
    pub fn into_reply(self) -> ReaderMsg<Seal> { self.1 }
}

pub enum ReaderMsg<Seal: RgbSeal> {
    State(ContractId, ContractState<Seal>),
    NotFound(ContractId),
}

#[derive(Debug)]
pub struct ContractsReader<Seal: RgbSeal> {
    state: HashMap<ContractId, ContractState<Seal>>,
    broker: Sender<Reader2Broker<Seal>>,
}

impl<Seal: RgbSeal> ContractsReader<Seal> {
    pub fn new(broker: Sender<Reader2Broker<Seal>>) -> Self { Self { state: none!(), broker } }
}

// TODO: Make it reactor to process non-blocking replies to the Broker
impl<Seal> UService for ContractsReader<Seal>
where Seal: RgbSeal + Send + 'static
{
    type Msg = Request2Reader<Seal>;
    type Error = Infallible;
    const NAME: &'static str = "contracts-reader";

    fn process(&mut self, msg: Self::Msg) -> Result<ControlFlow<u8>, Self::Error> {
        match msg {
            Request2Reader::ReadState(req_id, id) => {
                log::trace!(target: Self::NAME, "Sending state for contract {id}");
                let state = self
                    .state
                    .get(&id)
                    .cloned()
                    .map(|state| ReaderMsg::State(id, state))
                    .unwrap_or_else(|| {
                        log::trace!(target: Self::NAME, "State for contract {id} is not known");
                        ReaderMsg::NotFound(id)
                    });
                if let Err(err) = self.broker.try_send(Reader2Broker(req_id, state)) {
                    log::error!(target: Self::NAME, "Failed to send reply {req_id}: {err}");
                }
            }
            Request2Reader::UpdateState(id, state) => {
                log::debug!(target: Self::NAME, "Received state update for contract {id}");
                self.state.insert(id, state);
            }
        }
        Ok(ControlFlow::Continue(()))
    }

    fn terminate(&mut self) {
        log::info!(target: Self::NAME, "Shutting down contracts reader service");
    }
}
