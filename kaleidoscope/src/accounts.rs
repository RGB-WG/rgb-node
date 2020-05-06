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

use rand::{thread_rng, RngCore};
use std::path::PathBuf;
use std::{convert::TryInto, fmt, fs, io};

use bitcoin::secp256k1;
use bitcoin::util::bip32::{self, ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin_wallet::{account::Seed, context::SecpContext};
use lnpbp::bitcoin;
use lnpbp::bp;
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};

use crate::error::Error;

#[derive(Debug)]
pub struct KeyringManager {
    pub keyrings: Vec<Keyring>,
}

impl fmt::Display for KeyringManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\n {:<8}    {:>4}    {:<24}    {:<32}     {}",
            "Keyring", "Id", "Name", "Derivation path", "Description"
        )?;
        writeln!(f,
                 "-------------------------------------------------------------------------------------------------------------------------------")?;
        self.keyrings
            .iter()
            .enumerate()
            .try_for_each(|(kidx, keyring)| {
                let (mut name, id) = match keyring {
                    Keyring::Hierarchical { .. } => ("HD:", format!("{}:", kidx + 1)),
                    Keyring::Keyset { .. } => ("Legacy:", "".to_string()),
                };
                keyring
                    .get_accounts()
                    .iter()
                    .enumerate()
                    .try_for_each(|(aidx, acc)| {
                        let path = match acc.derivation_path {
                            Some(ref dp) => format!("{}", dp),
                            None => "-".to_string(),
                        };
                        writeln!(
                            f,
                            " {:<8}    {:>4}    {:<24}    {:<32}     {}",
                            name,
                            format!("{}{}", id, aidx + 1),
                            acc.name,
                            path,
                            acc.description
                        )?;
                        name = "";
                        Ok(())
                    })
            })
    }
}

impl KeyringManager {
    pub fn setup(file_name: PathBuf, passphrase: &str) -> Result<Self, Error> {
        let main = Keyring::new(passphrase);
        let me = Self {
            keyrings: vec![main],
        };

        let file = fs::File::create(file_name)?;
        let mut writer = io::BufWriter::new(file);
        me.strict_encode(&mut writer)?;

        Ok(me)
    }

    pub fn load(file_name: PathBuf) -> Result<Self, Error> {
        let file = fs::File::open(file_name)?;
        let mut reader = io::BufReader::new(file);
        Ok(Self::strict_decode(&mut reader)?)
    }

    pub fn store(&self, file_name: PathBuf) -> Result<usize, Error> {
        let file = fs::File::create(file_name)?;
        let mut writer = io::BufWriter::new(file);
        Ok(self.strict_encode(&mut writer)?)
    }

    pub fn get_accounts(&self) -> Vec<Account> {
        self.keyrings
            .iter()
            .map(Keyring::get_accounts)
            .flatten()
            .collect()
    }

    pub fn get_main_keyring(&self) -> &Keyring {
        self.keyrings.first().unwrap()
    }
}

impl StrictEncode for KeyringManager {
    type Error = strict_encoding::Error;

    fn strict_encode<E: io::Write>(&self, mut e: E) -> Result<usize, Self::Error> {
        self.keyrings.strict_encode(&mut e)
    }
}

impl StrictDecode for KeyringManager {
    type Error = strict_encoding::Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        Ok(Self {
            keyrings: Vec::<Keyring>::strict_decode(&mut d)?,
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
        let encrypted = seed.encrypt(passphrase).expect("Encryption failed");
        let master_key = context
            .master_private_key(bp::Network::Mainnet.try_into().unwrap(), &seed)
            .expect("Public key generation failed");
        let xpubkey = context.extended_public_from_private(&master_key);
        Keyring::Hierarchical {
            xpubkey,
            encrypted,
            accounts: vec![Account {
                name: "bitcoin_default".to_string(),
                description: "Bitcoin transactions signatures".to_string(),
                derivation_path: Some(
                    "m/44'/0'/0'/0'/0"
                        .parse()
                        .expect("Compile-time default derivation path error"),
                ),
            }],
        }
    }

    pub fn add_account(&mut self, account: Account) -> Result<(), Error> {
        use Keyring::*;
        match self {
            Hierarchical { accounts, .. } => Ok(accounts.push(account)),
            Keyset { .. } => Err(Error::OperationNotSupported(
                "for the legacy keyring format".to_string(),
            )),
        }
    }

    pub fn list_deposit_boxes(
        &self,
        account_tag: &String,
        offset: u32,
        no: u8,
    ) -> Option<Vec<DepositBox>> {
        if let Keyring::Hierarchical {
            xpubkey, encrypted, ..
        } = self
        {
            let account = self.get_account(account_tag)?;
            let dp = account.derivation_path.as_ref().unwrap().clone();
            let secp = secp256k1::Secp256k1::new();
            let to = offset + (no as u32);
            let mut dp_iter = dp.children_from(ChildNumber::Normal { index: offset });

            if Err(bip32::Error::CannotDeriveFromHardenedKey) == xpubkey.derive_pub(&secp, &dp) {
                let password = rpassword::prompt_password_stderr(
                    "Generation of hardened public keys requires unlocking extended private key: ",
                )
                .unwrap();
                let seed = Seed::decrypt(encrypted, &password).expect("Wrong password");
                let xprivkey = ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, &seed.0)
                    .expect("Wrong password");
                Some(
                    (offset..to)
                        .map(|_| {
                            let dp = dp_iter.next().unwrap();
                            let sk = xprivkey.derive_priv(&secp, &dp).unwrap().private_key;
                            DepositBox::from(sk.public_key(&secp))
                        })
                        .collect(),
                )
            } else {
                Some(
                    (offset..to)
                        .map(|_| {
                            let dp = dp_iter.next().unwrap();
                            DepositBox::from(xpubkey.derive_pub(&secp, &dp).unwrap().public_key)
                        })
                        .collect(),
                )
            }
        } else {
            None
        }
    }

