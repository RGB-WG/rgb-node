use chrono::Utc;
use core::convert::TryFrom;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use lnpbp::bp;
use lnpbp::rgb::prelude::*;

use super::schema::{self, AssignmentsType, FieldType};
use super::{Asset, Coins, Outcoins, Store as AssetStore};

use crate::rgbkit::{InteroperableError, MagicNumber, SealSpec, Store as RgbStore};
use crate::{field, type_map};

pub struct Manager<'inner> {
    rgb_storage: Arc<dyn RgbStore + 'inner>,
    asset_storage: Arc<dyn AssetStore + 'inner>,
}

pub enum ExchangableData {
    File(PathBuf),
    Bech32(String),
}

pub enum IssueStructure {
    SingleIssue,
    MultipleIssues {
        max_supply: f32,
        reissue_control: SealSpec,
    },
}

impl<'inner> Manager<'inner> {
    pub fn new(
        rgb_storage: Arc<impl RgbStore + 'inner>,
        asset_storage: Arc<impl AssetStore + 'inner>,
    ) -> Result<Self, InteroperableError> {
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
        allocations: Vec<Outcoins>,
        precision: u8,
        prune_seals: Vec<SealSpec>,
        dust_limit: Option<Amount>,
    ) -> Result<(Asset, Genesis), InteroperableError> {
        let now = Utc::now().timestamp();
        let mut metadata = type_map! {
            FieldType::Ticker => field!(String, ticker),
            FieldType::Name => field!(String, name),
            FieldType::FractionalBits => field!(U8, precision),
            FieldType::DustLimit => field!(U64, dust_limit.unwrap_or(0)),
            FieldType::Timestamp => field!(U32, now as u32)
        };
        if let Some(description) = description {
            metadata.insert(-FieldType::Description, field!(String, description));
        }

        let mut issued_supply = 0u64;
        let allocations = allocations
            .into_iter()
            .map(|outcoins| {
                let amount = Coins::transmutate(outcoins.coins, precision);
                issued_supply += amount;
                (outcoins.seal_definition(), amount)
            })
            .collect();
        let mut assignments = BTreeMap::new();
        assignments.insert(
            -AssignmentsType::Assets,
            AssignmentsVariant::zero_balanced(allocations),
        );
        metadata.insert(-FieldType::IssuedSupply, field!(U64, issued_supply));

        let mut total_supply = issued_supply;
        if let IssueStructure::MultipleIssues {
            max_supply,
            reissue_control,
        } = issue_structure
        {
            total_supply = Coins::transmutate(max_supply, precision);
            if total_supply < issued_supply {
                Err(InteroperableError(format!(
                    "Total supply ({}) should be greater than the issued supply ({})",
                    total_supply, issued_supply
                )))?;
            }
            metadata.insert(-FieldType::TotalSupply, field!(U64, total_supply));
            assignments.insert(
                -AssignmentsType::Issue,
                AssignmentsVariant::Void(bset![Assignment::Revealed {
                    seal_definition: reissue_control.seal_definition(),
                    assigned_state: data::Void
                }]),
            );
        } else {
            metadata.insert(-FieldType::TotalSupply, field!(U64, total_supply));
        }

        assignments.insert(
            -AssignmentsType::Prune,
            AssignmentsVariant::Void(
                prune_seals
                    .into_iter()
                    .map(|seal_spec| Assignment::Revealed {
                        seal_definition: seal_spec.seal_definition(),
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

        let asset = Asset::try_from(genesis.clone())?;
        //self.asset_storage.add_asset(&asset);

        Ok((asset, genesis))
    }

    pub fn import(&self, data: ExchangableData) -> Result<MagicNumber, InteroperableError> {
        unimplemented!()
    }
}
