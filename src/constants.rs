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


pub const BPD_API_ADDR: &str = "ipc://bp.queryd.api";
pub const BPD_PUSH_ADDR: &str = "ipc://bp.queryd.push";

pub const WALLET_FILE: &str = "./wallet.dat";

pub const HD_PURPOSE: u32 = 0x84;
pub const HD_COIN: u32 = 0x524742; // Base16 encoding for "RGB"
