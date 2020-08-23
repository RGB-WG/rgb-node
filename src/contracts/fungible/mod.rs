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

mod config;
mod data;
mod processor;
mod runtime;

pub(self) mod cache;

pub use data::{
    schema, Asset, Coins, Invoice, Issue, Outcoincealed, Outcoins, Outpoint, OutpointDescriptor,
    SchemaError, Supply,
};

pub use config::{Config, Opts};
pub use runtime::{main_with_config, Runtime};

pub use cache::CacheError;
pub use processor::IssueStructure;
pub(self) use processor::Processor;
