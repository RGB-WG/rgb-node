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

pub mod fungible;
mod reply;

pub use reply::Reply;

pub type Multipart = Vec<zmq::Message>;

impl From<Reply> for Multipart {
    fn from(reply: Reply) -> Self {
        match reply {
            Reply::Success => vec![zmq::Message::from("success")],
            Reply::Failure(err) => vec![
                zmq::Message::from("failure"),
                zmq::Message::from(format!("{:?}", err).as_str()),
            ],
        }
    }
}
