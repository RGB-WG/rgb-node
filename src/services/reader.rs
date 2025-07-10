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
use rgb::{CellAddr, ContractId, ContractState, ContractStateName};
use rgbp::descriptor::RgbDescr;
use rgbrpc::WalletInfo;
use strict_types::StrictVal;

use crate::ReqId;

pub enum Request2Reader {
    // These are requests from the broker
    ReadContract(ReqId, ContractId),
    ReadWallet(ReqId, DescrId),

    // These are requests from the writer
    UpsertWallet(DescrId, RoWallet),
    RemoveWallet(DescrId),
    UpsertContract(ContractId, ContractState<TxoSeal>),
    RemoveContract(ContractId),
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

#[derive(Clone)]
pub struct RoWallet {
    pub utxos: HashSet<Utxo>,
    pub descriptor: RgbDescr,
}

pub struct ReaderService {
    state: HashMap<ContractId, ContractState<TxoSeal>>,
    wallets: HashMap<DescrId, RoWallet>,
    broker: Sender<Reader2Broker>,
}

impl ReaderService {
    pub fn new(broker: Sender<Reader2Broker>) -> Self {
        Self { state: none!(), wallets: none!(), broker }
    }

    pub fn wallet_info(&self, wallet_id: DescrId) -> Option<WalletInfo> {
        let wallet = self.wallets.get(&wallet_id)?;

        let mut immutable = bmap! {};
        let mut owned: BTreeMap<Utxo, BTreeMap<ContractStateName, BTreeMap<CellAddr, StrictVal>>> =
            wallet
                .utxos
                .iter()
                .map(|utxo| (*utxo, default!()))
                .collect();
        let mut aggregated = bmap! {};
        let mut confirmations = bmap! {};
        for (contract_id, state) in &self.state {
            for (state_name, state) in &state.immutable {
                let contract_state_name = ContractStateName::new(*contract_id, state_name.clone());
                let mut immutable_state = bmap! {};
                for state in state {
                    confirmations.insert(state.addr.opid, state.status);
                    immutable_state.insert(state.addr, state.data.clone());
                }
                immutable.insert(contract_state_name, immutable_state);
            }
            for (state_name, state) in &state.owned {
                let contract_state_name = ContractStateName::new(*contract_id, state_name.clone());
                for state in state {
                    let Some(owned_state) = owned
                        .iter_mut()
                        .find(|(utxo, _)| utxo.outpoint == state.assignment.seal.primary)
                        .map(|(_, data)| data)
                    else {
                        continue;
                    };
                    confirmations.insert(state.addr.opid, state.status);
                    owned_state
                        .entry(contract_state_name.clone())
                        .or_default()
                        .insert(state.addr, state.assignment.data.clone());
                }
            }
            for (state_name, state) in &state.aggregated {
                let contract_state_name = ContractStateName::new(*contract_id, state_name.clone());
                aggregated.insert(contract_state_name, state.clone());
            }
        }
        Some(WalletInfo {
            descriptor: wallet.descriptor.clone(),
            immutable,
            owned,
            aggregated,
            confirmations,
        })
    }

    pub fn send_to_broker(&self, req_id: ReqId, reply: Reader2Broker) {
        if let Err(err) = self.broker.try_send(reply) {
            log::error!(target: Self::NAME, "Failed to send reply {req_id}: {err}");
        }
    }
}

// TODO: Make it reactor to process non-blocking replies to the Broker
impl UService for ReaderService {
    type Msg = Request2Reader;
    type Error = Infallible;
    const NAME: &'static str = "contracts-reader";

    fn process(&mut self, msg: Self::Msg) -> Result<ControlFlow<u8>, Self::Error> {
        match msg {
            Request2Reader::ReadContract(req_id, id) => {
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
                self.send_to_broker(req_id, Reader2Broker(req_id, state));
            }
            Request2Reader::ReadWallet(req_id, id) => {
                log::trace!(target: Self::NAME, "Sending state for wallet {id}");
                let state = self
                    .wallet_info(id)
                    .map(|info| ReaderMsg::WalletInfo(id, info))
                    .unwrap_or_else(|| {
                        log::trace!(target: Self::NAME, "State for wallet {id} is not known");
                        ReaderMsg::WalletNotFount(id)
                    });
                self.send_to_broker(req_id, Reader2Broker(req_id, state));
            }
            Request2Reader::UpsertContract(id, state) => {
                log::debug!(target: Self::NAME, "Received update for contract {id}");
                self.state.insert(id, state);
            }
            Request2Reader::RemoveContract(id) => {
                log::debug!(target: Self::NAME, "Received request to remove contract {id}");
                if self.state.remove(&id).is_none() {
                    log::warn!(target: Self::NAME, "Contract {id} is not known");
                }
            }
            Request2Reader::UpsertWallet(id, wallet) => {
                log::debug!(target: Self::NAME, "Received update for wallet {id}");
                self.wallets.insert(id, wallet);
            }
            Request2Reader::RemoveWallet(id) => {
                log::debug!(target: Self::NAME, "Received request to remove wallet {id}");
                if self.wallets.remove(&id).is_none() {
                    log::warn!(target: Self::NAME, "Wallet {id} is not known");
                }
            }
        }
        Ok(ControlFlow::Continue(()))
    }

    fn terminate(&mut self) {
        log::info!(target: Self::NAME, "Shutting down contracts reader service");
    }
}
