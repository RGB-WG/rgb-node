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

use std::collections::{BTreeSet, VecDeque};

use bitcoin::hashes::Hash;
use bitcoin::OutPoint;
use bp::dbc::{Anchor, AnchorId};
use commit_verify::lnpbp4::{LeafNotKnown, MerkleBlock, MerkleProof, ProtocolId, UnrelatedProof};
use rgb::{
    seal, AssignmentVec, ConcealState, Consignment, ContractId, Disclosure, Extension, Genesis,
    MergeReveal, Node, NodeId, Schema, SchemaId, SealEndpoint, Stash, Transition,
};
use strict_encoding::LargeVec;
use wallet::onchain::ResolveTx;

use super::index::Index;
use super::storage::Store;
use super::Runtime;

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(doc_comments)]
pub enum Error {
    /// Storage error
    #[from(super::storage::DiskStorageError)]
    StorageError,

    /// Index error
    #[from(super::index::BTreeIndexError)]
    IndexError,

    /// To create consignment for a state transition (state extension)
    /// operation you have to provide anchor data
    AnchorParameterIsRequired,

    /// You can't create consignments for pure genesis data; just share plain
    /// genesis instead
    GenesisNode,

    /// Trying to import data related to an unknown contract {0}. Please import
    /// genesis for that contract first.
    UnknownContract(ContractId),

    /// Broken anchor which is not related to the contract and state transition.
    #[from(UnrelatedProof)]
    #[from(LeafNotKnown)]
    BrokenAnchor,
}

pub struct DumbIter<T>(std::marker::PhantomData<T>);
impl<T> Iterator for DumbIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> { unimplemented!() }
}

impl Stash for Runtime {
    type Error = Error;
    type SchemaIterator = DumbIter<Schema>;
    type GenesisIterator = DumbIter<Genesis>;
    type AnchorIterator = DumbIter<Anchor<MerkleBlock>>;
    type TransitionIterator = DumbIter<Transition>;
    type ExtensionIterator = DumbIter<Extension>;
    type NodeIdIterator = DumbIter<NodeId>;

    fn get_schema(&self, _schema_id: SchemaId) -> Result<Schema, Self::Error> { todo!() }

    fn get_genesis(&self, _contract_id: ContractId) -> Result<Genesis, Self::Error> { todo!() }

    fn get_transition(&self, _node_id: NodeId) -> Result<Transition, Self::Error> { todo!() }

    fn get_extension(&self, _node_id: NodeId) -> Result<Extension, Self::Error> { todo!() }

    fn get_anchor(&self, _anchor_id: AnchorId) -> Result<Anchor<MerkleBlock>, Self::Error> {
        todo!()
    }

    fn genesis_iter(&self) -> Self::GenesisIterator { todo!() }

    fn anchor_iter(&self) -> Self::AnchorIterator { todo!() }

    fn transition_iter(&self, _contract_id: ContractId) -> Self::TransitionIterator { todo!() }

    fn extension_iter(&self, _contract_id: ContractId) -> Self::ExtensionIterator { todo!() }

    fn consign(
        &self,
        contract_id: ContractId,
        node: &impl Node,
        anchor: Option<&Anchor<MerkleProof>>,
        endpoints: &BTreeSet<SealEndpoint>,
    ) -> Result<Consignment, Error> {
        debug!(
            "Preparing consignment for contract {} node {} for endpoints {:?}",
            contract_id,
            node.node_id(),
            endpoints
        );

        trace!("Looking up for genesis");
        let genesis = self.storage.genesis(&contract_id)?;

        trace!("Getting node matching node id");
        let mut state_transitions = LargeVec::from(vec![]);
        let mut state_extensions: LargeVec<Extension> = LargeVec::from(vec![]);
        if let Some(transition) = node.as_any().downcast_ref::<Transition>().clone() {
            let anchor = anchor.ok_or(Error::AnchorParameterIsRequired)?;
            state_transitions.push((anchor.clone(), transition.clone()));
        } else if let Some(extension) = node.as_any().downcast_ref::<Extension>().clone() {
            state_extensions.push(extension.clone());
        } else {
            Err(Error::GenesisNode)?;
        }

        trace!("Collecting other involved nodes");
        let mut sources = VecDeque::<NodeId>::new();
        sources.extend(node.parent_owned_rights().iter().map(|(id, _)| id));
        sources.extend(node.parent_public_rights().iter().map(|(id, _)| id));
        trace!("Node list for consignment: {:#?}", sources);
        while let Some(node_id) = sources.pop_front() {
            if node_id.into_inner() == genesis.contract_id().into_inner() {
                continue;
            }
            trace!(
                "Getting anchor id for node/protocol id {} from the index",
                &ProtocolId::from((*node_id).into_inner())
            );
            let anchor_id = self.indexer.anchor_id_by_transition_id(node_id)?;
            trace!("Retrieving anchor with id {}", anchor_id);
            let anchor = self.storage.anchor(&anchor_id)?;
            trace!("Anchor data: {:#?}", anchor);
            let concealed_anchor = anchor.into_merkle_proof(contract_id)?;

            trace!("Extending source data with the ancestors");
            // TODO #162: (new) Improve this logic
            match (
                self.storage.transition(&node_id),
                self.storage.extension(&node_id),
            ) {
                (Ok(mut transition), Err(_)) => {
                    transition.conceal_state();
                    state_transitions.push((concealed_anchor, transition.clone()));
                    sources.extend(transition.parent_owned_rights().keys());
                    sources.extend(transition.parent_public_rights().keys());
                }
                (Err(_), Ok(mut extension)) => {
                    extension.conceal_state();
                    state_extensions.push(extension.clone());
                    sources.extend(extension.parent_owned_rights().keys());
                    sources.extend(extension.parent_public_rights().keys());
                }
                _ => Err(Error::StorageError)?,
            }
        }

        let node_id = node.node_id();
        let endpoints = endpoints.iter().map(|op| (node_id, *op)).collect();
        Ok(Consignment::with(
            genesis,
            endpoints,
            state_transitions,
            state_extensions,
        ))
    }

