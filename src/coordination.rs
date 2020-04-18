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


use lnpbp::bp::scripts::PubkeyScript;
use lnpbp::bitcoin::{Transaction, TxIn, TxOut, OutPoint};
use lnpbp::rgb::*;


pub trait OwnershipProvider {
    fn can_spend(&self, script: PubkeyScript) -> bool { unimplemented!() }
}

pub enum CoordinationError { }

pub struct CoordinatedTransition {
    pub transitions: Vec<Transition>,
    pub commitment: MultimessageCommitment,
    pub transaction: Transaction,
}

pub struct Coordinator {
    pub known_owned_contracts: Vec<Box<dyn KnownContract>>,
    pub ownership_provider: dyn OwnershipProvider,
}

impl Coordinator {
    pub fn coordinate_transition(&self,
                                 transition: Transition,
                                 aux_inputs: Vec<TxIn>,
                                 aux_outputs: Vec<TxOut>,
    ) -> Result<CoordinatedTransition, CoordinationError> { unimplemented!() }

    pub fn known_owned_utxos(&self) -> Vec<OutPoint> { unimplemented!() }
}

pub trait KnownContract {
    fn defined_seals(&self) -> Vec<UtxoSeal>;
}
