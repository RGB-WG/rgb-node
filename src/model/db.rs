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

use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::rc::Rc;
use std::sync::LazyLock;

use bpstd::psbt::Utxo;
use bpstd::{DescrId, Idx, Keychain, NormalIndex, Outpoint, Sats, Terminal, XpubDerivable};
use native_db::transaction::{RTransaction, RwTransaction};
use native_db::{Builder, Database, Models, db_type};
use rgbp::descriptors::RgbDescr;
use rgbp::{Holder, MultiHolder, OwnerProvider, UtxoSet};

use crate::model::utxo::UtxoModelKey;
use crate::model::{OutpointModel, UtxoModel, WalletModel};

static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<WalletModel>().unwrap();
    models.define::<UtxoModel>().unwrap();
    models
});

pub struct DbHolder {
    inner: MultiHolder<XpubDerivable, DbUtxos>,
    db: Rc<RefCell<Database<'static>>>,
}

impl Deref for DbHolder {
    type Target = MultiHolder<XpubDerivable, DbUtxos>;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl DerefMut for DbHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl DbHolder {
    const FILE: &'static str = "wallets.dat";

    pub fn init(path: impl AsRef<Path>) -> Result<(), db_type::Error> {
        Builder::new().create(&MODELS, path.as_ref().join(Self::FILE))?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, db_type::Error> {
        let db = Builder::new().open(&MODELS, path.as_ref().join(Self::FILE))?;
        let db = Rc::new(RefCell::new(db));
        let mut inner = MultiHolder::new();
        {
            let borrow = db.borrow();
            let r = borrow.r_transaction()?;
            for model in r.scan().primary::<WalletModel>()?.all()? {
                let model = model?;
                let descr_id = model.descr_id();
                let db_utxo = DbUtxos { id: descr_id, name: model.name, db: db.clone() };
                let holder = Holder::with_components(model.descriptor, db_utxo);
                inner.upsert(descr_id, holder)
            }
        }
        Ok(Self { inner, db })
    }

    pub fn name(&self) -> String { self.inner.current().utxos().name() }
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
    id: DescrId,
    name: String,
    db: Rc<RefCell<Database<'static>>>,
}

impl DbUtxos {
    fn with_reader<T>(&self, f: impl FnOnce(RTransaction) -> Result<T, db_type::Error>) -> T {
        let borrow = self.db.borrow();
        let tx = borrow.r_transaction().expect("unable to start transaction");
        f(tx).expect("database read operation has failed")
    }

    fn with_writer(&mut self, f: impl FnOnce(&RwTransaction) -> Result<(), db_type::Error>) {
        let borrow = self.db.borrow_mut();
        let tx = borrow
            .rw_transaction()
            .expect("unable to start transaction");
        f(&tx).expect("database write operation has failed");
        tx.commit().expect("database data can't be committed");
    }

    fn all(&self) -> impl Iterator<Item = Utxo> {
        self.with_reader(|tx| {
            let scan = tx.scan().primary::<UtxoModel>()?;
            Ok(scan
                .all()?
                .map(|res| res.unwrap().into())
                .collect::<Vec<_>>()
                .into_iter())
        })
    }

    fn utxo(&self, outpoint: Outpoint) -> Option<UtxoModel> {
        self.with_reader(|tx| {
            tx.get()
                .secondary::<UtxoModel>(UtxoModelKey::outpoint, OutpointModel(outpoint))
        })
    }

    fn model(&self) -> WalletModel {
        self.with_reader(|tx| {
            Ok(tx
                .get()
                .primary::<WalletModel>(self.id.0)?
                .expect("descriptor not found"))
        })
    }

    pub fn name(&self) -> String { self.name.clone() }
    pub fn utxos(&self) -> HashSet<Utxo> { self.all().collect() }
}

impl UtxoSet for DbUtxos {
    fn len(&self) -> usize { self.with_reader(|tx| tx.len().primary::<UtxoModel>()) as usize }

    fn has(&self, outpoint: Outpoint) -> bool { self.get(outpoint).is_some() }

    fn get(&self, outpoint: Outpoint) -> Option<(Sats, Terminal)> {
        let rec = self.utxo(outpoint)?;
        Some((rec.value, rec.terminal))
    }

    fn insert(&mut self, outpoint: Outpoint, value: Sats, terminal: Terminal) {
        let descr = self.id;
        self.with_writer(|tx| {
            let utxo = UtxoModel { descr, outpoint: OutpointModel(outpoint), value, terminal };
            tx.insert(utxo)
        });
    }

    fn insert_all(&mut self, utxos: impl IntoIterator<Item = Utxo>) {
        let id = self.id;
        self.with_writer(|tx| {
            for utxo in utxos {
                tx.insert(UtxoModel::with(id, utxo))?;
            }
            Ok(())
        });
    }

    fn clear(&mut self) {
        self.with_writer(|tx| {
            for row in tx.scan().primary::<UtxoModel>()?.all()? {
                tx.remove(row?)?;
            }
            Ok(())
        });
    }

    fn remove(&mut self, outpoint: Outpoint) -> Option<(Sats, Terminal)> {
        let utxo = self.utxo(outpoint)?;
        self.with_writer(|tx| {
            tx.remove(utxo)?;
            Ok(())
        });
        Some((utxo.value, utxo.terminal))
    }

    fn remove_all(&mut self, outpoints: impl IntoIterator<Item = Outpoint>) {
        self.with_writer(|tx| {
            for outpoint in outpoints {
                if let Some(utxo) = tx
                    .get()
                    .secondary::<UtxoModel>(UtxoModelKey::outpoint, OutpointModel(outpoint))?
                {
                    tx.remove(utxo)?;
                }
            }
            Ok(())
        });
    }

    fn outpoints(&self) -> impl Iterator<Item = Outpoint> { self.all().map(|utxo| utxo.outpoint) }

    fn next_index_noshift(&self, keychain: impl Into<Keychain>) -> NormalIndex {
        self.model().next_index(keychain)
    }

    fn next_index(&mut self, keychain: impl Into<Keychain>, shift: bool) -> NormalIndex {
        let keychain = keychain.into();
        let mut descr = self.model();
        let next_index = descr.next_index(keychain);
        if shift {
            descr
                .next_index
                .entry(keychain)
                .and_modify(|i| *i = next_index.saturating_inc());
            self.with_writer(|tx| {
                tx.upsert(descr)?;
                Ok(())
            });
        }
        next_index
    }
}

// We send it only once on `WriterService` construction, and then use it from a single thread
// always.
unsafe impl Send for DbHolder {}
unsafe impl Send for DbUtxos {}
