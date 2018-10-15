use bitcoin::util::hash::Sha256dHash;
use secp256k1::Error;
use secp256k1::PublicKey;
use secp256k1::Secp256k1;
use secp256k1::SecretKey;
use secp256k1::Verification;
use std::fmt;

// Create a wrapper around secp256k1's data type, to improve readability
pub struct ECTweakFactor(SecretKey);

impl ECTweakFactor {
    pub fn from_pk_data<C>(secp: &Secp256k1<C>, pk: &PublicKey, data: &Sha256dHash) -> Result<ECTweakFactor, Error> {
        let mut tmp = [0; 65];

        tmp[..33].copy_from_slice(&pk.serialize());
        tmp[33..].copy_from_slice(data.as_bytes());

        let tweak_factor = SecretKey::from_slice(&secp, Sha256dHash::from_data(&tmp).as_bytes())?;

        Ok(ECTweakFactor(tweak_factor))
    }

    pub fn as_inner(&self) -> &SecretKey {
        &self.0
    }

    pub fn add_to_pk<C: Verification>(&self, secp: &Secp256k1<C>, pk: &mut PublicKey) -> Result<(), Error> {
        pk.add_exp_assign(&secp, &self.as_inner())
    }
}

impl fmt::Display for ECTweakFactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}