// RGB standard library
// Written in 2019-2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

#[macro_use]
mod macros;
mod bech32data;
pub mod file;
mod magic_numbers;
// TODO: Consider deleting mod
// mod seal_spec;

pub use bech32data::{FromBech32Data, ToBech32Data};
pub use magic_numbers::MagicNumber;
// pub use seal_spec::SealSpec;
