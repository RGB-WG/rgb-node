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
use rgbp::ContractInfo;
use rgbp::descriptors::RgbDescr;
use rgbrpc::{WalletInfo, WalletState};
use strict_types::StrictVal;

use crate::ReqId;

#[derive(Debug, Display)]
pub enum Request2Reader {
    // These are requests from the broker
    #[display("LIST_CONTRACTS({0})")]
    ListContracts(ReqId),
    #[display("READ_CONTRACTS({0}, {1})")]
    ReadContract(ReqId, ContractId),
    #[display("LIST_WALLETS({0})")]
    ListWallets(ReqId),
    #[display("READ_WALLET({0}, {1})")]
    ReadWallet(ReqId, DescrId),

    // These are requests from the writer
    #[display("UPSERT_WALLET({0}, ...)")]
    UpsertWallet(DescrId, RoWallet),
    #[display("REMOVE_WALLET({0})")]
    RemoveWallet(DescrId),
    #[display("UPSERT_CONTRACT({0}, ...)")]
    UpsertContract(ContractId, RoContract),
    #[display("REMOVE_CONTRACT({0})")]
    RemoveContract(ContractId),
}

#[derive(Debug)]
pub struct Reader2Broker(ReqId, ReaderMsg);

impl Reader2Broker {
    pub fn req_id(&self) -> ReqId { self.0 }
    pub fn as_reply(&self) -> &ReaderMsg { &self.1 }
    pub fn into_reply(self) -> ReaderMsg { self.1 }
}

#[derive(Debug, Display)]
pub enum ReaderMsg {
    #[display("CONTRACTS(...)")]
    Contracts(Vec<ContractInfo>),
    #[display("CONTRACT_STATE({0}, ...)")]
    ContractState(ContractId, ContractState<TxoSeal>),
    #[display("CONTRACT_NOT_FOUND({0})")]
    ContractNotFound(ContractId),

    #[display("WALLETS(...)")]
    Wallets(Vec<WalletInfo>),
    #[display("WALLET_STATE({0}, ...)")]
    WalletState(DescrId, WalletState),
    #[display("WALLET_NOT_FOUND({0})")]
    WalletNotFount(DescrId),
}

#[derive(Clone, Debug)]
pub struct RoWallet {
    pub id: DescrId,
    pub name: String,
    pub descriptor: RgbDescr,
    pub utxos: HashSet<Utxo>,
}

impl RoWallet {
    pub fn info(&self) -> WalletInfo {
        WalletInfo {
            id: self.id,
            name: self.name.clone(),
            descriptor: self.descriptor.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoContract {
    pub id: ContractId,
    pub info: ContractInfo,
    pub state: ContractState<TxoSeal>,
}

#[derive(Debug)]
pub struct ReaderService {
    contracts: HashMap<ContractId, RoContract>,
    wallets: HashMap<DescrId, RoWallet>,
    broker: Sender<Reader2Broker>,
}

impl ReaderService {
    pub fn new(broker: Sender<Reader2Broker>) -> Self {
        Self { contracts: none!(), wallets: none!(), broker }
    }

    pub fn wallet_info(&self, wallet_id: DescrId) -> Option<WalletState> {
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
        for (contract_id, contract) in &self.contracts {
            let state = &contract.state;
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
        Some(WalletState {
            info: wallet.info(),
            immutable,
            owned,
            aggregated,
            confirmations,
        })
    }

    pub fn send_to_broker(&self, reply: Reader2Broker) {
        log::trace!(target: Self::NAME, "Sending reply `{}` to broker for request #{}", reply.1, reply.req_id());
        let req_id = reply.req_id();
        if let Err(err) = self.broker.try_send(reply) {
            log::error!(target: Self::NAME, "Failed to send reply to broker for request #{req_id}: {err}");
        }
    }
}

impl UService for ReaderService {
    type Msg = Request2Reader;
    type Error = Infallible;
    const NAME: &'static str = "contracts-reader";

    fn process(&mut self, msg: Self::Msg) -> Result<ControlFlow<u8>, Self::Error> {
        match msg {
            Request2Reader::ListContracts(req_id) => {
                log::debug!(target: Self::NAME, "Listing all contracts");
                let contracts = self
                    .contracts
                    .values()
                    .map(|contract| contract.info.clone())
                    .collect();
                self.send_to_broker(Reader2Broker(req_id, ReaderMsg::Contracts(contracts)));
            }
            Request2Reader::ReadContract(req_id, id) => {
                log::debug!(target: Self::NAME, "Sending state for contract {id}");
                let reply = self
                    .contracts
                    .get(&id)
                    .map(|contract| ReaderMsg::ContractState(id, contract.state.clone()))
                    .unwrap_or_else(|| {
                        log::trace!(target: Self::NAME, "State for contract {id} is not known");
                        ReaderMsg::ContractNotFound(id)
                    });
                self.send_to_broker(Reader2Broker(req_id, reply));
            }
            Request2Reader::UpsertContract(id, contract) => {
                log::debug!(target: Self::NAME, "Received update for contract {id}");
                self.contracts.insert(id, contract);
            }
            Request2Reader::RemoveContract(id) => {
                log::debug!(target: Self::NAME, "Received request to remove contract {id}");
                if self.contracts.remove(&id).is_none() {
                    log::warn!(target: Self::NAME, "Contract {id} is not known");
                }
            }

            Request2Reader::ListWallets(req_id) => {
                log::debug!(target: Self::NAME, "Listing all wallets");
                let wallets = self.wallets.values().map(RoWallet::info).collect();
                self.send_to_broker(Reader2Broker(req_id, ReaderMsg::Wallets(wallets)));
            }
            Request2Reader::ReadWallet(req_id, id) => {
                log::debug!(target: Self::NAME, "Sending state for wallet {id}");
                let state = self
                    .wallet_info(id)
                    .map(|info| ReaderMsg::WalletState(id, info))
                    .unwrap_or_else(|| {
                        log::trace!(target: Self::NAME, "State for wallet {id} is not known");
                        ReaderMsg::WalletNotFount(id)
                    });
                self.send_to_broker(Reader2Broker(req_id, state));
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
