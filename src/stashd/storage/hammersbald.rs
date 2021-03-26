// RGB standard library
// Written in 2020 by
//     Rajarshi Maitra <rajarshi149@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.
use hammersbald::{persistent, HammersbaldAPI};

use super::store::Store;
use crate::error::{BootstrapError, ServiceErrorDomain};
use lnpbp::strict_encoding::{strict_serialize, StrictDecode};
use rgb::prelude::*;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Display, Error, From)]
#[display(Debug)]
pub enum HammersbaldError {
    #[from]
    Io(io::Error),

    #[from(bitcoin::hashes::Error)]
    HashName,

    #[from]
    Encoding(lnpbp::strict_encoding::Error),

    #[from(bitcoin::hashes::hex::Error)]
    #[from(rgb::bech32::Error)]
    BrokenFilenames,

    #[from]
    Hammersbald(hammersbald::Error),

    DataDirNotFound,

    DataNotFound,
}

impl From<HammersbaldError> for ServiceErrorDomain {
    fn from(err: HammersbaldError) -> Self {
        ServiceErrorDomain::Storage(err.to_string())
    }
}

impl From<HammersbaldError> for BootstrapError {
    fn from(_: HammersbaldError) -> Self {
        BootstrapError::StorageError
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(Debug)]
pub struct HammersbaldConfig {
    pub data_dir: PathBuf,
    pub cached_pages: usize,       // sample value 100
    pub bucket_fill_targes: usize, // sample value 2
}

impl HammersbaldConfig {
    #[inline]
    pub fn schemata_db(&self) -> PathBuf {
        self.data_dir.join("hammersbald").join("schemata")
    }

    #[inline]
    pub fn geneses_db(&self) -> PathBuf {
        self.data_dir.join("hammersbald").join("geneses")
    }

    #[inline]
    pub fn anchors_db(&self) -> PathBuf {
        self.data_dir.join("hammersbald").join("anchors")
    }

    #[inline]
    pub fn transitions_db(&self) -> PathBuf {
        self.data_dir.join("hammersbald").join("transitions")
    }

    #[inline]
    pub fn extensions_db(&self) -> PathBuf {
        self.data_dir.join("hammersbald").join("extensions")
    }
}

/// Keeps all Hammersbald RGB contract data, stash etc
pub struct HammersbaldStorage {
    schemata_db: Box<dyn HammersbaldAPI>,
    geneses_db: Box<dyn HammersbaldAPI>,
    anchors_db: Box<dyn HammersbaldAPI>,
    transitions_db: Box<dyn HammersbaldAPI>,
    extensions_db: Box<dyn HammersbaldAPI>,
}

impl HammersbaldStorage {
    pub fn new(config: HammersbaldConfig) -> Result<Self, HammersbaldError> {
        let data_dir = config.data_dir.clone().join("hammersbald");
        if !data_dir.exists() {
            println!("Datadir doesn't exist, creating one");
            fs::create_dir_all(data_dir)?;
        }

        let schemata_db = persistent(
            config
                .schemata_db()
                .to_str()
                .ok_or(HammersbaldError::DataDirNotFound)?,
            config.cached_pages,
            config.bucket_fill_targes,
        )?;

        let geneses_db = persistent(
            config
                .geneses_db()
                .to_str()
                .ok_or(HammersbaldError::DataDirNotFound)?,
            config.cached_pages,
            config.bucket_fill_targes,
        )?;

        let anchors_db = persistent(
            config
                .anchors_db()
                .to_str()
                .ok_or(HammersbaldError::DataDirNotFound)?,
            config.cached_pages,
            config.bucket_fill_targes,
        )?;

        let transitions_db = persistent(
            config
                .transitions_db()
                .to_str()
                .ok_or(HammersbaldError::DataDirNotFound)?,
            config.cached_pages,
            config.bucket_fill_targes,
        )?;

        let extensions_db = persistent(
            config
                .extensions_db()
                .to_str()
                .ok_or(HammersbaldError::DataDirNotFound)?,
            config.cached_pages,
            config.bucket_fill_targes,
        )?;

        Ok(Self {
            schemata_db,
            geneses_db,
            anchors_db,
            transitions_db,
            extensions_db,
        })
    }
}

impl Store for HammersbaldStorage {
    type Error = HammersbaldError;

    fn schema_ids(&self) -> Result<Vec<SchemaId>, Self::Error> {
        let mut result = vec![];
        for item in self.schemata_db.iter() {
            result.push(SchemaId::strict_decode(&item.1[..])?);
        }
        Ok(result)
    }

