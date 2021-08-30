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
use microservices::FileFormat;
use rgb::{Anchor, AnchorId, NodeId};
use strict_encoding::{StrictDecode, StrictEncode};

use super::Index;
use crate::error::{BootstrapError, ServiceErrorDomain};
use crate::util::file::{file, FileMode};

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Default,
    StrictEncode,
    StrictDecode,
)]
struct BTreeIndexData {
    /// TODO #164: Replace with DisplayFromStr once RGB node will fix node
    ///       display
    // #[cfg_attr(feature = "serde", serde(with =
    // "As::<BTreeMap<DisplayFromStr, DisplayFromStr>>"))]
    node_anchors: BTreeMap<NodeId, AnchorId>,
}

#[derive(Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum BTreeIndexError {
    #[from]
    #[from(io::Error)]
    /// I/O error: {0}
    Io(IoError),

    #[from]
    /// Encoding error: {0}
    Encoding(strict_encoding::Error),

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

    /// Anchor is not found, index is probably broken
    AnchorNotFound,
}

impl From<BTreeIndexError> for ServiceErrorDomain {
    fn from(err: BTreeIndexError) -> Self {
        ServiceErrorDomain::Index(err.to_string())
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
    pub index_dir: PathBuf,
    pub data_format: FileFormat,
}

impl BTreeIndexConfig {
    #[inline]
    pub fn index_dir(&self) -> PathBuf {
        self.index_dir.clone()
    }

    #[inline]
    pub fn index_filename(&self) -> PathBuf {
        self.index_dir()
            .join("index")
            .with_extension(self.data_format.extension())
    }
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

        let index_dir = config.index_dir();
        if !index_dir.exists() {
            debug!(
                "RGB index directory '{:?}' is not found; creating one",
                index_dir
            );
            fs::create_dir_all(index_dir)?;
        }

        let mut me = Self {
            config,
            index: empty!(),
        };

        if me.config.index_filename().exists() {
            me.load()?;
            debug!("Index data read");
        } else {
            debug!(
                "Initializing index file {:?} ...",
                me.config.index_filename()
            );
            me.store()?;
        }

        Ok(me)
    }

    fn load(&mut self) -> Result<(), BTreeIndexError> {
        debug!("Reading index information ...");
        let mut f = file(&self.config.index_filename(), FileMode::Read)?;
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
        trace!("Saving index information ...");
        let _ = fs::remove_file(&self.config.index_filename());
        let mut f = file(&self.config.index_filename(), FileMode::Create)?;
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
            .node_anchors
            .get(&node_id)
            .copied()
            .ok_or(BTreeIndexError::AnchorNotFound)
    }

    fn index_anchor(&mut self, anchor: &Anchor) -> Result<bool, Self::Error> {
        for commitment in anchor
            .commitment
            .commitments
            .iter()
            .filter(|commitment| commitment.protocol.is_some())
            .map(|commitment| commitment.message)
        {
            self.index.node_anchors.insert(
                NodeId::from_inner(<NodeId as Wrapper>::Inner::from_inner(
                    commitment.into_inner(),
                )),
                anchor.anchor_id(),
            );
        }
        self.store()?;
        Ok(true)
    }
}
