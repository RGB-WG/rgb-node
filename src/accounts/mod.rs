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


use std::{io, fs, fmt, hash::Hash, convert::TryInto};
use std::path::PathBuf;
use std::collections::HashMap;
use rand::{thread_rng, RngCore};

use lnpbp::bp;
use lnpbp::bitcoin;
use bitcoin::secp256k1;
use bitcoin::util::bip32::{ExtendedPubKey, DerivationPath, ChildNumber};
use bitcoin_wallet::{account::Seed, context::SecpContext};

use lnpbp::csv::serialize::{self, network::*, storage::*};

use crate::error::Error;


#[derive(Debug)]
pub struct KeyringManager {
    pub keyrings: Vec<Keyring>,
}

impl fmt::Display for KeyringManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f,
                 "\n {:<8}    {:>4}    {:<24}    {:<32}     {}",
                 "Keyring", "Id", "Name", "Derivation path", "Description")?;
        writeln!(f,
                 "-------------------------------------------------------------------------------------------------------------------------------")?;
        self.keyrings.iter().enumerate().try_for_each(|(kidx, keyring)| {
            let (mut name, id) = match keyring {
                Keyring::Hierarchical { .. } => ("HD:", format!("{}:", kidx + 1)),
                Keyring::Keyset { .. } => ("Legacy:", "".to_string()),
                _ => unreachable!(),
            };
            keyring.get_accounts().iter().enumerate().try_for_each(|(aidx, acc)| {
                let path = match acc.derivation_path {
                    Some(ref dp) => format!("{}", dp),
                    None => "-".to_string()
                };
                writeln!(f,
                         " {:<8}    {:>4}    {:<24}    {:<32}     {}",
                         name, format!("{}{}", id, aidx + 1), acc.name, path, acc.description)?;
                name = "";
                Ok(())
            })
        })
    }
}

impl KeyringManager {
    pub fn setup(file_name: PathBuf, passphrase: &str) -> Result<Self, Error> {
        let main = Keyring::new(passphrase);
        let me = Self { keyrings: vec![main] };

        let file = fs::File::create(file_name)?;
        let mut writer = io::BufWriter::new(file);
        me.storage_serialize(&mut writer)?;

        Ok(me)
    }

    pub fn load(file_name: PathBuf) -> Result<Self, Error> {
        let file = fs::File::open(file_name)?;
        let mut reader = io::BufReader::new(file);
        Ok(Self::storage_deserialize(&mut reader)?)
    }

    pub fn store(&self, file_name: PathBuf) -> Result<usize, Error> {
        let file = fs::File::create(file_name)?;
        let mut writer = io::BufWriter::new(file);
        Ok(self.storage_serialize(&mut writer)?)
    }

    pub fn get_accounts(&self) -> Vec<Account> {
        self.keyrings.iter().map(Keyring::get_accounts).flatten().collect()
    }
}

impl serialize::Network for KeyringManager {
    fn network_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, serialize::Error> {
       self.keyrings.network_serialize(&mut e)
    }

    fn network_deserialize<D: io::Read>(mut d: D) -> Result<Self, serialize::Error> {
        Ok(Self {
            keyrings: Vec::<Keyring>::network_deserialize(&mut d)?,
        })
    }
}


#[non_exhaustive]
#[derive(Clone, Debug, Display)]
#[display_from(Debug)]
pub enum Keyring {
    Hierarchical {
        xpubkey: ExtendedPubKey,
        encrypted: Vec<u8>,
        accounts: Vec<Account>,
    },
    Keyset {
        account: Account,
        keys: Vec<EncryptedKeypair>,
    },
}