    fn schema(&self, id: &SchemaId) -> Result<Schema, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self
            .schemata_db
            .get_keyed(&key[..])?
            .ok_or(HammersbaldError::DataNotFound)?;
        let schema = Schema::strict_decode(&value.1[..])?;
        Ok(schema)
    }

    fn has_schema(&self, id: &SchemaId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self.schemata_db.get_keyed(&key[..])?;
        match value {
            Some(_) => return Ok(true),
            None => return Ok(false),
        }
    }

    fn add_schema(&mut self, schema: &Schema) -> Result<bool, Self::Error> {
        let schema_id = schema.schema_id();
        let key = strict_serialize(&schema_id)?;
        let value = strict_serialize(schema)?;
        self.schemata_db.put_keyed(&key[..], &value[..])?;
        Ok(true)
    }

    fn remove_schema(&mut self, id: &SchemaId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        self.schemata_db.forget(&key[..])?;
        Ok(true)
    }

    fn contract_ids(&self) -> Result<Vec<ContractId>, Self::Error> {
        let mut result = vec![];
        for item in self.geneses_db.iter() {
            result.push(ContractId::strict_decode(&item.1[..])?);
        }
        Ok(result)
    }

    fn genesis(&self, id: &ContractId) -> Result<Genesis, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self
            .geneses_db
            .get_keyed(&key[..])?
            .ok_or(HammersbaldError::DataNotFound)?;
        let genesis = Genesis::strict_decode(&value.1[..])?;
        Ok(genesis)
    }

    fn has_genesis(&self, id: &ContractId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self.geneses_db.get_keyed(&key[..])?;
        match value {
            Some(_) => return Ok(true),
            None => return Ok(false),
        }
    }

    fn add_genesis(&mut self, genesis: &Genesis) -> Result<bool, Self::Error> {
        let contract_id = genesis.contract_id();
        let key = strict_serialize(&contract_id)?;
        let value = strict_serialize(genesis)?;
        self.geneses_db.put_keyed(&key[..], &value[..])?;
        Ok(true)
    }

    fn remove_genesis(&mut self, id: &ContractId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        self.geneses_db.forget(&key[..])?;
        Ok(true)
    }

    fn anchor(&self, id: &AnchorId) -> Result<Anchor, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self
            .anchors_db
            .get_keyed(&key[..])?
            .ok_or(HammersbaldError::DataNotFound)?;
        let anchor = Anchor::strict_decode(&value.1[..])?;
        Ok(anchor)
    }

    fn has_anchor(&self, id: &AnchorId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self.anchors_db.get_keyed(&key[..])?;
        match value {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn add_anchor(&mut self, anchor: &Anchor) -> Result<bool, Self::Error> {
        let anchor_id = anchor.anchor_id();
        let key = strict_serialize(&anchor_id)?;
        let value = strict_serialize(anchor)?;
        self.anchors_db.put_keyed(&key[..], &value[..])?;
        Ok(true)
    }

    fn remove_anchor(&mut self, id: &AnchorId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        self.anchors_db.forget(&key[..])?;
        Ok(true)
    }

    fn transition(&self, id: &NodeId) -> Result<Transition, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self
            .transitions_db
            .get_keyed(&key[..])?
            .ok_or(HammersbaldError::DataNotFound)?;
        let transition = Transition::strict_decode(&value.1[..])?;
        Ok(transition)
    }

    fn has_transition(&self, id: &NodeId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self.transitions_db.get_keyed(&key[..])?;
        match value {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn add_transition(
        &mut self,
        transition: &Transition,
    ) -> Result<bool, Self::Error> {
        let node_id = transition.node_id();
        let key = strict_serialize(&node_id)?;
        let value = strict_serialize(transition)?;
        self.transitions_db.put_keyed(&key[..], &value[..])?;
        Ok(true)
    }

    fn remove_transition(&mut self, id: &NodeId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        self.transitions_db.forget(&key[..])?;
        Ok(true)
    }

    fn extension(&self, id: &NodeId) -> Result<Extension, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self
            .extensions_db
            .get_keyed(&key[..])?
            .ok_or(HammersbaldError::DataNotFound)?;
        let extension = Extension::strict_decode(&value.1[..])?;
        Ok(extension)
    }

    fn has_extension(&self, id: &NodeId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        let value = self.extensions_db.get_keyed(&key[..])?;
        match value {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn add_extension(
        &mut self,
        extension: &Extension,
    ) -> Result<bool, Self::Error> {
        let node_id = extension.node_id();
        let key = strict_serialize(&node_id)?;
        let value = strict_serialize(extension)?;
        self.extensions_db.put_keyed(&key[..], &value[..])?;
        Ok(true)
    }

    fn remove_extension(&mut self, id: &NodeId) -> Result<bool, Self::Error> {
        let key = strict_serialize(id)?;
        self.extensions_db.forget(&key[..])?;
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rgb::schema::{
        Bits, DataFormat, DiscreteFiniteFieldFormat, GenesisSchema, Occurences,
        StateFormat, StateSchema, TransitionSchema,
    };
    use std::env;

    // Create a sample schema for testing
    fn schema() -> Schema {
        const FIELD_TICKER: usize = 0;
        const FIELD_NAME: usize = 1;
        const FIELD_DESCRIPTION: usize = 2;
        const FIELD_TOTAL_SUPPLY: usize = 3;
        const FIELD_ISSUED_SUPPLY: usize = 4;
        const FIELD_DUST_LIMIT: usize = 5;
        const FIELD_PRECISION: usize = 6;
        const FIELD_PRUNE_PROOF: usize = 7;
        const FIELD_TIMESTAMP: usize = 8;

        const FIELD_PROOF_OF_BURN: usize = 0x10;

        const ASSIGNMENT_ISSUE: usize = 0;
        const ASSIGNMENT_ASSETS: usize = 1;
        const ASSIGNMENT_PRUNE: usize = 2;

        const TRANSITION_ISSUE: usize = 0;
        const TRANSITION_TRANSFER: usize = 1;
        const TRANSITION_PRUNE: usize = 2;

        const VALENCIES_DECENTRALIZED_ISSUE: usize = 0;

        const EXTENSION_DECENTRALIZED_ISSUE: usize = 0;

        Schema {
            rgb_features: Default::default(),
            root_id: Default::default(),
            field_types: bmap! {
                FIELD_TICKER => DataFormat::String(16),
                FIELD_NAME => DataFormat::String(256),
                FIELD_DESCRIPTION => DataFormat::String(1024),
                FIELD_TOTAL_SUPPLY => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
                FIELD_PRECISION => DataFormat::Unsigned(Bits::Bit64, 0, 18u128),
                FIELD_ISSUED_SUPPLY => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
                FIELD_DUST_LIMIT => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
                FIELD_PRUNE_PROOF => DataFormat::Bytes(core::u16::MAX),
                FIELD_TIMESTAMP => DataFormat::Unsigned(Bits::Bit64, 0, core::u64::MAX as u128),
                FIELD_PROOF_OF_BURN => DataFormat::TxOutPoint
            },
            owned_right_types: bmap! {
                ASSIGNMENT_ISSUE => StateSchema {
                    format: StateFormat::Declarative,
                    abi: bmap! {
                        AssignmentAction::Validate => script::Procedure::Embedded(script::StandardProcedure::FungibleInflation)
                    }
                },
                ASSIGNMENT_ASSETS => StateSchema {
                    format: StateFormat::DiscreteFiniteField(DiscreteFiniteFieldFormat::Unsigned64bit),
                    abi: bmap! {
                        AssignmentAction::Validate => script::Procedure::Embedded(script::StandardProcedure::NoInflationBySum)
                    }
                },
                ASSIGNMENT_PRUNE => StateSchema {
                    format: StateFormat::Declarative,
                    abi: bmap! {
                        AssignmentAction::Validate => script::Procedure::Embedded(script::StandardProcedure::ProofOfBurn)
                    }
                }
            },
            public_right_types: bset! {
                VALENCIES_DECENTRALIZED_ISSUE
            },
            genesis: GenesisSchema {
                metadata: bmap! {
                    FIELD_TICKER => Occurences::Once,
                    FIELD_NAME => Occurences::Once,
                    FIELD_DESCRIPTION => Occurences::NoneOrOnce,
                    FIELD_TOTAL_SUPPLY => Occurences::Once,
                    FIELD_ISSUED_SUPPLY => Occurences::Once,
                    FIELD_DUST_LIMIT => Occurences::NoneOrOnce,
                    FIELD_PRECISION => Occurences::Once,
                    FIELD_TIMESTAMP => Occurences::Once
                },
                owned_rights: bmap! {
                    ASSIGNMENT_ISSUE => Occurences::NoneOrOnce,
                    ASSIGNMENT_ASSETS => Occurences::NoneOrMore,
                    ASSIGNMENT_PRUNE => Occurences::NoneOrMore
                },
                public_rights: bset! { VALENCIES_DECENTRALIZED_ISSUE },
                abi: bmap! {},
            },
            extensions: bmap! {
                EXTENSION_DECENTRALIZED_ISSUE => ExtensionSchema {
                    extends: bset! { VALENCIES_DECENTRALIZED_ISSUE },
                    metadata: bmap! {
                        FIELD_ISSUED_SUPPLY => Occurences::Once,
                        FIELD_PROOF_OF_BURN => Occurences::OnceOrMore
                    },
                    owned_rights: bmap! {
                        ASSIGNMENT_ASSETS => Occurences::NoneOrMore
                    },
                    public_rights: bset! { },
                    abi: bmap! {},
                }
            },
            transitions: bmap! {
                TRANSITION_ISSUE => TransitionSchema {
                    closes: bmap! {
                        ASSIGNMENT_ISSUE => Occurences::Once
                    },
                    metadata: bmap! {
                        FIELD_ISSUED_SUPPLY => Occurences::Once
                    },
                    owned_rights: bmap! {
                        ASSIGNMENT_ISSUE => Occurences::NoneOrOnce,
                        ASSIGNMENT_PRUNE => Occurences::NoneOrMore,
                        ASSIGNMENT_ASSETS => Occurences::NoneOrMore
                    },
                    public_rights: bset! {},
                    abi: bmap! {}
                },
                TRANSITION_TRANSFER => TransitionSchema {
                    closes: bmap! {
                        ASSIGNMENT_ASSETS => Occurences::OnceOrMore
                    },
                    metadata: bmap! {},
                    owned_rights: bmap! {
                        ASSIGNMENT_ASSETS => Occurences::NoneOrMore
                    },
                    public_rights: bset! {},
                    abi: bmap! {}
                },
                TRANSITION_PRUNE => TransitionSchema {
                    closes: bmap! {
                        ASSIGNMENT_PRUNE => Occurences::OnceOrMore,
                        ASSIGNMENT_ASSETS => Occurences::OnceOrMore
                    },
                    metadata: bmap! {
                        FIELD_PRUNE_PROOF => Occurences::NoneOrMore
                    },
                    owned_rights: bmap! {
                        ASSIGNMENT_PRUNE => Occurences::NoneOrMore,
                        ASSIGNMENT_ASSETS => Occurences::NoneOrMore
                    },
                    public_rights: bset! {},
                    abi: bmap! {}
                }
            },
        }
    }

    // TODO #165: Add testing for Anchors when easy anchor sample
    // are availble
    // To run the test set an env variable `export
    // DATABASE_URL='~/.rgb/hammersbald-tests/'
    #[test]
    fn test_hammersbald_db() {
        let schema = schema();
        let schema_id = schema.schema_id();
        let genesis = Genesis::default();
        let contract_id = genesis.contract_id();
        let transition = Transition::default();
        let transition_node_id = transition.node_id();
        let extension = Extension::default();
        let extension_node_id = extension.node_id();

        let database_url = env::var("DATABASE_URL").expect(
            "Environment Variable 'DATABASE_URL' must be set to run this test",
        );

        let config = HammersbaldConfig {
            data_dir: std::path::PathBuf::from(&database_url[..]),
            cached_pages: 100,
            bucket_fill_targes: 2,
        };

        let mut database = HammersbaldStorage::new(config).unwrap();

        assert!(database.add_schema(&schema).unwrap());
        assert!(database.has_schema(&schema_id).unwrap());
        assert_eq!(schema, database.schema(&schema_id).unwrap());
        assert_eq!(vec![schema_id], database.schema_ids().unwrap());
        assert!(database.remove_schema(&schema_id).unwrap());

        assert!(database.add_genesis(&genesis).unwrap());
        assert!(database.has_genesis(&contract_id).unwrap());
        assert_eq!(database.genesis(&contract_id).unwrap(), genesis);
        assert!(database.remove_genesis(&contract_id).unwrap());

        assert!(database.add_transition(&transition).unwrap());
        assert!(database.has_transition(&transition_node_id).unwrap());
        assert_eq!(
            database.transition(&transition_node_id).unwrap(),
            transition
        );

        assert!(database.remove_transition(&transition_node_id).unwrap());
        assert!(database.add_extension(&extension).unwrap());
        assert!(database.has_extension(&extension_node_id).unwrap());
        assert_eq!(database.extension(&extension_node_id).unwrap(), extension);
        assert!(database.remove_extension(&extension_node_id).unwrap());
    }
}
