use bech32::ToBase32;

pub trait ToBech32Data {
    fn to_bech32data(&self) -> String;
}

pub trait FromBech32Data {
    fn from_bech32data(data: String) -> Vec<u8>;
}

impl ToBech32Data for Vec<u8> {
    fn to_bech32data(&self) -> String {
        ::bech32::encode("data", self.to_base32())
            .expect("HRP is hardcoded and can't fail")
    }
}
