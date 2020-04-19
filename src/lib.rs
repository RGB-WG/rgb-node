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


#[macro_use]
extern crate derive_wrapper;
extern crate chrono;
extern crate regex;
extern crate lightning_invoice;
extern crate lnpbp;


pub mod fungible;
pub mod collectible;
pub mod identity;

pub mod marker;
mod coordination;

pub use coordination::*;


pub mod managers {
    pub struct KeyringManager {}

    pub trait KeyringStore {}
}
