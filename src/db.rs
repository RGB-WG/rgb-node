// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::BTreeSet;
use std::fmt::Debug;

use amplify::Slice32;
use bitcoin::hashes::{sha256t, Hash};
use commit_verify::TaggedHash;
use internet2::addr::ServiceAddr;
use rgb::schema::TransitionType;
use rgb::{ContractId, MergeReveal, NodeId};
use storm::ChunkId;
use strict_encoding::{StrictDecode, StrictEncode};

use crate::{DaemonError, LaunchError};

pub(crate) struct Db {
    pub(crate) store: store_rpc::Client,
}

impl Db {
    pub const SCHEMATA: &'static str = "schemata";
    pub const CONTRACTS: &'static str = "contracts";
    pub const BUNDLES: &'static str = "bundles";
    pub const GENESIS: &'static str = "genesis";
    pub const TRANSITIONS: &'static str = "transitions";
    pub const ANCHORS: &'static str = "anchors";
    pub const EXTENSIONS: &'static str = "extensions";
    pub const ATTACHMENT_CHUNKS: &'static str = "chunks";
    pub const ATTACHMENT_INDEX: &'static str = "attachments";
    pub const ALU_LIBS: &'static str = "alu";

    pub const CONTRACT_TRANSITIONS: &'static str = "contract_transitions";

    pub fn index_two_pieces(a: impl StrictEncode, b: impl StrictEncode) -> ChunkId {
        let mut engine = ChunkId::engine();
        let _ = a.strict_encode(&mut engine);
        let _ = b.strict_encode(&mut engine);
        ChunkId::from_engine(engine)
    }

    pub fn with(store_endpoint: &ServiceAddr) -> Result<Db, LaunchError> {
        let mut store = store_rpc::Client::with(store_endpoint).map_err(LaunchError::from)?;

        for table in [
            Db::SCHEMATA,
            Db::CONTRACTS,
            Db::BUNDLES,
            Db::GENESIS,
            Db::TRANSITIONS,
            Db::ANCHORS,
            Db::EXTENSIONS,
            Db::ATTACHMENT_CHUNKS,
            Db::ATTACHMENT_INDEX,
            Db::ALU_LIBS,
            Db::CONTRACT_TRANSITIONS,
        ] {
            store.use_table(table.to_owned()).map_err(LaunchError::from)?;
        }

        Ok(Db { store })
    }

    pub fn transitions_by_type(
        &mut self,
        contract_id: ContractId,
        transition_type: TransitionType,
    ) -> Result<BTreeSet<NodeId>, DaemonError> {
        let chunk_id = Db::index_two_pieces(contract_id, transition_type);
        Ok(self.retrieve_h(Db::CONTRACT_TRANSITIONS, chunk_id)?.unwrap_or_default())
    }

    pub fn insert_into_set<T>(
        &mut self,
        table: &'static str,
        id: ChunkId,
        item: T,
    ) -> Result<(), DaemonError>
    where
        T: Ord,
        BTreeSet<T>: StrictEncode + StrictDecode,
    {
        let mut set: BTreeSet<T> = self.retrieve_h(table, id)?.unwrap_or_default();
        set.insert(item);
        self.store_h(table, id, &set)?;
        Ok(())
    }

    pub fn retrieve<'a, H: 'a + sha256t::Tag, T: StrictDecode>(
        &mut self,
        table: &'static str,
        key: impl TaggedHash<'a, H> + Debug + 'a,
    ) -> Result<Option<T>, DaemonError> {
        debug!("Read object {:?}", key);
        let slice = key.into_inner();
        let slice = slice.into_inner();
        match self.store.retrieve(table.to_owned(), Slice32::from(slice))? {
            Some(data) => Ok(Some(T::strict_decode(data.as_ref())?)),
            None => {
                warn!("Object not found");
                Ok(None)
            }
        }
    }

    pub fn retrieve_h<T: StrictDecode>(
        &mut self,
        table: &'static str,
        key: impl Hash<Inner = [u8; 32]>,
    ) -> Result<Option<T>, DaemonError> {
        let slice = *key.as_inner();
        debug!("Read object {}", key);
        match self.store.retrieve(table.to_owned(), Slice32::from(slice))? {
            Some(data) => Ok(Some(T::strict_decode(data.as_ref())?)),
            None => {
                warn!("Object {} not found", key);
                Ok(None)
            }
        }
    }

    pub fn store<'a, H: 'a + sha256t::Tag>(
        &mut self,
        table: &'static str,
        key: impl TaggedHash<'a, H> + Debug + 'a,
        data: &impl StrictEncode,
    ) -> Result<(), DaemonError> {
        debug!("Store object {:?}", key);
        let slice = key.into_inner();
        let slice = slice.into_inner();
        self.store.store(table.to_owned(), Slice32::from(slice), data.strict_serialize()?)?;
        Ok(())
    }

    pub fn store_h(
        &mut self,
        table: &'static str,
        key: impl Hash<Inner = [u8; 32]>,
        data: &impl StrictEncode,
    ) -> Result<(), DaemonError> {
        debug!("Store object {}", key);
        let slice = *key.as_inner();
        self.store.store(table.to_owned(), Slice32::from(slice), data.strict_serialize()?)?;
        Ok(())
    }

    pub fn store_merge<'a, H: 'a + sha256t::Tag>(
        &mut self,
        table: &'static str,
        key: impl TaggedHash<'a, H> + Debug + Copy + 'a,
        new_obj: impl StrictEncode + StrictDecode + MergeReveal + Clone,
    ) -> Result<(), DaemonError> {
        debug!("Store-merging object {:?}", key);
        // FIXME: Racing conditions are possible
        let stored_obj = self.retrieve(table, key)?.unwrap_or_else(|| {
            debug!("Object {:?} is not yet stored in the database", key);
            new_obj.clone()
        });
        let obj = new_obj
            .merge_reveal(stored_obj)
            .expect("merge-revealed objects does not match; usually it means hacked database");
        self.store(Db::GENESIS, key, &obj)
    }

    pub fn store_merge_h(
        &mut self,
        table: &'static str,
        key: impl Hash<Inner = [u8; 32]>,
        new_obj: impl StrictEncode + StrictDecode + MergeReveal + Clone,
    ) -> Result<(), DaemonError> {
        debug!("Store-merging object {}", key);
        // FIXME: Racing conditions are possible
        let stored_obj = self.retrieve_h(table, key)?.unwrap_or_else(|| {
            debug!("Object {} is not yet stored in the database", key);
            new_obj.clone()
        });
        let obj = new_obj
            .merge_reveal(stored_obj)
            .expect("merge-revealed objects does not match; usually it means hacked database");
        self.store_h(Db::GENESIS, key, &obj)
    }
}
