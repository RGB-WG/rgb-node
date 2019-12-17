#[macro_use] extern crate clap;
extern crate bitcoin_wallet;

use std::{io, fs::File, io::prelude::*};
use bitcoin::util::bip32::{ExtendedPubKey, ChildNumber};
use bitcoin::network::constants::Network;
use bitcoin::Address;
use bitcoin_wallet::{account::*, context::*};

enum Verbosity {
    Silent = 0,
    Laconic = 1,
    Verbose = 2
}
use Verbosity::*;

impl From<u64> for Verbosity {
    fn from(level: u64) -> Self {
        match level {
            0 => Silent,
            1 => Laconic,
            2 => Verbose,
            _ => panic!("Unknown level of verbosity")
        }
    }
}
impl From<&Verbosity> for i8 {
    fn from(verb: &Verbosity) -> Self {
        match verb {
            Silent => 0,
            Laconic => 1,
            Verbose => 2,
        }
    }
}

struct Config {
    verbosity: Verbosity,
}
static mut CONFIG: Config = Config {
    verbosity: Verbosity::Silent
};

macro_rules! vprintln {
    ( $level:expr, $($arg:tt)* ) => ({
        unsafe {
            let lvl = i8::from(&CONFIG.verbosity);
            if lvl - ($level) as i8 >= 0 {
                eprintln!($($arg)*);
            }
        }
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Error(String);
impl Error {
    fn from(str: &str) -> Self {
        return Error(String::from(str))
    }
}
impl<E: ToString> From<E> for Error {
    fn from(err: E) -> Self {
        Error(err.to_string())
    }
}
impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.0)
    }
}


const HD_PURPOSE: u32 = 0x84;
const HD_COIN: u32 = 0x524742; // Base16 encoding for "RGB"


fn main() -> io::Result<()> {
    let matches = clap_app!(lbx =>
        (@setting SubcommandRequiredElseHelp)
        (version: "0.2.0")
        (author: "Dr Maxim Orlovsky <orlovsky@pandoracore.com>")
        (about: "Command-line wallet for Bitcoin and RGB assets")
        (@arg verbose: -v ... #{0,2} +global "Sets verbosity level")
        (@subcommand ("wallet-create") =>
            (about: "creates a new wallet and stores it in WALLET_FILE; prints extended public key to STDOUT")
            (@arg WALLET_FILE: +required "A file which will contain the wallet; must not exist")
        )
        (@subcommand ("address-derive") =>
            (about: "returns an address for a given XPUBKEY and HD path")
            (@arg XPUBKEY: +required "Extended public key")
            (@arg ACCOUNT: +required "Number of account to use")
            (@arg ADDR: +required "Index to use for the address under the acocunt")
        )
    ).get_matches();

    unsafe {
        CONFIG.verbosity = Verbosity::from(matches.occurrences_of("verbose"));
    }

    if let Err(err) = match matches.subcommand() {
        ("wallet-create", Some(sm)) => wallet_create(sm.value_of("WALLET_FILE").unwrap()),
        ("address-derive", Some(sm)) => address_derive(
            value_t_or_exit!(sm, "XPUBKEY", ExtendedPubKey),
            value_t_or_exit!(sm, "ACCOUNT", u32),
            value_t_or_exit!(sm, "ADDR", u32),
        ),
        _ => Ok(()),
    } {
        Err(err.into())
    } else {
        Ok(())
    }
}

fn wallet_create(filename: &str) -> Result<(), Error> {
    vprintln!(Laconic, "Generating new HD wallet file");

    let password = rpassword::prompt_password_stderr("Password for wallet encryption: ")?;
    if !(8..256).contains(&password.len()) {
        return Err(Error::from("The length of the password must be at least 8 and no more than 256 characters"));
    }

    vprintln!(Verbose, "- collecting 64 bits of entropy");
    let master = MasterAccount::new(
        MasterKeyEntropy::Paranoid,
        Network::Bitcoin,
        password.as_str()
    ).unwrap();

    vprintln!(Verbose, "- the generated extended pubkey identifier:");
    println!("{}", master.master_public());

    let mut file = File::create(filename)?;
    file.write_all(master.encrypted())?;
    Ok(())
}

fn address_derive(xpubkey: ExtendedPubKey, acc_i: u32, addr_i: u32) -> Result<(), Error> {
    vprintln!(Laconic, "Generating new address from account #{} and index {}", acc_i, addr_i);
    let ctx = SecpContext::new();
    let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: HD_PURPOSE})?;
    let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: HD_COIN})?;
    let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: acc_i})?;
    let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: 0})?;
    let xpubkey = ctx.public_child(&xpubkey, ChildNumber::Normal{index: addr_i})?;
    vprintln!(Verbose, "- the generated pubkey in compressed format:");
    println!("{}", Address::p2wpkh(&xpubkey.public_key, Network::Bitcoin));
    Ok(())
}
