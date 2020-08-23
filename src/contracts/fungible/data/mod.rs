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

mod asset;
mod invoice;
mod outcoins;
pub mod schema;

pub use asset::{Asset, Coins, Issue, Supply};
pub use invoice::{Invoice, Outpoint, OutpointDescriptor};
pub use outcoins::{Outcoincealed, Outcoins};
pub use schema::SchemaError;
