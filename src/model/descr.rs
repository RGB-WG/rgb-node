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

use bpstd::{DescrId, Keychain, NormalIndex};
use native_db::ToKey;
use native_model::Model;
use rgbp::descriptors::RgbDescr;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct DescrModel {
    #[primary_key]
    pub id: u64,
    pub descriptor: RgbDescr,
    pub next_index: BTreeMap<Keychain, NormalIndex>,
}

impl DescrModel {
    pub fn descr_id(&self) -> DescrId { DescrId(self.id) }

    pub fn next_index(&self, keychain: impl Into<Keychain>) -> NormalIndex {
        self.next_index
            .get(&keychain.into())
            .copied()
            .unwrap_or_default()
    }
}
