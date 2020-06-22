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
use std::fs;
use std::io;
use std::path::PathBuf;

use lnpbp::rgb::{Anchor, AnchorId, TransitionId};
use lnpbp::strict_encoding::{StrictDecode, StrictEncode};

use super::Index;
use crate::error::{BootstrapError, ServiceErrorDomain};

type BTreeIndexData = BTreeMap<Vec<u8>, Vec<u8>>;

#[derive(Debug, Display, Error, From)]
#[display_from(Debug)]
pub enum BTreeIndexError {
    #[derive_from]
    Io(io::Error),

    #[derive_from]
    Encoding(lnpbp::strict_encoding::Error),
}

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

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display_from(Debug)]
pub struct BTreeIndexConfig {
    pub index_file: PathBuf,
}

#[derive(Display, Debug)]
#[display_from(Debug)]
pub struct BTreeIndex {
    config: BTreeIndexConfig,
    index: BTreeIndexData,
}

impl BTreeIndex {
    pub fn new(config: BTreeIndexConfig) -> Self {
        debug!("Instantiating RGB index (file & memory storage) ...");
        Self {
            config,
            index: bmap! {},
        }
    }

    pub fn load(config: BTreeIndexConfig) -> Result<Self, BTreeIndexError> {
        if let Ok(file) = fs::File::with_options().read(true).open(&config.index_file) {
            debug!("Loading RGB index from file {:?} ...", &config.index_file);
            Ok(Self {
                config,
                index: BTreeIndexData::strict_decode(file)?,
            })
        } else {
            Ok(Self::new(config))
        }
    }

    pub fn store(&self) -> Result<(), BTreeIndexError> {
        debug!("Saving RGB index to file {:?} ...", &self.config.index_file);
        let file = fs::File::with_options()
            .write(true)
            .create(true)
            .open(&self.config.index_file)?;
        self.index.strict_encode(file)?;
        Ok(())
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
