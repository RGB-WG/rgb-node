
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display_from(Debug)]
#[non_exhaustive]
pub enum Challenge {
    Signature(bitcoin::PublicKey),
    Multisig(u32, Vec<bitcoin::PublicKey>),
    Custom(LockScript),
}
