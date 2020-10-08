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

use std::collections::VecDeque;

use lnpbp::bitcoin::hashes::Hash;
use lnpbp::bp::blind::OutpointHash;
use lnpbp::rgb::{Anchor, AutoConceal, Consignment, ContractId, Node, NodeId, Stash, Transition};

use super::index::Index;
use super::storage::Store;
use super::Runtime;

#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error)]
#[display(Debug)]
pub enum Error {
    #[from(super::storage::DiskStorageError)]
    StorageError,

    #[from(super::index::BTreeIndexError)]
    IndexError,
}

// TODO: Move business logic to LNP/BP Core Library
impl Stash for Runtime {
    fn consign(
        &self,
        contract_id: &ContractId,
        transition: &Transition,
        anchor: &Anchor,
        endpoints: Vec<OutpointHash>,
    ) -> Result<Consignment, Error> {
        let genesis = self.storage.genesis(&contract_id)?;

        let mut transition = transition.clone();
        transition.conceal_except(&endpoints);
        let mut data = vec![(anchor.clone(), transition.clone())];
        let mut sources = VecDeque::<NodeId>::new();
        sources.extend(transition.ancestors().into_iter().map(|(id, _)| id));
        while let Some(tsid) = sources.pop_front() {
            if tsid.into_inner() == genesis.contract_id().into_inner() {
                continue;
            }
            let anchor_id = self.indexer.anchor_id_by_transition_id(tsid)?;
            let anchor = self.storage.anchor(&anchor_id)?;
            let mut transition = self.storage.transition(&tsid)?;
            transition.conceal_all();
            data.push((anchor, transition.clone()));
            sources.extend(transition.ancestors().into_iter().map(|(id, _)| id));
        }

        let node_id = transition.node_id();
        let extended_endpoints = endpoints.iter().map(|op| (node_id, *op)).collect();
        Ok(Consignment::with(genesis, extended_endpoints, data))
    }

    fn merge(&mut self, consignment: Consignment) -> Result<Vec<Box<dyn Node>>, Error> {
        let mut nodes: Vec<Box<dyn Node>> = vec![];
        consignment
            .data
            .into_iter()
            .try_for_each(|(anchor, transition)| -> Result<(), Error> {
                self.storage
                    .add_transition(&transition)?
                    .then(|| nodes.push(Box::new(transition)));
                self.storage.add_anchor(&anchor)?;
                self.indexer.index_anchor(&anchor)?;
                Ok(())
            })?;
        let genesis = consignment.genesis;
        self.storage
            .add_genesis(&genesis)?
            .then(|| nodes.push(Box::new(genesis)));

        Ok(nodes)
    }
}
