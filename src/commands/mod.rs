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


pub mod account;
pub mod bitcoin;
pub mod lightning;
pub mod fungible;
pub mod collectible;
pub mod identity;


use clap::Clap;


#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Initializes data directory
    Init,

    /// Accounts-related operations
    Account(account::Command),

    /// Bitcoin operations
    Bitcoin(bitcoin::Command),

    /// Ligthning operations
    Lightning(lightning::Command),

    /// Operations with fungible assets (RGB-1 standard)
    Fungible(fungible::Command),

    /// Operations with non-funcible/collectible assets (RGB-2 standard)
    Collectible(collectible::Command),

    /// Operations with decentralized identities (RGB-3 standard)
    Identity(identity::Command),
}
