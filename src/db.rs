// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

pub const SCHEMATA: &str = "schemata";
pub const CONTRACTS: &str = "contracts";
pub const BUNDLES: &str = "bundles";
pub const GENESIS: &str = "genesis";
pub const TRANSITIONS: &str = "transitions";
pub const ANCHORS: &str = "anchors";
pub const EXTENSIONS: &str = "extensions";
pub const ALU_LIBS: &str = "alu";

pub const OUTPOINTS: &str = "outpoints";
pub const NODE_CONTRACTS: &str = "node_contracts";
pub const TRANSITION_WITNESS: &str = "transition_txid";
pub const CONTRACT_TRANSITIONS: &str = "contract_transitions";

pub const DISCLOSURES: &str = "disclosures";

// Storm intgration
pub const ATTACHMENT_CHUNKS: &str = "chunks";
pub const ATTACHMENT_INDEX: &str = "attachments";
pub const ATTACHMENT_CONTAINER_HEADERS: &str = "container_headers";
pub const ATTACHMENT_CONTAINERS: &str = "containers";

pub(crate) trait StoreRpcExt {
    fn retrieve_sten<T>(
        &mut self,
        table: impl ToString,
        key: impl PrimaryKey,
    ) -> Result<Option<T>, DaemonError>
    where
        T: StrictEncodedChunk;

    fn store_sten(
        &mut self,
        table: impl ToString,
        key: impl PrimaryKey,
        data: &impl StrictEncodedChunk,
    ) -> Result<ChunkId, DaemonError>;

    fn store_merge(
        &mut self,
        table: impl ToString,
        key: impl PrimaryKey,
        new_obj: impl StrictEncodedChunk + MergeReveal + Clone,
    ) -> Result<(), DaemonError>;
}

impl StoreRpcExt for store_rpc::Client {
    fn retrieve_sten<T>(
        &mut self,
        table: impl ToString,
        key: impl PrimaryKey,
    ) -> Result<Option<T>, DaemonError>
    where
        T: StrictEncodedChunk,
    {
        let maybe_holder = self.retrieve(table, key)?;
        Ok(maybe_holder.map(ChunkHolder::unbox))
    }

    fn store_sten(
        &mut self,
        table: impl ToString,
        key: impl PrimaryKey,
        data: &impl StrictEncodedChunk,
    ) -> Result<ChunkId, DaemonError> {
        self.store(table, key, &data.chunk()).map_err(DaemonError::from)
    }

    fn store_merge(
        &mut self,
        table: impl ToString,
        key: impl PrimaryKey,
        new_obj: impl StrictEncodedChunk + MergeReveal + Clone,
    ) -> Result<(), DaemonError> {
        let key = key.into_slice32();
        let table = table.to_string();
        debug!("Store-merging object {}", key);
        // FIXME: Racing conditions are possible
        let stored_obj = self.retrieve_sten(&table, key)?.unwrap_or_else(|| {
            debug!("Object {} is not yet stored in the database", key);
            new_obj.clone()
        });
        let obj = new_obj
            .merge_reveal(stored_obj)
            .expect("merge-revealed objects does not match; usually it means hacked database");
        self.store_sten(table, key, &obj)?;
        Ok(())
    }
}

mod encoding {
    use std::collections::BTreeSet;
    use std::io::{Read, Write};

    use bitcoin::Txid;
    use commit_verify::lnpbp4;
    use storm::chunk;
    use strict_encoding::{StrictDecode, StrictEncode};

    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    pub struct ChunkRefHolder<'inner, T>(&'inner T)
    where T: StrictEncode;

    impl<'inner, T> chunk::encoding::ApplyStrictEncoding for ChunkRefHolder<'inner, T> where T: StrictEncode
    {}

    impl<'inner, T> StrictEncode for ChunkRefHolder<'inner, T>
    where T: StrictEncode
    {
        fn strict_encode<E: Write>(&self, e: E) -> Result<usize, strict_encoding::Error> {
            self.0.strict_encode(e)
        }
    }

    pub struct ChunkHolder<T>(T)
    where T: StrictDecode;

    impl<T> chunk::encoding::ApplyStrictEncoding for ChunkHolder<T> where T: StrictDecode {}

    impl<T> StrictDecode for ChunkHolder<T>
    where T: StrictDecode
    {
        fn strict_decode<D: Read>(d: D) -> Result<Self, strict_encoding::Error> {
            T::strict_decode(d).map(Self)
        }
    }

    impl<T> ChunkHolder<T>
    where T: StrictDecode
    {
        pub fn unbox(self) -> T { self.0 }
    }

    pub trait StrictEncodedChunk: StrictEncode + StrictDecode + Clone {
        fn chunk(&self) -> ChunkRefHolder<Self>
        where Self: Sized {
            ChunkRefHolder(self)
        }
    }

    impl StrictEncodedChunk for Txid {}

    // TODO: Probably we need to split disclosures into a multiple chunks
    impl StrictEncodedChunk for rgb::Disclosure {}

    impl StrictEncodedChunk for rgb::SchemaId {}
    impl StrictEncodedChunk for rgb::ContractId {}
    impl StrictEncodedChunk for rgb::NodeId {}
    impl StrictEncodedChunk for rgb::Schema {}
    impl StrictEncodedChunk for rgb::Genesis {}
    impl StrictEncodedChunk for rgb::Transition {}
    impl StrictEncodedChunk for rgb::Extension {}
    impl StrictEncodedChunk for rgb::TransitionBundle {}
    impl StrictEncodedChunk for rgb::Anchor<lnpbp4::MerkleBlock> {}
    impl StrictEncodedChunk for rgb::ContractState {}

    impl StrictEncodedChunk for BTreeSet<rgb::NodeId> {}
    impl StrictEncodedChunk for Vec<(rgb::NodeId, BTreeSet<u16>)> {}
}

pub use encoding::{ChunkHolder, StrictEncodedChunk};
use rgb::MergeReveal;
use store_rpc::PrimaryKey;
use storm::ChunkId;

use crate::DaemonError;
