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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::Infallible;
use std::ops::ControlFlow;

use bpstd::DescrId;
use bpstd::psbt::Utxo;
use crossbeam_channel::Sender;
use microservices::UService;
use rgb::popls::bp::seals::TxoSeal;
use rgb::{
    CellAddr, ContractId, ContractState, ContractStateName, ImmutableState, Opid, WitnessStatus,
};
use rgbp::descriptor::RgbDescr;
use rgbrpc::WalletInfo;
use strict_types::StrictVal;

use crate::ReqId;

pub enum Request2Reader {
    ReadContractState(ReqId, ContractId),
    ReadWallet(ReqId, DescrId),
    UpdateState(ContractId, ContractState<TxoSeal>),
}

pub struct Reader2Broker(ReqId, ReaderMsg);

impl Reader2Broker {
    pub fn req_id(&self) -> ReqId { self.0 }
    pub fn into_reply(self) -> ReaderMsg { self.1 }
}

pub enum ReaderMsg {
    ContractState(ContractId, ContractState<TxoSeal>),
    ContractNotFound(ContractId),

    WalletInfo(DescrId, WalletInfo),
    WalletNotFount(DescrId),
}

pub struct ContractsReader {
    state: HashMap<ContractId, ContractState<TxoSeal>>,
    wallets: HashMap<DescrId, RoWallet>,
    broker: Sender<Reader2Broker>,
}

impl ContractsReader {
    pub fn new(broker: Sender<Reader2Broker>) -> Self {
        Self { state: none!(), wallets: none!(), broker }
    }
}

#[derive(Clone)]
pub struct RoWallet {
    pub utxos: HashSet<Utxo>,
    pub descriptor: RgbDescr,
}

// TODO: Make it reactor to process non-blocking replies to the Broker
impl UService for ContractsReader {
    type Msg = Request2Reader;
    type Error = Infallible;
    const NAME: &'static str = "contracts-reader";

    fn process(&mut self, msg: Self::Msg) -> Result<ControlFlow<u8>, Self::Error> {
        match msg {
            Request2Reader::ReadContractState(req_id, id) => {
                log::trace!(target: Self::NAME, "Sending state for contract {id}");
                let state = self
                    .state
                    .get(&id)
                    .cloned()
                    .map(|state| ReaderMsg::ContractState(id, state))
                    .unwrap_or_else(|| {
                        log::trace!(target: Self::NAME, "State for contract {id} is not known");
                        ReaderMsg::ContractNotFound(id)
                    });
                if let Err(err) = self.broker.try_send(Reader2Broker(req_id, state)) {
                    log::error!(target: Self::NAME, "Failed to send reply {req_id}: {err}");
                }
            }
            Request2Reader::ReadWallet(req_id, id) => {}
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
