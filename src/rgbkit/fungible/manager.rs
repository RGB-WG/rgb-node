use chrono::Utc;
use core::convert::TryFrom;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use lnpbp::bp;
use lnpbp::rgb;
use rgb::prelude::*;

use super::storage::Error as AssetStoreError;
use super::{schema, Allocations, Asset, Coins, Error, Store as AssetStore};
use crate::rgbkit::fungible::schema::{AssignmentsType, FieldType};
use crate::rgbkit::{Error as RgbStoreError, MagicNumber, Store as RgbStore};
use crate::{field, type_map};

pub struct Manager<'inner, E1, E2>
where
    E1: RgbStoreError,
    E2: AssetStoreError,
{
    rgb_storage: Arc<dyn RgbStore<Error = E1> + 'inner>,
    asset_storage: Arc<dyn AssetStore<Error = E2> + 'inner>,
}

pub enum ExchangableData {
    File(PathBuf),
    Bech32(String),
}

pub enum IssueStructure {
    SingleIssue,
    MultipleIssues {
        max_supply: Coins,
        reissue_control: SealDefinition,
    },
}

impl<'inner, E1, E2> Manager<'inner, E1, E2>
where
    E1: RgbStoreError,
    E2: AssetStoreError,
{
    pub fn new(
        rgb_storage: Arc<impl RgbStore<Error = E1> + 'inner>,
        asset_storage: Arc<impl AssetStore<Error = E2> + 'inner>,
    ) -> Result<Self, Error<E1, E2>> {
        debug!("Instantiating RGB asset manager ...");

        let storage = rgb_storage.clone();
        let me = Self {
            rgb_storage,
            asset_storage,
        };
        let schema = schema::schema();
        if !me.rgb_storage.has_schema(schema.schema_id())? {
            info!("RGB fungible assets schema file not found, creating one");
            storage.add_schema(&schema)?;
        }

        Ok(me)
    }

    pub fn issue(
        &self,
        network: bp::Network,
        ticker: String,
        name: String,
        description: Option<String>,
        issue_structure: IssueStructure,
        allocations: Allocations,
        prune_seals: Vec<SealDefinition>,
        dust_limit: Option<rgb::Amount>,
    ) -> Result<Asset, Error<E1, E2>> {
        let now = Utc::now().timestamp();
        let mut metadata = type_map! {
            FieldType::Ticker => field!(String, ticker),
            FieldType::Name => field!(String, name),
            FieldType::DustLimit => field!(U64, dust_limit.unwrap_or(0)),
            FieldType::Timestamp => field!(U32, now as u32)
        };
        if let Some(description) = description {
            metadata.insert(-FieldType::Description, field!(String, description));
        }

        let mut issued_supply = 0u64;

        // Zero-knowledge black magic :)
        let allocations = {
            use lnpbp::rand;
            use lnpbp::secp256k1zkp;
            use secp256k1zkp::*;

            let secp = secp256k1zkp::Secp256k1::with_caps(ContextFlag::Commit);

            let mut blinding_factors = vec![];
            let mut assignments = allocations
                .iter()
                .map(|(seal_definition, coins)| {
                    let blinding = amount::BlindingFactor::new(&secp, &mut rand::thread_rng());
                    blinding_factors.push(blinding.clone());
                    issued_supply += coins.sats();
                    (
                        seal_definition,
                        amount::Revealed {
                            amount: coins.sats(),
                            blinding,
                        },
                    )
                })
                .collect::<Vec<_>>();

            let mut blinding_correction = secp
                .blind_sum(vec![secp256k1zkp::key::ZERO_KEY], blinding_factors)
                .unwrap();
            blinding_correction.neg_assign(&secp)?;
            if let Some(assign) = assignments.last_mut() {
                let amount = assign.1.amount;
                let blinding = &mut assign.1.blinding;
                blinding.add_assign(&secp, &blinding_correction)?;
            }

            assignments
        };

        metadata.insert(-FieldType::IssuedSupply, field!(U64, issued_supply));

        let mut assignments = BTreeMap::new();
        assignments.insert(
            -AssignmentsType::Assets,
            AssignmentsVariant::Homomorphic(
                allocations
                    .into_iter()
                    .map(|assign| Assignment::Revealed {
                        seal_definition: assign.0.clone(),
                        assigned_state: assign.1,
                    })
                    .collect(),
            ),
        );

        let mut total_supply = issued_supply;
        if let IssueStructure::MultipleIssues {
            max_supply,
            reissue_control,
        } = issue_structure
        {
            metadata.insert(-FieldType::TotalSupply, field!(U64, max_supply.sats()));
            assignments.insert(
                -AssignmentsType::Issue,
                AssignmentsVariant::Void(bset![Assignment::Revealed {
                    seal_definition: reissue_control,
                    assigned_state: data::Void
                }]),
            );
            total_supply = max_supply.sats();
        }

        metadata.insert(-FieldType::TotalSupply, field!(U64, total_supply));

        assignments.insert(
            -AssignmentsType::Prune,
            AssignmentsVariant::Void(
                prune_seals
                    .into_iter()
                    .map(|seal_definition| Assignment::Revealed {
                        seal_definition,
                        assigned_state: data::Void,
                    })
                    .collect(),
            ),
        );

        let genesis = Genesis::with(
            schema::schema().schema_id(),
            network,
            metadata,
            assignments,
            vec![],
        );
        self.rgb_storage.add_genesis(&genesis)?;

        let asset = Asset::try_from(genesis)?;
        //self.asset_storage.add_asset(&asset);

        Ok(asset)
    }

    pub fn import(&self, data: ExchangableData) -> Result<MagicNumber, Error<E1, E2>> {
        unimplemented!()
    }
}