impl Keyring {
    pub fn new(passphrase: &str) -> Self {
        let mut random = vec![0u8; 32];
        thread_rng().fill_bytes(random.as_mut_slice());
        let seed = Seed(random);
        let context = SecpContext::new();
        let encrypted = seed.encrypt(passphrase)
            .expect("Encryption failed");
        let master_key = context.master_private_key(bp::Network::Mainnet.try_into().unwrap(), &seed)
            .expect("Public key generation failed");
        let xpubkey = context.extended_public_from_private(&master_key);
        Keyring::Hierarchical {
            xpubkey,
            encrypted,
            accounts: vec![Account {
                name: "bitcoin_default".to_string(),
                description: "Bitcoin transactions signatures".to_string(),
                derivation_path: Some("m/44'/0'/0'/0'/0".parse()
                    .expect("Compile-time default derivation path error"))
            }]
        }
    }

    pub fn add_account(&mut self, account: Account) -> Result<(), Error> {
        use Keyring::*;
        match self {
            Hierarchical { accounts, .. } => Ok(accounts.push(account)),
            Keyset { .. } => Err(Error::OperationNotSupported("for the legacy keyring format".to_string())),
            _ => unreachable!()
        }
    }

    #[inline]
    fn get_accounts(&self) -> Vec<Account> {
        use Keyring::*;
        match self {
            Hierarchical { accounts, .. } => accounts.clone(),
            Keyset { account, .. } => vec![account.clone()],
            _ => unreachable!()
        }
    }
}

impl serialize::Network for Keyring {
    fn network_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, serialize::Error> {
        use Keyring::*;
        Ok(match self {
            Hierarchical { xpubkey, accounts, encrypted } =>
                1u8.network_serialize(&mut e)? +
                xpubkey.network_serialize(&mut e)? +
                encrypted.network_serialize(&mut e)? +
                accounts.network_serialize(&mut e)?,
            Keyset { account, keys } =>
                0u8.network_serialize(&mut e)? +
                account.network_serialize(&mut e)? +
                keys.network_serialize(&mut e)?,
            _ => unreachable!()
        })
    }

    fn network_deserialize<D: io::Read>(mut d: D) -> Result<Self, serialize::Error> {
        Ok(match u8::network_deserialize(&mut d)? {
            0u8 => Keyring::Keyset {
                account: Account::network_deserialize(&mut d)?,
                keys: Vec::<EncryptedKeypair>::network_deserialize(&mut d)?,
            },
            1u8 => Keyring::Hierarchical {
                xpubkey: ExtendedPubKey::network_deserialize(&mut d)?,
                encrypted: Vec::network_deserialize(&mut d)?,
                accounts: Vec::<Account>::network_deserialize(&mut d)?,
            },
            u => Err(serialize::Error::EnumValueUnknown(u))?,
        })
    }
}


#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct EncryptedKeypair {
    pub pk: secp256k1::PublicKey,
    pub encrypted_sk: Vec<u8>,
}

impl serialize::Network for EncryptedKeypair {
    fn network_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, serialize::Error> {
        Ok(
            self.pk.network_serialize(&mut e)? +
            self.encrypted_sk.network_serialize(&mut e)?
        )
    }

    fn network_deserialize<D: io::Read>(mut d: D) -> Result<Self, serialize::Error> {
        Ok(Self {
            pk: secp256k1::PublicKey::network_deserialize(&mut d)?,
            encrypted_sk: Vec::<u8>::network_deserialize(&mut d)?,
        })
    }
}


#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct Account {
    pub name: String,
    pub description: String,
    pub derivation_path: Option<DerivationPath>,
}

impl serialize::Network for Account {
    fn network_serialize<E: io::Write>(&self, mut e: E) -> Result<usize, serialize::Error> {
        Ok(
            self.name.network_serialize(&mut e)? +
            self.description.network_serialize(&mut e)? +
            self.derivation_path.network_serialize(&mut e)?
        )
    }

    fn network_deserialize<D: io::Read>(mut d: D) -> Result<Self, serialize::Error> {
        Ok(Self {
            name: String::network_deserialize(&mut d)?,
            description: String::network_deserialize(&mut d)?,
            derivation_path: Option::<DerivationPath>::network_deserialize(&mut d)?,
        })
    }
}
