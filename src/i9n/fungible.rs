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

use lnpbp::bp;
use lnpbp::rgb::Amount;

use super::Runtime;
use crate::fungible::{IssueStructure, Outcoins};
use crate::util::SealSpec;

impl Runtime {
    pub fn issue(
        &self,
        _network: bp::Network,
        _ticker: String,
        _name: String,
        _description: Option<String>,
        _issue_structure: IssueStructure,
        _allocations: Vec<Outcoins>,
        _precision: u8,
        _prune_seals: Vec<SealSpec>,
        _dust_limit: Option<Amount>,
    ) -> Result<(), String> {
        // TODO: pass RPC call via ZMQ inproc socket
        Ok(())
    }
}
