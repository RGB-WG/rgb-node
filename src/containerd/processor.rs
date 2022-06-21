// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use rgb::{ConsignmentType, InmemConsignment, Node};

use super::Runtime;
use crate::{DaemonError, Db};

impl Runtime {
    fn process_consignment<C: ConsignmentType>(
        &mut self,
        consignment: InmemConsignment<C>,
    ) -> Result<(), DaemonError> {
        let contract_id = consignment.contract_id();

        info!("Registering consignment for contract {}", contract_id);

        // TODO: Validate consignment

        self.db.store(Db::SCHEMATA, consignment.schema.schema_id(), &consignment.schema)?;
        if let Some(root_schema) = &consignment.root_schema {
            self.db.store(Db::SCHEMATA, root_schema.schema_id(), root_schema)?;
        }

        self.db.store_merge(Db::GENESIS, contract_id, consignment.genesis)?;

        for (anchor, bundle) in consignment.anchored_bundles {
            let bundle_id = bundle.bundle_id();
            let anchor = anchor
                .into_merkle_block(contract_id, bundle_id.into())
                .expect("broken anchor data");
            self.db.store_merge_h(Db::ANCHORS, anchor.txid, anchor)?;
            let mut data =
                bundle.concealed_iter().map(|(id, set)| (*id, set.clone())).collect::<Vec<_>>();
            for (transition, inputs) in bundle.into_revealed_iter() {
                data.push((transition.node_id(), inputs.clone()));
                self.db.store_merge(Db::TRANSITIONS, transition.node_id(), transition)?;
            }
            self.db.store(Db::BUNDLES, bundle_id, &data)?;
        }
        for extension in consignment.state_extensions {
            self.db.store_merge(Db::EXTENSIONS, extension.node_id(), extension)?;
        }

        Ok(())
    }
}
