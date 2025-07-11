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
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;

use bpstd::psbt::Utxo;
use bpstd::{Keychain, NormalIndex, Outpoint, Sats, Terminal, XpubDerivable};
use native_db::{Builder, Database, Models, db_type};
use rgbp::descriptors::RgbDescr;
use rgbp::{MultiHolder, OwnerProvider, UtxoSet};

use crate::model::UtxoModel;

static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<UtxoModel>().unwrap();
    models
});

fn db() -> Result<Database<'static>, db_type::Error> { Builder::new().create_in_memory(&MODELS) }

pub struct DbHolder {
    inner: MultiHolder<XpubDerivable, DbUtxos>,
    db: Database<'static>,
}

impl Deref for DbHolder {
    type Target = MultiHolder<XpubDerivable, DbUtxos>;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for DbHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl DbHolder {
    pub fn load() -> Result<Self, db_type::Error> {
        let db = db()?;
        let inner = MultiHolder::new();
        // TODO: populate with data
        Ok(Self { inner, db })
    }
}

impl OwnerProvider for DbHolder {
    type Key = XpubDerivable;
    type UtxoSet = DbUtxos;

    fn descriptor(&self) -> &RgbDescr<Self::Key> { self.inner.descriptor() }

    fn utxos(&self) -> &Self::UtxoSet { self.inner.utxos() }

    fn descriptor_mut(&mut self) -> &mut RgbDescr<Self::Key> { self.inner.descriptor_mut() }

    fn utxos_mut(&mut self) -> &mut Self::UtxoSet { self.inner.utxos_mut() }
}

pub struct DbUtxos {
    db: Database<'static>,
}

impl DbUtxos {
    pub fn utxos(&self) -> HashSet<Utxo> { todo!() }
}

impl UtxoSet for DbUtxos {
    fn len(&self) -> usize { todo!() }

    fn has(&self, outpoint: Outpoint) -> bool { todo!() }

    fn get(&self, outpoint: Outpoint) -> Option<(Sats, Terminal)> { todo!() }

    fn insert(&mut self, outpoint: Outpoint, value: Sats, terminal: Terminal) { todo!() }

    fn insert_all(&mut self, utxos: impl IntoIterator<Item = Utxo>) { todo!() }

    fn clear(&mut self) { todo!() }

    fn remove(&mut self, outpoint: Outpoint) -> Option<(Sats, Terminal)> { todo!() }

    fn remove_all(&mut self, outpoints: impl IntoIterator<Item = Outpoint>) { todo!() }

    fn outpoints(&self) -> impl Iterator<Item = Outpoint> {
        todo!();
        std::iter::empty()
    }

    fn next_index_noshift(&self, keychain: impl Into<Keychain>) -> NormalIndex { todo!() }

    fn next_index(&mut self, keychain: impl Into<Keychain>, shift: bool) -> NormalIndex { todo!() }
}
