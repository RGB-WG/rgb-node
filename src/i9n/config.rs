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

use super::{fungible, Runtime};
use crate::BootstrapError;

use lnpbp::lnp::transport::zmq::SocketLocator;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Config {
    pub endpoint: SocketLocator,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: SocketLocator::Inproc("fungible".to_string()),
        }
    }
}
