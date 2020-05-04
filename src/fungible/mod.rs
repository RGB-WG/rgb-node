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

pub mod invoice;
mod error;
mod data;
pub mod selection;
mod accountant;

pub use invoice::Invoice;
pub use error::Error;
pub use data::*;
pub use accountant::*;

pub type Amount = u64;

/*
// Wrong tx
"txinwitness": [
"30440220019d6ddc12d1ab3636a117f5b0165e6bee20c859dae08cdd0ed4d90b25e184d502202223776508442d52f99ac656d8efdc5318b528ee99123c1cfba6aa015c52652501",
"033250ab4e12a1dc6a9fd2aad2a834ab30dcae7ea7891867781a27f823d00c9689"
],

// Correct tx
"txinwitness": [
"30440220236fd43bd59cadbba37d37f0e739f2741f3759c91da2b7e2bbe248b112e9e30b02206f164b33cda7f1fd7e0d69e8d02e98cab2667589112d163208463e33f258678601",
"033250ab4e12a1dc6a9fd2aad2a834ab30dcae7ea7891867781a27f823d00c9689"
],
*/