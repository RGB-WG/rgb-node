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


use std::{io, fs};
use std::path::PathBuf;
use std::collections::HashMap;
use num_traits::{ToPrimitive, FromPrimitive};
use num_derive::{ToPrimitive, FromPrimitive};
use rand::{thread_rng, RngCore};

use bitcoin::secp256k1;
use bitcoin::util::bip32::{ExtendedPubKey, DerivationPath};
use bitcoin_wallet::{account::Seed, context::SecpContext};

use lnpbp::csv::{serialize, Storage};


#[derive(From, Debug, Display)]
#[display_from(Debug)]
pub enum Error {
    #[derive_from]
    IoError(io::Error),

    #[derive_from]
    SerializeError(serialize::Error),

    FutureVersion,
}


#[derive(Debug, Display)]
#[display_from(Debug)]
pub struct KeyringManager {
    main: Hierarchical,
    others: Vec<Box<dyn Keyring>>,
}

impl KeyringManager {
    pub fn load(file_name: PathBuf) -> Result<Self, Error> {
        let file = fs::File::open(file_name)?;
        let mut reader = io::BufReader::new(file);
        Self::storage_deserialize(&mut reader)
    }

    pub fn store(&self, file_name: PathBuf) -> Result<usize, Error> {
        let file = fs::File::create(file_name)?;
        let mut writer = io::BufWriter::new(file);
        self.storage_serialize(&mut writer)
    }

    pub fn get_accounts(&self) -> Vec<Account> {
        let mut accounts = self.main.get_accounts();
        accounts.extend(self.others.iter().map(Keyring::get_accounts).flatten());
        accounts
    }
}

impl serialize::Storage for KeyringManager {
    fn storage_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, Error> {
        let mut len = self.main.storage_serialize(&mut e)?;
        let count: u16 = self.others.len() as u16;
        len += count.storage_serialize(&mut e)?;
        Ok(self.others.iter().try_fold(len, |mut len, keyring| {
            len += keyring.get_type().storage_serialize(&mut e)?;
            len += keyring.storage_serialize(&mut e)?;
            Ok(len)
        })?)
    }

    fn storage_deserialize<D: io::Read>(mut d: D) -> Result<Self, Error> {
        let main = Hierarchical::storage_deserialize(&mut d)?;
        let count = u16::storage_deserialize(&mut d)?;
        let mut others = Vec::new();
        for _ in count {
            let keyring = match KeyringType::storage_deserialize(&mut d)? {
                KeyringType::Hierarchical =>
                    Hierarchical::storage_deserialize(&mut d)? as dyn Keyring,
                KeyringType::Keyset =>
                    Keyset::storage_deserialize(&mut d)? as dyn Keyring,
                _ =>
                    Err(Error::FutureVersion)?
            };
            others.push(Box::new(keyring));
        }
        Ok(Self {
            main,
            others,
        })
    }
}


#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Display, ToPrimitive, FromPrimitive)]
#[display_from(Debug)]
pub enum KeyringType {
    Hierarchical,
    Keyset,
}


pub trait Keyring: serialize::Storage {
    fn get_accounts(&self) -> Vec<Account>;
    fn get_type(&self) -> KeyringType;
}


#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Hierarchical {
    xpubkey: ExtendedPubKey,
    encrypted: Vec<u8>,
    accounts: HashMap<Account, DerivationPath>,
}

impl Hierarchical {
    pub fn new(passphrase: &str) -> Self {
        let mut random = vec![0u8; 32];
        thread_rng().fill_bytes(random.as_mut_slice());
        let seed = Seed(random);
        let context = SecpContext::new();
        let encrypted = seed.encrypt(passphrase)?;
        let master_key = context.master_private_key(bitcoin::Network::Bitcoin, &seed)?;
        let xpubkey = context.extended_public_from_private(&master_key);
        Self {
            xpubkey,
            encrypted,
            accounts: HashMap::new()
        }
    }
}

impl Keyring for Hierarchical {
    #[inline]
    fn get_accounts(&self) -> Vec<Account> {
        self.accounts.keys().collect()
    }

    #[inline]
    fn get_type(&self) -> KeyringType { KeyringType::Hierarchical }
}

impl serialize::Storage for Hierarchical {
    fn storage_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, Error> {
        Ok(
            self.xpubkey.storage_serialize(&mut e)? +
            self.encrypted.storage_serialize(&mut e)? +
            self.accounts.storage_serialize(&mut e)?
        )
    }

    fn storage_deserialize<D: io::Read>(mut d: D) -> Result<Self, Error> {
        Ok(Self {
            xpubkey: ExtendedPubKey::storage_deserialize(&mut d)?,
            encrypted: Vec::storage_deserialize(&mut d)?,
            accounts: HashMap::<Account, DerivationPath>::storage_deserialize(&mut d)?,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Keyset {
    account: Account,
    keys: HashMap<secp256k1::PublicKey, Vec<u8>>,
}

impl Keyring for Keyset {
    #[inline]
    fn get_accounts(&self) -> Vec<Account> {
        vec![self.account]
    }

    #[inline]
    fn get_type(&self) -> KeyringType { KeyringType::Keyset }
}


impl serialize::Storage for Keyset {
    fn storage_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, Error> {
        Ok(
            self.account.storage_serialize(&mut e)? +
            self.keys.storage_serialize(&mut e)?
        )
    }

    fn storage_deserialize<D: io::Read>(mut d: D) -> Result<Self, Error> {
        Ok(Self {
            account: Account::storage_deserialize(&mut d)?,
            keys: HashMap::<secp256k1::PublicKey, Vec<u8>>::storage_deserialize(&mut d)?,
        })
    }
}


#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Account {
    pub name: String,
    pub description: String,
}

impl serialize::Storage for Account {
    fn storage_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, Error> {
        Ok(
            self.name.storage_serialize(&mut e)? +
            self.description.storage_serialize(&mut e)?
        )
    }

    fn storage_deserialize<D: io::Read>(mut d: D) -> Result<Self, Error> {
        Ok(Self {
            name: String::storage_deserialize(&mut d)?,
            description: String::storage_deserialize(&mut d)?,
        })
    }
}
