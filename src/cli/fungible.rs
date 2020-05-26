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

use bech32::{self, ToBase32};
use clap::Clap;

use lnpbp::strict_encoding::strict_encode;

use super::Runtime;
use crate::api::fungible::{Issue, Transfer};
use crate::error::ServiceErrorDomain;
use crate::fungible::IssueStructure;

#[derive(Clap, Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Command {
    /// Lists all known assets
    List,

    /// Creates a new asset
    Issue(Issue),

    /// Transfers some asset to another party
    Transfer(Transfer),
}

impl Command {
    pub fn exec(self, runtime: Runtime) -> Result<(), ServiceErrorDomain> {
        match self {
            Command::List => {
                println!("\nKnown assets:\n\n");
                unimplemented!();
                Ok(())
            }
            Command::Issue(issue) => issue.exec(runtime),
            Command::Transfer(_) => unimplemented!(),
        }
    }
}

impl Issue {
    pub fn exec(self, mut runtime: Runtime) -> Result<(), ServiceErrorDomain> {
        info!("Issuing asset ...");
        debug!("{}", self.clone());

        runtime.issue(self)?;
        // TODO: Wait for the information from push notification

        /*let (asset, genesis) = debug!("Asset information:\n {}\n", asset);
        trace!("Genesis contract:\n {}\n", genesis);

        let bech = bech32::encode(
            crate::RGB_BECH32_HRP_GENESIS,
            strict_encode(&genesis).to_base32(),
        )
        .unwrap();
        info!(
            "Use this string to send information about the issued asset:\n{}\n",
            bech
        );
         */

        Ok(())
    }
}
