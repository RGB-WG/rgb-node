// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use bitcoin::{OutPoint, Txid};
use commit_verify::lnpbp4;
use psbt::Psbt;
use rgb::schema::TransitionType;
use rgb::{
    bundle, validation, Anchor, BundleId, Consignment, ConsignmentType, ContractId, ContractState,
    Genesis, InmemConsignment, Node, NodeId, Schema, SchemaId, SealEndpoint, StateTransfer,
    Transition, TransitionBundle, Validator, Validity,
};
use rgb_rpc::OutpointFilter;

use super::Runtime;
use crate::{DaemonError, Db};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum StashError {
    /// contract is unknown. Probably you haven't imported the contract yet.
    GenesisAbsent,

    /// schema {0} is unknown.
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    SchemaAbsent(SchemaId),

    /// transition {0} is absent.
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    TransitionAbsent(NodeId),

    /// witness Txid is not known for transition {0}
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    TransitionTxidAbsent(NodeId),

    /// anchor for txid {0} is absent
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    AnchorAbsent(Txid),

    /// bundle data for txid {0} is absent
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    BundleAbsent(Txid),

    /// the anchor is not related to the contract
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    #[from(lnpbp4::LeafNotKnown)]
    UnrelatedAnchor,

    /// bundle reveal error
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    #[from]
    #[display(inner)]
    BundleReveal(bundle::RevealError),

    /// the resulting bundle size exceeds consensus restrictions
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    OutsizedBundle,
}

impl Runtime {
    pub(super) fn process_consignment<C: ConsignmentType>(
        &mut self,
        consignment: InmemConsignment<C>,
        force: bool,
    ) -> Result<validation::Status, DaemonError> {
        let contract_id = consignment.contract_id();
        let id = consignment.id();

        info!("Registering consignment {} for contract {}", id, contract_id);

        let mut state = self.db.retrieve(Db::CONTRACTS, contract_id)?.unwrap_or_else(|| {
            debug!("Contract {} was previously unknown", contract_id);
            ContractState::with(contract_id, consignment.genesis())
        });
        trace!("Starting with contract state {:?}", state);

        debug!("Validating consignment {} for contract {}", id, contract_id);
        let status = Validator::validate(&consignment, &self.electrum);
        info!("Consignment validation result is {}", status.validity());

        match status.validity() {
            Validity::Valid => {}
            Validity::UnresolvedTransactions if force => {
                warn!("Forcing import of consignment with non-mined transactions");
            }
            _ => {
                error!("Invalid consignment: {:?}", status);
                return Ok(status);
            }
        }

        info!("Storing consignment {} into database", id);
        trace!("Schema: {:?}", consignment.schema());
        self.db.store(Db::SCHEMATA, consignment.schema_id(), consignment.schema())?;
        if let Some(root_schema) = consignment.root_schema() {
            trace!("Root schema: {:?}", root_schema);
            self.db.store(Db::SCHEMATA, root_schema.schema_id(), root_schema)?;
        }

        trace!("Genesis: {:?}", consignment.genesis());
        self.db.store_merge(Db::GENESIS, contract_id, consignment.genesis().clone())?;

        for (anchor, bundle) in consignment.anchored_bundles() {
            let bundle_id = bundle.bundle_id();
            let witness_txid = anchor.txid;
            debug!("Processing anchored bundle {} for txid {}", bundle_id, witness_txid);
            trace!("Anchor: {:?}", anchor);
            trace!("Bundle: {:?}", bundle);
            let anchor =
                anchor.to_merkle_block(contract_id, bundle_id.into()).expect("broken anchor data");
            debug!("Restored anchor id is {}", anchor.anchor_id());
            trace!("Restored anchor: {:?}", anchor);
            self.db.store_merge_h(Db::ANCHORS, anchor.txid, anchor)?;
            let mut data =
                bundle.concealed_iter().map(|(id, set)| (*id, set.clone())).collect::<Vec<_>>();
            for (transition, inputs) in bundle.revealed_iter() {
                let node_id = transition.node_id();
                let transition_type = transition.transition_type();
                debug!("Processing state transition {}", node_id);
                trace!("State transition: {:?}", transition);

                // TODO: For owned state, use only state which is a part of state tips
                state.add_transition(witness_txid, transition);
                trace!("Contract state now is {:?}", state);

                trace!("Storing state transition data");
                data.push((node_id, inputs.clone()));
                self.db.store_merge(Db::TRANSITIONS, node_id, transition.clone())?;
                self.db.store(Db::TRANSITION_WITNESS, node_id, &witness_txid)?;

                trace!("Indexing transition");
                let index_id = Db::index_two_pieces(contract_id, transition_type);
                self.db.insert_into_set(Db::CONTRACT_TRANSITIONS, index_id, node_id)?;

                // TODO: Add outpoint-to-node-id index
                // TODO: Add node-to-contract index
            }
            self.db.store_h(Db::BUNDLES, witness_txid, &data)?;
        }
        for extension in consignment.state_extensions() {
            let node_id = extension.node_id();
            debug!("Processing state extension {}", node_id);
            trace!("State transition: {:?}", extension);

            // TODO: For owned state, use only state which is a part of state tips
            state.add_extension(&extension);
            trace!("Contract state now is {:?}", state);

            self.db.store_merge(Db::EXTENSIONS, node_id, extension.clone())?;
        }

        debug!("Storing contract state for {}", contract_id);
        trace!("Final contract state is {:?}", state);
        self.db.store(Db::CONTRACTS, contract_id, &state)?;

        info!("Consignment processing complete for {}", id);
        Ok(status)
    }

