use std::fmt;
use bitcoin::util::hash::Sha256dHash;
use secp256k1::{Error, PublicKey, Secp256k1, SecretKey, Verification};

// Wrapper around secp256k1's data type, to improve readability
pub struct ECTweakFactor(SecretKey);

impl ECTweakFactor {
    pub fn from_pk_data<C>(secp: &Secp256k1<C>, pk: &PublicKey, data: &Sha256dHash) -> Result<ECTweakFactor, Error> {
        // 1. Constructing data for hashing by concatenating bytes of the existing public key and some  hash
        let mut tweaking_data = [0; 65];
        tweaking_data[..33].copy_from_slice(&pk.serialize());
        tweaking_data[33..].copy_from_slice(data.as_bytes());

        // 2. Getting hash of the concatenated value
        let tweaking_hash = Sha256dHash::from_data(&tweaking_data);

        // 3. Converting hash value into a private key
        let tweak_factor = SecretKey::from_slice(&secp, tweaking_hash.as_bytes())?;

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