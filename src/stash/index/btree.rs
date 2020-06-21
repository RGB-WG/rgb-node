// RGB standard library
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeMap;

use lnpbp::rgb::{Anchor, AnchorId, TransitionId};

use super::Index;
use crate::error::{BootstrapError, ServiceErrorDomain};

#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum BTreeIndexError {}

impl From<BTreeIndexError> for ServiceErrorDomain {
    fn from(_: BTreeIndexError) -> Self {
        ServiceErrorDomain::Storage
    }
}

impl From<BTreeIndexError> for BootstrapError {
    fn from(_: BTreeIndexError) -> Self {
        BootstrapError::StorageError
    }
}

#[derive(Display, Debug)]
#[display_from(Debug)]
pub struct BTreeIndex {
    index: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl BTreeIndex {
    pub fn new() -> Self {
        Self { index: bmap! {} }
    }
}

impl Index for BTreeIndex {
    type Error = BTreeIndexError;

    fn anchor_id_by_transition_id(&self, _tsid: TransitionId) -> Result<AnchorId, Self::Error> {
        unimplemented!()
    }

    fn index_anchor(&mut self, _anchor: &Anchor) -> Result<bool, Self::Error> {
        unimplemented!()
    }
}
