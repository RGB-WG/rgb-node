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

use std::convert::Infallible;
use std::ops::ControlFlow;

use microservices::{USender, UService};
use rgb::{Contracts, Pile, Stockpile};

use super::ReaderReq;

pub enum WriterReq {
    Consign(),
    Accept(),
}

pub struct ContractsWriter<Sp: Stockpile> {
    contracts: Contracts<Sp>,
    reader: USender<ReaderReq<<Sp::Pile as Pile>::Seal>>,
}

impl<Sp: Stockpile> ContractsWriter<Sp>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Send,
    <Sp::Pile as Pile>::Seal: Send,
{
    pub fn new(stockpile: Sp, reader: USender<ReaderReq<<Sp::Pile as Pile>::Seal>>) -> Self {
        log::info!(target: Self::NAME, "Loading contracts from persistence");
        let me = Self { contracts: Contracts::load(stockpile), reader };

        log::info!(target: Self::NAME, "Contracts loaded successfully, sending state to the reader");
        for id in me.contracts.contract_ids() {
            let state = me.contracts.contract_state(id);
            log::debug!(target: Self::NAME, "Sending contract state for {id}");
            me.reader
                .send(ReaderReq::UpdateState(id, state))
                .unwrap_or_else(|err| panic!("Failed to send state for contract {id}: {err}"));
        }
        me
    }
}

impl<Sp: Stockpile> UService for ContractsWriter<Sp>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Send,
    <Sp::Pile as Pile>::Seal: Send,
{
    type Msg = WriterReq;
    type Error = Infallible;
    const NAME: &'static str = "contracts-writer";

    fn process(&mut self, msg: Self::Msg) -> Result<ControlFlow<u8>, Self::Error> {
        match msg {
            WriterReq::Consign() => {}
            WriterReq::Accept() => {}
        }
        Ok(ControlFlow::Continue(()))
    }

    fn terminate(&mut self) {
        log::info!(target: Self::NAME, "Shutting down contracts writer service");
    }
}
