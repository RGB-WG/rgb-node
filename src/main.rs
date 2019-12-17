#[macro_use] extern crate clap;
extern crate bitcoin_wallet;

use std::{io, fs::File, io::prelude::*, thread, env, fmt::Display, str::FromStr, process::exit};
use clap::ArgMatches;
use bitcoin::network::constants::Network;
use bitcoin_wallet::account::*;

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
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(err.to_string())
    }
}
impl Into<io::Error> for Error {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self.0)
    }
}

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
    ).get_matches();

    unsafe {
        CONFIG.verbosity = Verbosity::from(matches.occurrences_of("verbose"));
    }

    if let Err(err) = match matches.subcommand() {
        ("wallet-create", Some(sm)) => wallet_create(sm.value_of("WALLET_FILE").unwrap()),
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
