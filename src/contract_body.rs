// RGB Rust Library
// Written in 2019 by
//     Dr. Maxim Orlovsky <dr.orlovsky@gmail.com>
// basing on ideas from the original RGB rust library by
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

use crate::*;

/// Trait to be used by custom contract blueprint implementation to provide its own custom fields.
pub trait ContractBody: Sized {
    /// Validates given proof to have a correct structure matching RGB contract blueprint.
    /// This is default implementation that checks nothing, all required functionality for specific
    /// blueprint type (like checking proof metadata/scripts) must be implemented by custom
    /// classes implementing `ContractBody` trait.
    fn validate_proof(&self, _: &Proof<Self>) -> Result<(), RgbError<Self>> {
        Ok(())
    }
}
