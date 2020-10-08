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

use core::convert::TryFrom;

/// Magic numbers here are used to distinguish files with RGB data of different
/// type.
///
/// NB: These numbers are used with binary data serialization only; they are
/// not a part of the commitments, ids or network-serialized packets.
///
/// Rationale: convenience for wallet data import/export and extra-wallet
/// file storage; additional check not to mis-interpret byte sequences
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
#[repr(u32)]
pub enum MagicNumber {
    /// Equals to first 4 bytes of SHA256("rgb:schema")
    /// = 18429ce35af7f898f765417b28471ab454b89ceff6fc33de77ff5fd98e066bc3
    /// Check with `echo -n "rgb:schema" | shasum -a 256`
    Schema = 0x18429ce3,

    /// Equals to first 4 bytes of SHA256("rgb:gensis")
    /// = 2e91cbc08b6205efb4f908bb9bd3fcf5c148763f7b23b0506ef64ffd414fc9b4
    Genesis = 0x2e91cbc0,

    /// Equals to first 4 bytes of SHA256("rgb:transition")
    /// = bf11926e3db131632bdfa8f996d52d6d19e25d0884c922365ff8cd3c73f10198
    Transition = 0xbf11926e,

    /// Equals to first 4 bytes of SHA256("rgb:extension")
    /// = 296c892df8b4231058b9ea8b55e0bf6a08069e4fea700a570c535de2d9f39e45
    Extension = 0x296c892d,

    /// Equals to first 4 bytes of SHA256("rgb:anchor")
    /// = dd53b6f17c16915ecd01de7935b5c38497f6f6c49b97627296496dc31a6ca86b
    Anchor = 0xdd53b6f1,

    /// Equals to first 4 bytes of SHA256("rgb:consignment")
    /// = 4c82bf5385ab9027f15f1ce17a8007956fe8f38cbad2ee312cf2c55b72a69420
    Consignment = 0x4c82bf53,

    /// Equals to first 4 bytes of SHA256("rgb:stash")
    /// = cd22a2cb85720d51f1616752cb85059a02f3d35f7dda30a4ca981b59b0924354
    Stash = 0xcd22a2cb,
}

impl MagicNumber {
    pub fn to_u32(&self) -> u32 {
        use std::mem;
        let m;
        unsafe {
            m = mem::transmute::<Self, u32>(self.clone());
        }
        m as u32
    }
}

impl TryFrom<u32> for MagicNumber {
    type Error = u32;
    fn try_from(number: u32) -> Result<Self, Self::Error> {
        Ok(match number {
            n if n == Self::Schema.to_u32() => Self::Schema,
            n if n == Self::Genesis.to_u32() => Self::Genesis,
            n if n == Self::Transition.to_u32() => Self::Transition,
            n if n == Self::Anchor.to_u32() => Self::Anchor,
            n if n == Self::Consignment.to_u32() => Self::Consignment,
            n if n == Self::Stash.to_u32() => Self::Stash,
            invalid => Err(invalid)?,
        })
    }
}
