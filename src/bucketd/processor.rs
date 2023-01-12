// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::io::Write;

use bitcoin::{OutPoint, Txid};
use commit_verify::{lnpbp4, CommitConceal, TaggedHash};
use psbt::Psbt;
use rgb::psbt::RgbExt;
use rgb::schema::{OwnedRightType, TransitionType};
use rgb::seal::Revealed;
use rgb::{
    bundle, validation, Anchor, Assignment, AttachmentStrategy, BundleId, Consignment,
    ConsignmentType, ContractId, ContractState, ContractStateMap, Disclosure, Genesis,
    InmemConsignment, Node, NodeId, OwnedRights, PedersenStrategy, Schema, SchemaId, SealEndpoint,
    StateTransfer, Transition, TransitionBundle, TypedAssignments, Validator, Validity,
};
use rgb_rpc::{FinalizeTransfersRes, OutpointFilter, Reveal};
use storm::chunk::ChunkIdExt;
use storm::{ChunkId, Container, ContainerId};
use strict_encoding::StrictDecode;

use super::Runtime;
use crate::amplify::Wrapper;
use crate::db::{self, StoreRpcExt};
use crate::DaemonError;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum StashError {
    /// state for contract {0} is not known or absent in the database.
    StateAbsent(ContractId),

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

    /// node {0} is not related to any contract - or at least not present in node-to-contract
    /// index.
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    NodeContractAbsent(NodeId),

    /// anchor for txid {0} is absent
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    AnchorAbsent(Txid),

    /// bundle data for contract {0} txid {1} is absent
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    BundleAbsent(ContractId, Txid),

    /// disclosure for txid {0} is absent
    ///
    /// It may happen due to RGB Node bug, or indicate internal stash inconsistency and compromised
    /// stash data storage.
    DisclosureAbsent(Txid),

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
    Outsizedbundle,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum FinalizeError {
    /// the provided PSBT does not contain transition bundle for the contract.
    ContractBundleMissed,

    /// the provided PSBT has invalid proprietary key structure. Details: {0}
    #[from]
    Psbt(rgb::psbt::KeyError),

    #[display(inner)]
    #[from]
    Anchor(bp::dbc::anchor::Error),

    #[display(inner)]
    Conceal,
}

impl Runtime {
    /// Processes incoming transfer downloaded as a container locally
    pub(super) fn process_container(
        &mut self,
        container_id: ContainerId,
    ) -> Result<validation::Status, DaemonError> {
        // Assemble consignment
        // TODO: Make this procedure part of Storm Core (assembling data from a container)
        let container_chunk = self
            .store
            .retrieve_chunk(storm_rpc::DB_TABLE_CONTAINERS, container_id)?
            .ok_or(DaemonError::NoContainer(container_id))?;
        let container = Container::strict_deserialize(container_chunk)?;
        let data = Vec::with_capacity(container.header.size as usize);
        let mut writer = io::Cursor::new(data);
        for chunk_id in container.chunks {
            let chunk = self
                .store
                .retrieve_chunk(storm_rpc::DB_TABLE_CHUNKS, chunk_id)?
                .unwrap_or_else(|| panic!("Chunk {} is absent", chunk_id));
            writer.write_all(chunk.as_slice()).expect("memory writers do not error");
        }

        let consignment = StateTransfer::strict_deserialize(writer.into_inner())?;
        self.process_consignment(consignment, true, None)
    }

    pub(super) fn process_consignment<C: ConsignmentType>(
        &mut self,
        consignment: InmemConsignment<C>,
        force: bool,
        reveal: Option<Reveal>,
    ) -> Result<validation::Status, DaemonError> {
        let contract_id = consignment.contract_id();
        let id = consignment.id();

        info!("Registering consignment {} for contract {}", id, contract_id);

        let mut state =
            self.store.retrieve_sten(db::CONTRACTS, contract_id)?.unwrap_or_else(|| {
                debug!("Contract {} was previously unknown", contract_id);
                ContractState::with(
                    consignment.schema_id(),
                    consignment.root_schema_id(),
                    contract_id,
                    consignment.genesis(),
                )
            });
        trace!("Starting with contract state {:?}", state);

        debug!("Validating consignment {} for contract {}", id, contract_id);
        let status = Validator::validate(&consignment, &self.electrum);
        info!("Consignment validation result is {}", status.validity());

        match status.validity() {
            Validity::Valid => {
                info!("Consignment is fully valid");
            }
            Validity::ValidExceptEndpoints if force => {
                warn!("Forcing import of consignment with non-mined transactions");
            }
            Validity::UnresolvedTransactions | Validity::ValidExceptEndpoints => {
                error!("Some of consignment-related transactions were not found: {:?}", status);
                return Ok(status);
            }
            Validity::Invalid => {
                error!("Invalid consignment: {:?}", status);
                return Ok(status);
            }
        }

        info!("Storing consignment {} into database", id);
        trace!("Schema: {:?}", consignment.schema());
        self.store.store_sten(db::SCHEMATA, consignment.schema_id(), consignment.schema())?;
        if let Some(root_schema) = consignment.root_schema() {
            trace!("Root schema: {:?}", root_schema);
            self.store.store_sten(db::SCHEMATA, root_schema.schema_id(), root_schema)?;
        }

        if let Some(Reveal {
            blinding_factor,
            outpoint,
            close_method,
        }) = reveal
        {
            let reveal_outpoint = Revealed {
                method: close_method,
                blinding: blinding_factor,
                txid: Some(outpoint.txid),
                vout: outpoint.vout,
            };

            let concealed_seals = consignment
                .endpoints()
                .filter(|&&(_, seal)| reveal_outpoint.to_concealed_seal() == seal.commit_conceal())
                .clone();

            if concealed_seals.count() == 0 {
                error!(
                    "The provided outpoint and blinding factors does not match with outpoint from \
                     the consignment"
                );
                Err(DaemonError::Finalize(FinalizeError::Conceal))?
            }
        };

        let genesis = consignment.genesis();
        debug!("Indexing genesis");
        trace!("Genesis: {:?}", genesis);
        self.store.store_merge(db::GENESIS, contract_id, genesis.clone())?;
        for seal in genesis.revealed_seals().unwrap_or_default() {
            debug!("Adding outpoint for seal {}", seal);
            let index_id = ChunkId::with_fixed_fragments(
                seal.txid.expect("genesis with vout-based seal which passed schema validation"),
                seal.vout,
            );
            self.store.insert_into_set(db::OUTPOINTS, index_id, contract_id)?;
        }
        debug!("Storing contract self-reference");
        self.store.store_sten(db::NODE_CONTRACTS, contract_id, &contract_id)?;

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
            self.store.store_merge(db::ANCHORS, anchor.txid, anchor)?;
            let concealed: BTreeMap<NodeId, BTreeSet<u16>> =
                bundle.concealed_iter().map(|(id, set)| (*id, set.clone())).collect();
            let mut revealed: BTreeMap<Transition, BTreeSet<u16>> = bmap!();
            for (transition, inputs) in bundle.revealed_iter() {
                let node_id = transition.node_id();
                let transition_type = transition.transition_type();
                debug!("Processing state transition {}", node_id);

                // TODO: refactoring this and move to rgb-core
                let new_transition = match reveal {
                    Some(Reveal {
                        blinding_factor,
                        outpoint,
                        close_method,
                    }) => {
                        let reveal_outpoint = Revealed {
                            method: close_method,
                            blinding: blinding_factor,
                            txid: Some(outpoint.txid),
                            vout: outpoint.vout,
                        };

                        let mut owned_rights: BTreeMap<OwnedRightType, TypedAssignments> = bmap! {};
                        for (owned_type, assignments) in transition.owned_rights().iter() {
                            match assignments {
                                TypedAssignments::Value(outpoints) => {
                                    let mut assignment: Vec<Assignment<PedersenStrategy>> =
                                        empty!();

                                    for out in outpoints.clone() {
                                        if out.commit_conceal().to_confidential_seal()
                                            != reveal_outpoint.to_concealed_seal()
                                        {
                                            assignment.push(out);
                                        } else {
                                            let accept = match out.as_revealed_state() {
                                                Some(seal) => Assignment::Revealed {
                                                    seal: reveal_outpoint,
                                                    state: *seal,
                                                },
                                                _ => out,
                                            };
                                            assignment.push(accept);
                                        }
                                    }

                                    owned_rights
                                        .insert(*owned_type, TypedAssignments::Value(assignment));
                                }
                                TypedAssignments::Attachment(outpoints) => {
                                    let mut assignment: Vec<Assignment<AttachmentStrategy>> =
                                        empty!();

                                    for out in outpoints.clone() {
                                        if out.commit_conceal().to_confidential_seal()
                                            != reveal_outpoint.to_concealed_seal()
                                        {
                                            assignment.push(out);
                                        } else {
                                            let accept = match out.as_revealed_state() {
                                                Some(seal) => Assignment::Revealed {
                                                    seal: reveal_outpoint,
                                                    state: seal.clone(),
                                                },
                                                _ => out,
                                            };
                                            assignment.push(accept);
                                        }
                                    }

                                    owned_rights.insert(
                                        *owned_type,
                                        TypedAssignments::Attachment(assignment),
                                    );
                                }
                                _ => {}
                            }
                        }

                        let tmp: Transition = Transition::with(
                            transition.transition_type(),
                            transition.metadata().clone(),
                            transition.parent_public_rights().clone(),
                            OwnedRights::from(owned_rights),
                            transition.public_rights().clone(),
                            transition.parent_owned_rights().clone(),
                        );

                        tmp
                    }
                    _ => transition.to_owned(),
                };

                trace!("State transition: {:?}", new_transition);
                state.add_transition(witness_txid, &new_transition);
                trace!("Contract state now is {:?}", state);

                trace!("Storing state transition data");
                revealed.insert(new_transition.clone(), inputs.clone());
                self.store.store_merge(db::TRANSITIONS, node_id, new_transition.clone())?;
                self.store.store_sten(db::TRANSITION_WITNESS, node_id, &witness_txid)?;

                trace!("Indexing transition");
                let index_id = ChunkId::with_fixed_fragments(contract_id, transition_type);
                self.store.insert_into_set(
                    db::CONTRACT_TRANSITIONS,
                    index_id,
                    node_id.into_array(),
                )?;

                self.store.store_sten(db::NODE_CONTRACTS, node_id, &contract_id)?;

                for seal in new_transition.filter_revealed_seals() {
                    let index_id = ChunkId::with_fixed_fragments(
                        seal.txid.expect("seal should contain revealed txid"),
                        seal.vout,
                    );
                    self.store.insert_into_set(db::OUTPOINTS, index_id, node_id.into_array())?;
                }
            }
            let data = TransitionBundle::with(revealed, concealed)
                .expect("enough data should be available to create bundle");
            let chunk_id = ChunkId::with_fixed_fragments(contract_id, witness_txid);
            self.store.store_sten(db::BUNDLES, chunk_id, &data)?;
        }
        for extension in consignment.state_extensions() {
            let node_id = extension.node_id();
            debug!("Processing state extension {}", node_id);
            trace!("State transition: {:?}", extension);

            state.add_extension(extension);
            trace!("Contract state now is {:?}", state);

            self.store.store_sten(db::NODE_CONTRACTS, node_id, &contract_id)?;

            self.store.store_merge(db::EXTENSIONS, node_id, extension.clone())?;
            // We do not store seal outpoint here - or will have to store it into a separate
            // database Extension rights are always closed seals, since the extension
            // can get into the history only through closing by a state transition
        }

        debug!("Storing contract state for {}", contract_id);
        trace!("Final contract state is {:?}", state);
        self.store.store_sten(db::CONTRACTS, contract_id, &state)?;

        info!("Consignment processing complete for {}", id);
        Ok(status)
    }

    pub(super) fn process_disclosure(&mut self, txid: Txid) -> Result<(), DaemonError> {
        let disclosure: Disclosure = self
            .store
            .retrieve_sten(db::DISCLOSURES, txid)?
            .ok_or(StashError::DisclosureAbsent(txid))?;

        for (anchor, bundle_map) in disclosure.anchored_bundles().values() {
            for (contract_id, bundle) in bundle_map {
                let mut state: ContractState = self
                    .store
                    .retrieve_sten(db::CONTRACTS, *contract_id)?
                    .ok_or(StashError::StateAbsent(*contract_id))?;
                trace!("Starting with contract state {:?}", state);

                let bundle_id = bundle.bundle_id();
                let witness_txid = anchor.txid;
                debug!("Processing anchored bundle {} for txid {}", bundle_id, witness_txid);
                trace!("Anchor: {:?}", anchor);
                trace!("Bundle: {:?}", bundle);
                self.store.store_sten(db::ANCHORS, anchor.txid, anchor)?;
                let concealed: BTreeMap<NodeId, BTreeSet<u16>> =
                    bundle.concealed_iter().map(|(id, set)| (*id, set.clone())).collect();
                let mut revealed: BTreeMap<Transition, BTreeSet<u16>> = bmap!();
                for (transition, inputs) in bundle.revealed_iter() {
                    let node_id = transition.node_id();
                    let transition_type = transition.transition_type();
                    debug!("Processing state transition {}", node_id);
                    trace!("State transition: {:?}", transition);

                    state.add_transition(witness_txid, transition);
                    trace!("Contract state now is {:?}", state);

                    trace!("Storing state transition data");
                    revealed.insert(transition.clone(), inputs.clone());
                    self.store.store_merge(db::TRANSITIONS, node_id, transition.clone())?;
                    self.store.store_sten(db::TRANSITION_WITNESS, node_id, &witness_txid)?;

                    trace!("Indexing transition");
                    let index_id = ChunkId::with_fixed_fragments(*contract_id, transition_type);
                    self.store.insert_into_set(
                        db::CONTRACT_TRANSITIONS,
                        index_id,
                        node_id.into_array(),
                    )?;

                    self.store.store_sten(db::NODE_CONTRACTS, node_id, contract_id)?;

                    for seal in transition.filter_revealed_seals() {
                        let index_id = ChunkId::with_fixed_fragments(
                            seal.txid.expect("seal should contain revealed txid"),
                            seal.vout,
                        );
                        self.store.insert_into_set(
                            db::OUTPOINTS,
                            index_id,
                            node_id.into_array(),
                        )?;
                    }
                }
                let data = TransitionBundle::with(revealed, concealed)
                    .expect("enough data should be available to create bundle");
                let chunk_id = ChunkId::with_fixed_fragments(*contract_id, witness_txid);
                self.store.store_sten(db::BUNDLES, chunk_id, &data)?;

                self.store.store_sten(db::CONTRACTS, *contract_id, &state)?;
            }
        }

        Ok(())
    }

    pub(super) fn compose_consignment<T: ConsignmentType>(
        &mut self,
        contract_id: ContractId,
        always_include: BTreeSet<TransitionType>,
        outpoint_filter: OutpointFilter,
        _phantom: T,
    ) -> Result<InmemConsignment<T>, DaemonError> {
        let genesis: Genesis =
            self.store.retrieve_sten(db::GENESIS, contract_id)?.ok_or(StashError::GenesisAbsent)?;
        let schema_id = genesis.schema_id();
        let schema: Schema = self
            .store
            .retrieve_sten(db::SCHEMATA, schema_id)?
            .ok_or(StashError::SchemaAbsent(schema_id))?;
        let root_schema_id = schema.root_id;
        let root_schema = if root_schema_id != zero!() {
            Some(
                self.store
                    .retrieve_sten(db::SCHEMATA, root_schema_id)?
                    .ok_or(StashError::SchemaAbsent(root_schema_id))?,
            )
        } else {
            None
        };

        let mut collector = Collector::new(contract_id);
        let outpoints_all = OutpointFilter::All;
        for transition_type in schema.transitions.keys() {
            let chunk_id = ChunkId::with_fixed_fragments(contract_id, *transition_type);
            let node_ids: BTreeSet<NodeId> =
                self.store.retrieve_sten(db::CONTRACT_TRANSITIONS, chunk_id)?.unwrap_or_default();
            let filter = if always_include.contains(transition_type) {
                &outpoints_all
            } else {
                &outpoint_filter
            };
            collector.process(&mut self.store, node_ids, filter)?;
        }

        collector = collector.iterate(&mut self.store)?;

        collector.into_consignment(schema, root_schema, genesis)
    }

    pub(super) fn outpoint_state(
        &mut self,
        outpoints: BTreeSet<OutPoint>,
    ) -> Result<ContractStateMap, DaemonError> {
        let mut res: ContractStateMap = bmap! {};

        let indexes = if outpoints.is_empty() {
            self.store.ids(db::OUTPOINTS)?
        } else {
            outpoints
                .iter()
                .map(|outpoint| ChunkId::with_fixed_fragments(outpoint.txid, outpoint.vout))
                .collect()
        };

        for index in &indexes {
            let set: BTreeSet<NodeId> =
                self.store.retrieve_sten(db::OUTPOINTS, *index)?.unwrap_or_default();
            for node_id in set {
                let contract_id: ContractId = self
                    .store
                    .retrieve_sten(db::NODE_CONTRACTS, node_id)?
                    .ok_or(StashError::NodeContractAbsent(node_id))?;

                let state: ContractState = self
                    .store
                    .retrieve_sten(db::CONTRACTS, contract_id)?
                    .ok_or(StashError::StateAbsent(contract_id))?;

                let map = if outpoints.is_empty() {
                    state.all_outpoint_state()
                } else {
                    state.filter_outpoint_state(&outpoints)
                };

                res.insert(contract_id, map);
            }
        }

        Ok(res)
    }

    pub(super) fn finalize_transfers(
        &mut self,
        transfers: Vec<(StateTransfer, Vec<SealEndpoint>)>,
        mut psbt: Psbt,
    ) -> Result<FinalizeTransfersRes, DaemonError> {
        // 1. Pack LNPBP-4 and anchor information.
        let mut bundles = psbt.rgb_bundles()?;
        debug!("Found {} bundles", bundles.len());
        trace!("Bundles: {:?}", bundles);

        let anchor = Anchor::commit(&mut psbt)?;
        trace!("Anchor: {:?}", anchor);

        let mut consignments = vec![];
        for (state_transfer, seal_endpoints) in &transfers {
            let mut consignment = state_transfer.clone();
            let contract_id = consignment.contract_id();
            info!("Finalizing transfer for {}", contract_id);

            // 2. Extract contract-related state transition from PSBT and put it
            //    into consignment.
            let bundle = bundles.remove(&contract_id).ok_or(FinalizeError::ContractBundleMissed)?;
            let bundle_id = bundle.bundle_id();
            consignment.push_anchored_bundle(anchor.to_merkle_proof(contract_id)?, bundle)?;

            // 3. Add seal endpoints.
            let endseals = seal_endpoints.clone();
            for endseal in endseals {
                consignment.push_seal_endpoint(bundle_id, endseal);
            }

            consignments.push(consignment);
        }

        // 4. Conceal all the state not related to the transfer.
        // TODO: Conceal all the amounts except the last transition
        // TODO: Conceal all seals outside of the paths from the endpoint to genesis

        // 5. Construct and store disclosure for the blank transfers.
        let txid = anchor.txid;
        let disclosure = Disclosure::with(anchor, bundles, None);
        self.store.store_sten(db::DISCLOSURES, txid, &disclosure)?;

        Ok(FinalizeTransfersRes { consignments, psbt })
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
        store: &mut store_rpc::Client,
        node_ids: impl IntoIterator<Item = NodeId>,
        outpoint_filter: &OutpointFilter,
    ) -> Result<(), DaemonError> {
        let contract_id = self.contract_id;

        for transition_id in node_ids {
            if transition_id.as_inner() == contract_id.as_inner() {
                continue;
            }
            let transition: Transition = store
                .retrieve_sten(db::TRANSITIONS, transition_id)?
                .ok_or(StashError::TransitionAbsent(transition_id))?;

            let witness_txid: Txid = store
                .retrieve_sten(db::TRANSITION_WITNESS, transition_id)?
                .ok_or(StashError::TransitionTxidAbsent(transition_id))?;

            let bundle = if let Some((_, bundle)) = self.anchored_bundles.get_mut(&witness_txid) {
                bundle
            } else {
                let anchor: Anchor<lnpbp4::MerkleBlock> = store
                    .retrieve_sten(db::ANCHORS, witness_txid)?
                    .ok_or(StashError::AnchorAbsent(witness_txid))?;
                let chunk_id = ChunkId::with_fixed_fragments(contract_id, witness_txid);
                let bundle: TransitionBundle = store
                    .retrieve_sten(db::BUNDLES, chunk_id)?
                    .ok_or(StashError::BundleAbsent(contract_id, witness_txid))?;
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

    pub fn iterate(mut self, store: &mut store_rpc::Client) -> Result<Self, DaemonError> {
        // Collect all transitions between endpoints and genesis independently from their type
        loop {
            let node_ids = self.endpoint_inputs;
            self.endpoint_inputs = vec![];
            self.process(store, node_ids, &OutpointFilter::All)?;
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
            .map_err(|_| StashError::Outsizedbundle)?;

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
