extern crate bitcoin;
extern crate core;
extern crate secp256k1;

#[cfg(test)]
mod tests;

pub mod contract;
pub mod proof;
pub mod traits;
pub mod utils;
pub mod pay_to_contract;