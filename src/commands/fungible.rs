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
use lnpbp::bitcoin::{TxIn};
use lnpbp::{bp, rgb};
use ::rgb::fungible;


/// Defines information required to generate bitcoin transaction output from
/// command-line argument
pub struct Output {
    pub amount: bitcoin::Amount,
    pub lock: bp::PubkeyScriptSource,
}

/// Defines information required to generate bitcoin transaction input from
/// command-line argument
pub struct Input {
    pub txin: TxIn,
    pub unlock: bp::PubkeyScriptSource,
}


#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Lists all known asset ids
    List {
        /// Include only the assets which are owned by the known accounts
        #[clap(short, long)]
        owned: bool
    },

    /// Transfers some asset to another party
    Pay {
        /// Use custom commitment output
        #[clap(short, long)]
        commit_into: Option<Output>,

        /// Adds output
        #[clap(short, long)]
        txout: Vec<Output>,

        /// Adds input
        #[clap(short, long)]
        txin: Vec<Input>,

        /// Allocates other assets to custom outputs
        #[clap(short, long)]
        allocate: Vec<fungible::Allocation>,

        /// Saves transaction to a file instead of publishing it
        #[clap(short, long)]
        transaction: Option<PathBuf>,

        /// Saves proof data to a file instead of sending it to the remote party
        #[clap(short, long)]
        proof: Option<PathBuf>,

        /// Invoice to pay
        invoice: fungible::Invoice,

        /// Overrides amount provided in the invoice
        amount: rgb::data::Amount,
    }
}
