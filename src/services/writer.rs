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

use std::collections::HashSet;
use std::convert::Infallible;
use std::ops::ControlFlow;

use bpstd::psbt::PsbtConstructor;
use bpstd::seals::TxoSeal;
use bpstd::{Network, XpubDerivable};
use microservices::{USender, UService};
use rgb::{Contracts, Pile, Stockpile};
use rgbp::resolvers::MultiResolver;
use rgbp::{Owner, OwnerProvider, RgbRuntime};

use super::Request2Reader;
use crate::services::reader::RoWallet;
use crate::{DbHolder, DbUtxos};

pub enum Request2Writer {
    Consign(),
    Accept(),
}

pub struct WriterService<Sp>
where
    Sp: Stockpile,
    Sp::Pile: Pile<Seal = TxoSeal>,
{
    runtime: RgbRuntime<Owner<MultiResolver, DbHolder, XpubDerivable, DbUtxos>, Sp>,
    reader: USender<Request2Reader>,
}

impl<Sp: Stockpile> WriterService<Sp>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Pile<Seal = TxoSeal> + Send,
{
    pub fn new(network: Network, stockpile: Sp, reader: USender<Request2Reader>) -> Self {
        log::info!(target: Self::NAME, "Loading contracts from persistence");
        let contracts = Contracts::load(stockpile);

        // TODO: Use real resolver
        let resolver = MultiResolver::new_absent().unwrap_or_else(|err| {
            log::error!(target: Self::NAME, "Unable to connect to the resolver. {err}");
            panic!("Unable to connect to the resolver due to {err}");
        });

        log::info!(target: Self::NAME, "Loading wallets from database");
        let holder = DbHolder::load().unwrap_or_else(|err| {
            log::error!(target: Self::NAME, "Unable to load database. {err}");
            panic!("Unable to load database due to {err}");
        });
        let owner = Owner::with_components(network, holder, resolver);

        let runtime = RgbRuntime::with_components(owner, contracts);
        let mut me = Self { runtime, reader };

        log::info!(target: Self::NAME, "Contracts loaded successfully, sending state to the reader");
        for id in me.runtime.contracts.contract_ids() {
            let state = me.runtime.contracts.contract_state(id);
            log::debug!(target: Self::NAME, "Sending contract state for {id}");
            me.reader
                .send(Request2Reader::UpsertContract(id, state))
                .unwrap_or_else(|err| panic!("Failed to send state for contract {id}: {err}"));
        }

        log::info!(target: Self::NAME, "Wallets loaded successfully, sending state to the reader");
        let all_wallets = me.runtime.wallet.wallet_ids().collect::<HashSet<_>>();
        for id in all_wallets {
            me.runtime.wallet.switch(id);
            let descriptor = me.runtime.wallet.descriptor().clone();
            let utxo = me.runtime.wallet.utxos();
            let wallet = RoWallet { descriptor, utxos: utxo.utxos() };
            log::debug!(target: Self::NAME, "Sending wallet state for {id}");
            me.reader
                .send(Request2Reader::UpsertWallet(id, wallet))
                .unwrap_or_else(|err| panic!("Failed to send state for wallet {id}: {err}"));
        }

        me
    }
}

impl<Sp: Stockpile> UService for WriterService<Sp>
where
    Sp: Stockpile + Send + 'static,
    Sp::Stock: Send,
    Sp::Pile: Pile<Seal = TxoSeal> + Send,
{
    type Msg = Request2Writer;
    type Error = Infallible;
    const NAME: &'static str = "contracts-writer";

    fn process(&mut self, msg: Self::Msg) -> Result<ControlFlow<u8>, Self::Error> {
        match msg {
            Request2Writer::Consign() => {}
            Request2Writer::Accept() => {}
        }
        Ok(ControlFlow::Continue(()))
    }

    fn terminate(&mut self) {
        log::info!(target: Self::NAME, "Shutting down contracts writer service");
    }
}
