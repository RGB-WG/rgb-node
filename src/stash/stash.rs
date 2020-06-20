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

//use lnpbp::rgb::{Anchor, Consignment, ContractId, SealDefinition, Transition};

//use super::storage::Store;
use super::Runtime;

impl Runtime {
    /*
    pub fn consign(
        &self,
        seals: Vec<SealDefinition>,
        contract_id: ContractId,
        transition: Transition,
        anchor: Anchor,
    ) -> Result<Consignment, Error> {
        let genesis = self.storage.genesis(&contract_id)?;

        // TODO: Conceal all unnecessary data

        for ref seal in seals {
            let (transition, anchor) = self.indexer.pair_defining_seal(seal);
            transition.conceal_except(seal);
            let input_seals = self.indexer.inputs(transition);
        }

        Ok(Consignment {
            genesis,
            endpoints: seals,
            data: vec![],
        })
    }

     */
}
