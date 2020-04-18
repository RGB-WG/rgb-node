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


use std::path::PathBuf;
use clap::Clap;
use bitcoin::util::bip32::ExtendedPubKey;

use crate::constants::*;


#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Creates a new wallet and stores it in WALLET_FILE; prints extended public key to STDOUT
    Create {
        /// A file which will contain the wallet; must not exist
        #[clap(default_value = WALLET_FILE)]
        file: PathBuf
    },

    /// Returns an address for a given XPUBKEY and HD path
    Derive {
        /// Extended public key
        xpubkey: ExtendedPubKey,
        /// Number of account to use
        account: u32,
        /// Index to use for the address under the account
        address: u32,
    },
}
