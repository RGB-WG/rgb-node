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
use lnpbp::seals::OutpointReveal;
use rgb::{
    Anchor, Assignments, ConcealState, Consignment, ContractId, Disclosure,
    Extension, Genesis, IntoRevealed, Node, NodeId, SchemaId, SealEndpoint,
    Stash, Transition,
};

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
}

pub struct DumbIter<T>(std::marker::PhantomData<T>);
impl<T> Iterator for DumbIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

impl Stash for Runtime {
    type Error = Error;
    type GenesisIterator = DumbIter<Genesis>;
    type AnchorIterator = DumbIter<Anchor>;
    type TransitionIterator = DumbIter<Transition>;
    type ExtensionIterator = DumbIter<Extension>;
    type NidIterator = DumbIter<NodeId>;

    fn get_schema(
        &self,
        _schema_id: SchemaId,
    ) -> Result<SchemaId, Self::Error> {
        unimplemented!()
    }

    fn get_genesis(
        &self,
        _contract_id: ContractId,
    ) -> Result<Genesis, Self::Error> {
        unimplemented!()
    }

    fn get_transition(
        &self,
        _node_id: NodeId,
    ) -> Result<Transition, Self::Error> {
        unimplemented!()
    }

    fn get_extension(
        &self,
        _node_id: NodeId,
    ) -> Result<Extension, Self::Error> {
        unimplemented!()
    }

    fn get_anchor(
        &self,
        _anchor_id: ContractId,
    ) -> Result<Anchor, Self::Error> {
        unimplemented!()
    }

    fn genesis_iter(&self) -> Self::GenesisIterator {
        unimplemented!()
    }

    fn anchor_iter(&self) -> Self::AnchorIterator {
        unimplemented!()
    }

    fn transition_iter(&self) -> Self::TransitionIterator {
        unimplemented!()
    }

    fn extension_iter(&self) -> Self::ExtensionIterator {
        unimplemented!()
    }

