// RGB Rust Library
// Written in 2019 by
//     Dr. Maxim Orlovsky <dr.orlovsky@gmail.com>
// basing on the original RGB rust library by
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

//! General utilitary functions

use bitcoin::consensus::encode::*;

// For optional strings we use zero-length string to represent `None` value

impl<S: Encoder> Encodable<S> for Option<String> {
    fn consensus_encode(&self, s: &mut S) -> Result<(), Error> {
        match self {
            Some(str) => str.consensus_encode(s),
            None => "".consensus_encode(s),
        }
    }
}

impl<D: Decoder> Decodable<D> for Option<String> {
    fn consensus_decode(d: &mut D) -> Result<Option<String>, Error> {
        let string: String = Decodable::consensus_decode(d)?;
        match string.len() {
            0 => Ok(None),
            _ => Ok(Some(string)),
        }
    }
}