    pub(super) fn compose_consignment<T: ConsignmentType>(
        &mut self,
        contract_id: ContractId,
        always_include: BTreeSet<TransitionType>,
        outpoint_filter: OutpointFilter,
        _phantom: T,
    ) -> Result<InmemConsignment<T>, DaemonError> {
        let genesis: Genesis =
            self.db.retrieve(Db::GENESIS, contract_id)?.ok_or(StashError::GenesisAbsent)?;
        let schema_id = genesis.schema_id();
        let schema: Schema = self
            .db
            .retrieve(Db::SCHEMATA, schema_id)?
            .ok_or(StashError::SchemaAbsent(schema_id))?;
        let root_schema_id = schema.root_id;
        let root_schema = if root_schema_id != zero!() {
            Some(
                self.db
                    .retrieve(Db::SCHEMATA, root_schema_id)?
                    .ok_or(StashError::SchemaAbsent(root_schema_id))?,
            )
        } else {
            None
        };

        let mut collector = Collector::new(contract_id);
        let outpoints_all = OutpointFilter::All;
        for transition_type in schema.transitions.keys() {
            let node_ids = self.db.transitions_by_type(contract_id, *transition_type)?;
            let filter = if always_include.contains(transition_type) {
                &outpoints_all
            } else {
                &outpoint_filter
            };
            collector.process(&mut self.db, node_ids, filter)?;
        }

        collector = collector.iterate(&mut self.db)?;

        collector.into_consignment(schema, root_schema, genesis)
    }

    pub(super) fn finalize_psbt(
        &mut self,
        transition: Transition,
        mut psbt: Psbt,
    ) -> Result<Psbt, DaemonError> {
        todo!();
        // 1. Put transition into PSBT (with LNPBP4s)
        // 2. Construct blank transitions for other contracts
        // 3. Put blank transitions into PSBT (with LNPBP4s)

        psbt.push_rgb_transition(transition)?;

        let outpoint_node_map: BTreeMap<OutPoint, BTreeSet<NodeId>> = psbt
            .inputs
            .iter()
            .map(|input| input.previous_outpoint)
            .map(|outpoint| {
                self.db
                    .retrieve(Db::OUTPOINTS, outpoint)
                    .filter_map(|set: Option<BTreeSet<NodeId>>| set.map(|nodes| (outpoint, nodes)))
            })
            .collect::<Result<_, DaemonError>>()?;

        let mut node_contracts: BTreeMap<NodeId, ContractId> = bmap! {};
        let mut prev_nodes: BTreeMap<NodeId, Box<dyn Node>> = bmap! {};
        let mut blank_transitions: BTreeMap<ContractId, BTreeSet<Transition>> = bmap! {};
        for input in &mut psbt.inputs {
            let outpoint = input.previous_outpoint;
            let set: BTreeSet<NodeId> =
                self.db.retrieve(Db::OUTPOINTS, outpoint)?.unwrap_or_default();
            for node_id in set {
                let contract_id = match node_contracts.entry(node_id) {
                    Entry::Vacant(entry) => {
                        let contract_id = self
                            .db
                            .retrieve(Db::NODE_CONTRACTS, node_id)?
                            .ok_or(StashError::UnknownNodeContract(contract_id, node_id))?;
                        entry.insert(contract_id);
                        contract_id
                    }
                    Entry::Occupied(entry) => *entry.get(),
                };
                input.set_rgb_consumer(contract_id, node_id)?;
                let node = match prev_nodes.entry(node_id) {
                    Entry::Vacant(entry) => {
                        let node = if let Some(transition) =
                            self.db.retrieve(Db::TRANSITIONS, node_id)?
                        {
                            Box::new(transition)
                        } else if let Some(genesis) = self.db.retrieve(Db::TRANSITIONS, node_id)? {
                            Box::new(genesis)
                        } else if let Some(extension) = self.db.retrieve(Db::EXTENSIONS, node_id)? {
                            Box::new(extension)
                        } else {
                            return Err(StashError::TransitionAbsent(node_id))
                                .map_err(DaemonError::from);
                        };
                        prev_nodes.insert(node_id, node.clone());
                        node
                    }
                    Entry::Occupied(entry) => entry.get(),
                };
            }
        }

        let contract_nodes: BTreeMap<ContractId, BTreeSet<NodeId>> = outpoint_node_map
            .values()
            .flatten()
            .map(|node_id| {
                self.db
                    .retrieve(Db::NODE_CONTRACTS, node_id)
                    .map(|contract_id| (contract_id, node_id))
            })
            .collect::<Result<_, DaemonError>>()?;

        for (contract_id, nodes) in contract_nodes {}

        Ok(psbt)
    }

