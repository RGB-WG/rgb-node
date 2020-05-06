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

use clap::Clap;

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Lists available funds within a given scope of accounts and deposit boxes
    Funds {
        /// Amount of deposit boxes to list
        #[clap(short = "N", long, default_value = "10")]
        no: u8,

        /// Offset for the first deposit box
        #[clap(short = "O", long, default_value = "0")]
        offset: u32,

        /// Tag name of the account to list deposit boxes
        account: String,

        /// Request funds on the specified deposit types only
        #[clap(arg_enum)]
        #[clap(default_value = "wpkh")]
        deposit_types: Vec<DepositType>,
    },
}

#[derive(Clap, Clone, PartialEq, Eq, Debug)]
pub enum DepositType {
    PK,
    PKH,
    WPKH,
    TPK,
}
