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

use std::collections::BTreeMap;
use std::io::{Read, Write};

use amplify::confinement::{MediumOrdSet, SmallBlob, SmallOrdSet, TinyBlob};
use bpstd::DescrId;
use bpstd::psbt::Utxo;
use bpstd::seals::TxoSeal;
use netservices::Frame;
use rgb::{ContractState, ContractStateName, WitnessStatus};
use rgbp::descriptors::RgbDescr;
use sonicapi::{CellAddr, CodexId, ContractId, Opid, StateAtom};
use strict_types::StrictVal;

use crate::{CiboriumError, Failure, Status};

#[derive(Clone, Debug, Display)]
#[derive(Serialize, Deserialize)]
pub enum RgbRpcResp {
    #[display("FAILURE({0})")]
    Failure(Failure),

    #[display("PONG")]
    Pong(TinyBlob),

    #[display("STATUS")]
    Status(Status),

    #[display("ISSUERS(...)")]
    Issuers(SmallOrdSet<CodexId>),

    #[display("CONTRACTS(...)")]
    Contracts(MediumOrdSet<ContractId>),

    /*
    #[display("ISSUER(...)")]
    Issuer(Schema),

    #[display("ARTICLES(...)")]
    Articles(Articles),
     */
    #[display("CONTRACT_STATE({0}, ...)")]
    ContractState(ContractId, ContractState<TxoSeal>),

    #[display("WALLET_STATE({0}, ...)")]
    WalletState(DescrId, WalletInfo),

    #[display("CONSIGN_INIT({0})")]
    ConsignInit(u64),

    #[display("CONSIGN_DATA({0}, ...)")]
    ConsignData(u64, SmallBlob),

    #[display("IMPORTED({0})")]
    Imported(CodexId),

    #[display("ACCEPT_START({0})")]
    AcceptStart(u64),

    #[display("ACCEPT_COMPLETE({0})")]
    AcceptComplete(u64),
}

impl Frame for RgbRpcResp {
    type Error = CiboriumError;

    fn unmarshall(reader: impl Read) -> Result<Option<Self>, Self::Error> {
        ciborium::from_reader(reader).map_err(CiboriumError::from)
    }

    fn marshall(&self, writer: impl Write) -> Result<(), Self::Error> {
        ciborium::into_writer(self, writer)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct WalletInfo {
    pub descriptor: RgbDescr,
    pub immutable: BTreeMap<ContractStateName, BTreeMap<CellAddr, StateAtom>>,
    pub owned: BTreeMap<Utxo, BTreeMap<ContractStateName, BTreeMap<CellAddr, StrictVal>>>,
    pub aggregated: BTreeMap<ContractStateName, StrictVal>,
    pub confirmations: BTreeMap<Opid, WitnessStatus>,
}