    pub(super) fn finalize_transfer(
        &mut self,
        consignment: StateTransfer,
        psbt: Psbt,
    ) -> Result<StateTransfer, DaemonError> {
        // 1. Extract contract-related state transition from PSBT and put it
        //    into consignment.
        // 2. Conceal all the state not related to the transfer
        todo!()
    }
}

struct Collector {
    pub contract_id: ContractId,
    pub anchored_bundles: BTreeMap<Txid, (Anchor<lnpbp4::MerkleProof>, TransitionBundle)>,
    pub endpoints: Vec<(BundleId, SealEndpoint)>,
    pub endpoint_inputs: Vec<NodeId>,
}

impl Collector {
    pub fn new(contract_id: ContractId) -> Self {
        Collector {
            contract_id,
            anchored_bundles: empty![],
            endpoints: vec![],
            endpoint_inputs: vec![],
        }
    }

    // TODO: Support state extensions
    pub fn process(
        &mut self,
        db: &mut Db,
        node_ids: impl IntoIterator<Item = NodeId>,
        outpoint_filter: &OutpointFilter,
    ) -> Result<(), DaemonError> {
        let contract_id = self.contract_id;

        for transition_id in node_ids {
            let transition: Transition = db
                .retrieve(Db::TRANSITIONS, transition_id)?
                .ok_or(StashError::TransitionAbsent(transition_id))?;

            let witness_txid: Txid = db
                .retrieve(Db::TRANSITION_WITNESS, transition_id)?
                .ok_or(StashError::TransitionTxidAbsent(transition_id))?;

            let bundle = if let Some((_, bundle)) = self.anchored_bundles.get_mut(&witness_txid) {
                bundle
            } else {
                let anchor: Anchor<lnpbp4::MerkleBlock> = db
                    .retrieve_h(Db::ANCHORS, witness_txid)?
                    .ok_or(StashError::AnchorAbsent(witness_txid))?;
                let bundle: TransitionBundle = db
                    .retrieve_h(Db::BUNDLES, witness_txid)?
                    .ok_or(StashError::BundleAbsent(witness_txid))?;
                let anchor = anchor.to_merkle_proof(contract_id)?;
                self.anchored_bundles.insert(witness_txid, (anchor, bundle));
                &mut self.anchored_bundles.get_mut(&witness_txid).expect("stdlib is broken").1
            };

            let bundle_id = bundle.bundle_id();
            for (_, assignments) in transition.owned_rights().iter() {
                for seal in assignments.filter_revealed_seals() {
                    let txid = seal.txid.unwrap_or(witness_txid);
                    let outpoint = OutPoint::new(txid, seal.vout);
                    let seal_endpoint = SealEndpoint::from(seal);
                    if outpoint_filter.includes(outpoint) {
                        self.endpoints.push((bundle_id, seal_endpoint));
                        self.endpoint_inputs
                            .extend(transition.parent_outputs().into_iter().map(|out| out.node_id));
                    }
                }
            }

            bundle.reveal_transition(transition)?;
        }

        Ok(())
    }

    pub fn iterate(mut self, db: &mut Db) -> Result<Self, DaemonError> {
        // Collect all transitions between endpoints and genesis independently from their type
        loop {
            let node_ids = self.endpoint_inputs;
            self.endpoint_inputs = vec![];
            self.process(db, node_ids, &OutpointFilter::All)?;
            if self.endpoint_inputs.is_empty() {
                break;
            }
        }
        Ok(self)
    }

    pub fn into_consignment<T: ConsignmentType>(
        self,
        schema: Schema,
        root_schema: Option<Schema>,
        genesis: Genesis,
    ) -> Result<InmemConsignment<T>, DaemonError> {
        let anchored_bundles = self
            .anchored_bundles
            .into_values()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| StashError::OutsizedBundle)?;

        Ok(InmemConsignment::<T>::with(
            schema,
            root_schema,
            genesis,
            self.endpoints,
            anchored_bundles,
            empty!(),
        ))
    }
}
