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
use lnpbp::lnp::presentation::Encode;
use lnpbp::lnp::Unmarshall;
use lnpbp::rgb::Amount;

use super::Runtime;
use crate::api::{fungible::Issue, fungible::Request, reply, Reply};
use crate::fungible::{IssueStructure, Outcoins};
use crate::util::SealSpec;

impl Runtime {
    pub fn issue(
        &mut self,
        _network: bp::Network,
        ticker: String,
        title: String,
        description: Option<String>,
        issue_structure: IssueStructure,
        allocate: Vec<Outcoins>,
        precision: u8,
        _prune_seals: Vec<SealSpec>,
        dust_limit: Option<Amount>,
    ) -> Result<(), reply::Failure> {
        // TODO: Make sure we use the same network
        let (supply, inflatable) = match issue_structure {
            IssueStructure::SingleIssue => (None, None),
            IssueStructure::MultipleIssues {
                max_supply,
                reissue_control,
            } => (Some(max_supply), Some(reissue_control)),
        };
        let command = Request::Issue(Issue {
            ticker,
            title,
            description,
            supply,
            inflatable,
            precision,
            dust_limit,
            allocate,
        });
        let data = command.encode()?;
        self.session_rpc.send_raw_message(data)?;
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = &*self.unmarshaller.unmarshall(&raw)?;
        match reply {
            Reply::Success => Ok(()),
            Reply::Failure(failmsg) => Err(failmsg.clone()),
            _ => Err(reply::Failure {
                code: 0,
                info: "Unexpected server response".to_string(),
            }),
        }
    }

    pub fn transfer(&mut self) -> Result<(), String> {
        Ok(())
    }
}
