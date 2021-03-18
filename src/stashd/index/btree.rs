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
use std::io::{self, Read, Write};
use std::path::PathBuf;

use amplify::{IoError, Wrapper};
use bitcoin::hashes::Hash;
use lnpbp::strict_encoding::{StrictDecode, StrictEncode};
use microservices::FileFormat;
use rgb::{Anchor, AnchorId, NodeId};

use super::Index;
use crate::error::{BootstrapError, ServiceErrorDomain};
use crate::util::file::{file, FileMode};

type BTreeIndexData = BTreeMap<Vec<u8>, Vec<u8>>;

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum BTreeIndexError {
    #[from]
    #[from(io::Error)]
    /// I/O error: {0}
    Io(IoError),

    #[from]
    /// Encoding error: {0}
    Encoding(lnpbp::strict_encoding::Error),

    #[cfg(feature = "serde")]
    #[from]
    /// Encoding error: {0}
    SerdeJson(serde_json::Error),

    #[cfg(feature = "serde")]
    #[from]
    /// Encoding error: {0}
    SerdeYaml(serde_yaml::Error),

    #[cfg(feature = "serde")]
    #[from(toml::de::Error)]
    #[from(toml::ser::Error)]
    /// Encoding error: {0}
    SerdeToml,

    /// Anchor is not found
    AnchorNotFound,
}

impl From<BTreeIndexError> for ServiceErrorDomain {
    fn from(err: BTreeIndexError) -> Self {
        ServiceErrorDomain::Storage(err.to_string())
    }
}

impl From<BTreeIndexError> for BootstrapError {
    fn from(_: BTreeIndexError) -> Self {
        BootstrapError::StorageError
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(Debug)]
pub struct BTreeIndexConfig {
    pub index_file: PathBuf,
    pub data_format: FileFormat,
}

#[derive(Display, Debug)]
#[display(Debug)]
pub struct BTreeIndex {
    config: BTreeIndexConfig,
    index: BTreeIndexData,
}

impl BTreeIndex {
    pub fn new(config: BTreeIndexConfig) -> Result<Self, BTreeIndexError> {
        debug!("Instantiating RGB index (file storage) ...");

        let mut me = Self {
            config,
            index: bmap! {},
        };

        if me.config.index_file.exists() {
            me.load()?;
        } else {
            debug!("Initializing assets file {:?} ...", me.config.index_file);
            me.store()?;
        }

        Ok(me)
    }

    fn load(&mut self) -> Result<(), BTreeIndexError> {
        debug!("Reading assets information ...");
        let mut f = file(&self.config.index_file, FileMode::Read)?;
        self.index = match self.config.data_format {
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => serde_yaml::from_reader(&f)?,
            #[cfg(feature = "serde_json")]
            FileFormat::Json => serde_json::from_reader(&f)?,
            #[cfg(feature = "toml")]
            FileFormat::Toml => {
                let mut data = String::new();
                f.read_to_string(&mut data)?;
                toml::from_str(&data)?
            }
            FileFormat::StrictEncode => StrictDecode::strict_decode(&mut f)?,
            _ => unimplemented!(),
        };
        Ok(())
    }

    pub fn store(&self) -> Result<(), BTreeIndexError> {
        trace!("Saving assets information ...");
        let _ = fs::remove_file(&self.config.index_file);
        let mut f = file(&self.config.index_file, FileMode::Create)?;
        match self.config.data_format {
            #[cfg(feature = "serde_yaml")]
            FileFormat::Yaml => serde_yaml::to_writer(&f, &self.index)?,
            #[cfg(feature = "serde_json")]
            FileFormat::Json => serde_json::to_writer(&f, &self.index)?,
            #[cfg(feature = "toml")]
            FileFormat::Toml => f.write_all(&toml::to_vec(&self.index)?)?,
            FileFormat::StrictEncode => {
                self.index.strict_encode(&mut f)?;
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}

impl Index for BTreeIndex {
    type Error = BTreeIndexError;

    fn anchor_id_by_transition_id(
        &self,
        node_id: NodeId,
    ) -> Result<AnchorId, Self::Error> {
        self.index
            .get(&node_id.to_vec())
            .and_then(|vec| {
                Some(AnchorId::from_inner(
                    <AnchorId as Wrapper>::Inner::from_slice(&vec[..]).ok()?,
                ))
            })
            .ok_or(BTreeIndexError::AnchorNotFound)
    }

    fn index_anchor(&mut self, anchor: &Anchor) -> Result<bool, Self::Error> {
        for protocol in anchor
            .commitment
            .commitments
            .iter()
            .filter_map(|commitment| commitment.protocol)
        {
            self.index.insert(protocol.to_vec(), anchor.txid.to_vec());
        }
        self.store()?;
        Ok(true)
    }
}