    fn consign(
        &self,
        contract_id: ContractId,
        node: &impl Node,
        anchor: Option<&Anchor>,
        endpoints: &BTreeSet<SealEndpoint>,
    ) -> Result<Consignment, Error> {
        let genesis = self.storage.genesis(&contract_id)?;

        let mut state_transitions = vec![];
        let mut state_extensions: Vec<Extension> = vec![];
        if let Some(transition) =
            node.as_any().downcast_ref::<Transition>().clone()
        {
            let anchor = anchor.ok_or(Error::AnchorParameterIsRequired)?;
            state_transitions.push((anchor.clone(), transition.clone()));
        } else if let Some(extension) =
            node.as_any().downcast_ref::<Extension>().clone()
        {
            state_extensions.push(extension.clone());
        } else {
            Err(Error::GenesisNode)?;
        }

        let mut sources = VecDeque::<NodeId>::new();
        sources
            .extend(node.parent_owned_rights().into_iter().map(|(id, _)| id));
        sources
            .extend(node.parent_public_rights().into_iter().map(|(id, _)| id));
        while let Some(node_id) = sources.pop_front() {
            if node_id.into_inner() == genesis.contract_id().into_inner() {
                continue;
            }
            let anchor_id = self.indexer.anchor_id_by_transition_id(node_id)?;
            let anchor = self.storage.anchor(&anchor_id)?;
            // TODO: (new) Improve this logic
            match (
                self.storage.transition(&node_id),
                self.storage.extension(&node_id),
            ) {
                (Ok(mut transition), Err(_)) => {
                    transition.conceal_state();
                    state_transitions.push((anchor, transition.clone()));
                    sources.extend(
                        transition
                            .parent_owned_rights()
                            .into_iter()
                            .map(|(id, _)| id),
                    );
                    sources.extend(
                        transition
                            .parent_public_rights()
                            .into_iter()
                            .map(|(id, _)| id),
                    );
                }
                (Err(_), Ok(mut extension)) => {
                    extension.conceal_state();
                    state_extensions.push(extension.clone());
                    sources.extend(
                        extension
                            .parent_owned_rights()
                            .into_iter()
                            .map(|(id, _)| id),
                    );
                    sources.extend(
                        extension
                            .parent_public_rights()
                            .into_iter()
                            .map(|(id, _)| id),
                    );
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
        known_seals: &Vec<OutpointReveal>,
    ) -> Result<(), Error> {
        let consignment = consignment.clone();

        // [PRIVACY]:
        // Update transition data with the revealed state information that we
        // kept since we did an invoice (and the sender did not know).
        let reveal_known_seals =
            |(_, assignments): (&usize, &mut Assignments)| match assignments {
                Assignments::Declarative(_) => {}
                Assignments::DiscreteFiniteField(set) => {
                    *set = set
                        .iter()
                        .map(|a| {
                            let mut a = a.clone();
                            a.reveal_seals(known_seals.iter());
                            a
                        })
                        .collect();
                }
                Assignments::CustomData(set) => {
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
        for (mut anchor, mut transition) in
            consignment.state_transitions.into_iter()
        {
            transition
                .owned_rights_mut()
                .into_iter()
                .for_each(reveal_known_seals);
            if let Ok(other_transition) =
                self.storage.transition(&transition.node_id())
            {
                transition = transition.into_revealed(other_transition).expect(
                    "Transition id or merge-revealed procedure is broken",
                );
            }
            if let Ok(other_anchor) = self.storage.anchor(&anchor.anchor_id()) {
                anchor = anchor
                    .into_revealed(other_anchor)
                    .expect("Anchor id or merge-revealed procedure is broken");
            }
            // Store the transition and the anchor data in the stash
            self.storage.add_anchor(&anchor)?;
            // TODO: Uncomment once indexing will be implemented
            // self.indexer.index_anchor(&anchor)?;
            self.storage.add_transition(&transition)?;
        }

        for mut extension in consignment.state_extensions.into_iter() {
            extension
                .owned_rights_mut()
                .into_iter()
                .for_each(reveal_known_seals);
            if let Ok(other_extension) =
                self.storage.extension(&extension.node_id())
            {
                extension = extension.into_revealed(other_extension).expect(
                    "Extension id or merge-revealed procedure is broken",
                );
            }
            self.storage.add_extension(&extension)?;
        }

        Ok(())
    }

    // TODO: Rename into `enclose`
    fn know_about(
        &mut self,
        disclosure: Disclosure,
    ) -> Result<(), Self::Error> {
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

        for anchor in
            disclosure.transitions().values().map(|(anchor, _)| anchor)
        {
            let mut anchor: Anchor = anchor.clone();
            if let Ok(other_anchor) = self.storage.anchor(&anchor.anchor_id()) {
                anchor = anchor
                    .into_revealed(other_anchor)
                    .expect("RGB commitment procedure is broken");
            }
            self.storage.add_anchor(&anchor)?;
        }

        for transition in disclosure
            .transitions()
            .values()
            .map(|(_, map)| map.values())
            .flatten()
        {
            let mut transition: Transition = transition.clone();
            if let Ok(other_transition) =
                self.storage.transition(&transition.node_id())
            {
                transition = transition
                    .into_revealed(other_transition)
                    .expect("RGB commitment procedure is broken");
            }
            self.storage.add_transition(&transition)?;
        }

        for extension in disclosure.extensions().values().flatten() {
            let mut extension: Extension = extension.clone();
            if let Ok(other_extension) =
                self.storage.extension(&extension.node_id())
            {
                extension = extension
                    .into_revealed(other_extension)
                    .expect("RGB commitment procedure is broken");
            }
            self.storage.add_extension(&extension)?;
        }

        Ok(())
    }

    fn forget(
        &mut self,
        _consignment: Consignment,
    ) -> Result<usize, Self::Error> {
        unimplemented!()
    }

    fn prune(&mut self) -> Result<usize, Self::Error> {
        unimplemented!()
    }
}