    #[inline]
    fn get_accounts(&self) -> Vec<Account> {
        use Keyring::*;
        match self {
            Hierarchical { accounts, .. } => accounts.clone(),
            Keyset { account, .. } => vec![account.clone()],
        }
    }

    pub fn get_account(&self, account_tag: &String) -> Option<Account> {
        self.get_accounts()
            .into_iter()
            .find(|a| a.name == *account_tag)
    }
}

impl StrictEncode for Keyring {
    type Error = strict_encoding::Error;

    fn strict_encode<E: io::Write>(&self, mut e: E) -> Result<usize, Self::Error> {
        use Keyring::*;
        Ok(match self {
            Hierarchical {
                xpubkey,
                accounts,
                encrypted,
            } => {
                1u8.strict_encode(&mut e)?
                    + xpubkey.strict_encode(&mut e)?
                    + encrypted.strict_encode(&mut e)?
                    + accounts.strict_encode(&mut e)?
            }
            Keyset { account, keys } => {
                0u8.strict_encode(&mut e)?
                    + account.strict_encode(&mut e)?
                    + keys.strict_encode(&mut e)?
            }
        })
    }
}

impl StrictDecode for Keyring {
    type Error = strict_encoding::Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        Ok(match u8::strict_decode(&mut d)? {
            0u8 => Keyring::Keyset {
                account: Account::strict_decode(&mut d)?,
                keys: Vec::<EncryptedKeypair>::strict_decode(&mut d)?,
            },
            1u8 => Keyring::Hierarchical {
                xpubkey: ExtendedPubKey::strict_decode(&mut d)?,
                encrypted: Vec::strict_decode(&mut d)?,
                accounts: Vec::<Account>::strict_decode(&mut d)?,
            },
            u => Err(strict_encoding::Error::EnumValueNotKnown(
                "Keyring".to_string(),
                u,
            ))?,
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct EncryptedKeypair {
    pub pk: secp256k1::PublicKey,
    pub encrypted_sk: Vec<u8>,
}

impl StrictEncode for EncryptedKeypair {
    type Error = strict_encoding::Error;

    fn strict_encode<E: io::Write>(&self, mut e: E) -> Result<usize, Self::Error> {
        Ok(self.pk.strict_encode(&mut e)? + self.encrypted_sk.strict_encode(&mut e)?)
    }
}

impl StrictDecode for EncryptedKeypair {
    type Error = strict_encoding::Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        Ok(Self {
            pk: secp256k1::PublicKey::strict_decode(&mut d)?,
            encrypted_sk: Vec::<u8>::strict_decode(&mut d)?,
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

impl StrictEncode for Account {
    type Error = strict_encoding::Error;

    fn strict_encode<E: io::Write>(&self, mut e: E) -> Result<usize, Self::Error> {
        Ok(self.name.strict_encode(&mut e)?
            + self.description.strict_encode(&mut e)?
            + self.derivation_path.strict_encode(&mut e)?)
    }
}

impl StrictDecode for Account {
    type Error = strict_encoding::Error;

    fn strict_decode<D: io::Read>(mut d: D) -> Result<Self, Self::Error> {
        Ok(Self {
            name: String::strict_decode(&mut d)?,
            description: String::strict_decode(&mut d)?,
            derivation_path: Option::<DerivationPath>::strict_decode(&mut d)?,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
pub struct DepositBox(secp256k1::PublicKey);

impl From<secp256k1::PublicKey> for DepositBox {
    #[inline]
    fn from(pk: secp256k1::PublicKey) -> Self {
        Self(pk)
    }
}

impl From<bitcoin::PublicKey> for DepositBox {
    #[inline]
    fn from(pk: bitcoin::PublicKey) -> Self {
        Self(pk.key)
    }
}

impl DepositBox {
    #[inline]
    pub fn get_pubkey(&self) -> bitcoin::PublicKey {
        bitcoin::PublicKey {
            compressed: true,
            key: self.0,
        }
    }

    #[inline]
    pub fn get_p2pkh_addr(&self, network: bitcoin::Network) -> bitcoin::Address {
        use bitcoin::util::address::Payload;
        bitcoin::Address {
            network,
            payload: Payload::PubkeyHash(self.get_pubkey().pubkey_hash()),
        }
    }

    #[inline]
    pub fn get_p2wpkh_addr(&self, network: bitcoin::Network) -> bitcoin::Address {
        bitcoin::Address::p2wpkh(&self.get_pubkey(), network)
    }
}
