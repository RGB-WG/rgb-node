// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use rgb::{validation, ConsignmentType, InmemConsignment, Node, Validator, Validity};

use super::Runtime;
use crate::{DaemonError, Db};

impl Runtime {
    pub(super) fn process_consignment<C: ConsignmentType>(
        &mut self,
        consignment: InmemConsignment<C>,
    ) -> Result<validation::Status, DaemonError> {
        let contract_id = consignment.contract_id();
        let id = consignment.id();

        info!("Registering consignment {} for contract {}", id, contract_id);

        debug!("Validating consignment {} for contract {}", id, contract_id);
        let status = Validator::validate(&consignment, &self.electrum);
        info!("Consignment validation result is {}", status.validity());
        if status.validity() != Validity::Valid {
            // We skip import only for invalid information
            debug!("Validation status report: {:?}", status);
            return Ok(status);
        }

        info!("Storing consignment {} into database", id);
        trace!("Schema: {:?}", schema);
        self.db.store(Db::SCHEMATA, consignment.schema.schema_id(), &consignment.schema)?;
        if let Some(root_schema) = &consignment.root_schema {
            trace!("Root schema: {:?}", root_schema);
            self.db.store(Db::SCHEMATA, root_schema.schema_id(), root_schema)?;
        }

        trace!("Genesis: {:?}", consignment.genesis);
        self.db.store_merge(Db::GENESIS, contract_id, consignment.genesis)?;

        for (anchor, bundle) in consignment.anchored_bundles {
            let bundle_id = bundle.bundle_id();
            debug!("Processing anchored bundle {} for txid {}", bundle_id, anchor.txid);
            trace!("Anchor: {:?}", anchor);
            trace!("Bundle: {:?}", bundle);
            let anchor = anchor
                .into_merkle_block(contract_id, bundle_id.into())
                .expect("broken anchor data");
            debug!("Restored anchor id is {}", anchor.anchor_id());
            trace!("Restored anchor: {:?}", anchor);
            self.db.store_merge_h(Db::ANCHORS, anchor.txid, anchor)?;
            let mut data =
                bundle.concealed_iter().map(|(id, set)| (*id, set.clone())).collect::<Vec<_>>();
            for (transition, inputs) in bundle.into_revealed_iter() {
                let node_id = transition.node_id();
                debug!("Processing state transition {}", node_id);
                trace!("State transition: {:?}", transition);
                data.push((node_id, inputs.clone()));
                self.db.store_merge(Db::TRANSITIONS, node_id, transition)?;
            }
            self.db.store(Db::BUNDLES, bundle_id, &data)?;
        }
        for extension in consignment.state_extensions {
            let node_id = extension.node_id();
            debug!("Processing state extension {}", node_id);
            trace!("State transition: {:?}", extension);
            self.db.store_merge(Db::EXTENSIONS, node_id, extension)?;
        }

        info!("Consignment processing complete for {}", id);
        Ok(status)
    }
}
