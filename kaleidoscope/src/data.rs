// Kaleidoscope: RGB command-line wallet utility
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Alekos Filini <alekos.filini@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use bitcoin::OutPoint;
use lnpbp::bitcoin;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DepositTerminal {
    pub outpoint: OutPoint,
    pub derivation_index: usize,
    pub spending_structure: bitcoin::AddressType,
    pub bitcoins: bitcoin::Amount,
    //pub fungibles: HashMap<TransitionId, Vec<(AssetAmount, TransitionId)>>
}

/*
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AssetAllocations {
    pub seals: HashMap<ContractId, Vec<rgb::fungible::Allocation>>,
}

impl AssetAllocations {
    pub fn new() -> Self {
        Self { seals: HashMap::new() }
    }
}
*/
