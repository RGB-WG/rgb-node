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

use bpstd::psbt::Utxo;
use bpstd::{ConsensusEncode, DescrId, Outpoint, Sats, Terminal};
use native_db::{Key, ToKey};
use native_model::Model;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(Serialize, Deserialize)]
pub struct UtxoId {
    pub descr: DescrId,
    pub outpoint: Outpoint,
}

impl ToKey for UtxoId {
    fn to_key(&self) -> Key { (self.descr.0, self.outpoint.to_string()).to_key() }

    fn key_names() -> Vec<String> { vec!["utxo_id".to_owned()] }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(Serialize, Deserialize)]
pub struct OutpointModel(pub Outpoint);

impl ToKey for OutpointModel {
    fn to_key(&self) -> Key { self.0.consensus_serialize().to_key() }

    fn key_names() -> Vec<String> { vec!["outpoint".to_owned()] }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[derive(Serialize, Deserialize)]
#[native_model(id = 2, version = 1)]
#[native_db(primary_key(utxo_id -> UtxoId))]
pub struct UtxoModel {
    pub descr: DescrId,
    #[secondary_key]
    pub outpoint: OutpointModel,
    pub terminal: Terminal,
    pub value: Sats,
}

impl UtxoModel {
    pub fn with(id: DescrId, utxo: Utxo) -> Self {
        Self {
            descr: id,
            outpoint: OutpointModel(utxo.outpoint),
            terminal: utxo.terminal,
            value: utxo.value,
        }
    }

    pub fn utxo_id(&self) -> UtxoId { UtxoId { descr: self.descr, outpoint: self.outpoint.0 } }
}

impl From<UtxoModel> for Utxo {
    fn from(utxo: UtxoModel) -> Self {
        Utxo {
            outpoint: utxo.outpoint.0,
            value: utxo.value,
            terminal: utxo.terminal,
        }
    }
}
