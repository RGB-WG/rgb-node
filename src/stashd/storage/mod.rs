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

mod store;

#[cfg(not(store_hammersbald))] // Default store
mod disk;
#[cfg(and(store_hammersbald, not(any(store_disk))))]
mod hammersbald;

pub(super) use error::Error;
pub(super) use store::Store;

#[cfg(not(store_hammersbald))] // Default store
pub(super) use disk::DiskStorage;
#[cfg(not(store_hammersbald))]
pub(super) use disk::DiskStorageConfig;

#[cfg(and(store_hammersbald, not(any(store_disk))))]
pub(super) use hammersbald::HammersbaldStorage;
