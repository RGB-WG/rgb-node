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


pub mod constants;
pub mod asset;
pub mod contract;
pub mod proof;
pub mod outputs;
pub mod util;

use crate::constants::*;
use crate::asset::Asset;
use crate::contract::{Contract, CommitmentScheme, BlueprintType};
use crate::proof::Proof;
use crate::outputs::RgbOutEntry;
use crate::outputs::RgbOutPoint;
use crate::util::*;