    fn accept(
        &mut self,
        consignment: &Consignment,
        known_seals: &Vec<seal::Revealed>,
    ) -> Result<(), Error> {
        let contract_id = consignment.genesis.contract_id();

        // [PRIVACY]:
        // Update transition data with the revealed state information that we
        // kept since we did an invoice (and the sender did not know).
        let reveal_known_seals = |(_, assignments): (&u16, &mut AssignmentVec)| match assignments {
            AssignmentVec::Declarative(_) => {}
            AssignmentVec::DiscreteFiniteField(set) => {
                *set = set
                    .iter()
                    .map(|a| {
                        let mut a = a.clone();
                        a.reveal_seals(known_seals.iter());
                        a
                    })
                    .collect();
            }
            AssignmentVec::CustomData(set) => {
                *set = set
                    .iter()
                    .map(|a| {
                        let mut a = a.clone();
                        a.reveal_seals(known_seals.iter());
                        a
                    })
                    .collect();
            }
        };

        // [PRIVACY] [SECURITY]:
        // Update all data with the previously known revealed information in the
        // stash
        for (anchor, transition) in consignment.state_transitions.iter() {
            let mut transition = transition.clone();
            transition
                .owned_rights_mut()
                .iter_mut()
                .for_each(reveal_known_seals);
            let node_id = transition.node_id();
            if let Ok(other_transition) = self.storage.transition(&node_id) {
                transition = transition
                    .merge_reveal(other_transition)
                    .expect("Transition id or merge-revealed procedure is broken");
            }
            let mut anchor = anchor.to_merkle_block(contract_id, node_id.into())?;
            let anchor_id = anchor.anchor_id();
            if let Ok(other_anchor) = self.storage.anchor(&anchor_id) {
                anchor = anchor
                    .merge_reveal(other_anchor)
                    .expect("Anchor id or merge-revealed procedure is broken");
            }
            // Store the transition and the anchor data in the stash
            self.storage.add_anchor(&anchor)?;
            self.indexer.index_anchor(&anchor)?;
            self.storage.add_transition(&transition)?;
        }

        for extension in consignment.state_extensions.iter() {
            let mut extension = extension.clone();
            extension
                .owned_rights_mut()
                .iter_mut()
                .for_each(reveal_known_seals);
            if let Ok(other_extension) = self.storage.extension(&extension.node_id()) {
                extension = extension
                    .merge_reveal(other_extension)
                    .expect("Extension id or merge-revealed procedure is broken");
            }
            self.storage.add_extension(&extension)?;
        }

        Ok(())
    }

    fn enclose(&mut self, disclosure: &Disclosure) -> Result<(), Self::Error> {
        // Do a disclosure verification: check that we know contract_ids
        let contract_ids = disclosure
            .transitions()
            .values()
            .map(|(_, map)| map.keys())
            .flatten()
            .chain(disclosure.extensions().keys())
            .copied()
            .collect::<BTreeSet<_>>();
        for contract_id in contract_ids {
            let _ = self
                .storage
                .genesis(&contract_id)
                .map_err(|_| Error::UnknownContract(contract_id))?;
        }

        for anchor in disclosure.transitions().values().map(|(anchor, _)| anchor) {
            let mut anchor = anchor.clone();
            if let Ok(other_anchor) = self.storage.anchor(&anchor.anchor_id()) {
                anchor = anchor
                    .merge_reveal(other_anchor)
                    .expect("RGB commitment procedure is broken");
            }
            self.storage.add_anchor(&anchor)?;
            self.indexer.index_anchor(&anchor)?;
        }

        for transition in disclosure
            .transitions()
            .values()
            .map(|(_, map)| map.values())
            .flatten()
        {
            let mut transition: Transition = transition.clone();
            if let Ok(other_transition) = self.storage.transition(&transition.node_id()) {
                transition = transition
                    .merge_reveal(other_transition)
                    .expect("RGB commitment procedure is broken");
            }
            self.storage.add_transition(&transition)?;
        }

        for extension in disclosure.extensions().values().flatten() {
            let mut extension: Extension = extension.clone();
            if let Ok(other_extension) = self.storage.extension(&extension.node_id()) {
                extension = extension
                    .merge_reveal(other_extension)
                    .expect("RGB commitment procedure is broken");
            }
            self.storage.add_extension(&extension)?;
        }

        Ok(())
    }

    fn prune(
        &mut self,
        _tx_resolver: &mut impl ResolveTx,
        _ownership_resolver: impl Fn(OutPoint) -> bool,
    ) -> Result<usize, Self::Error> {
        todo!()
    }
}
