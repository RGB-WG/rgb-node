// For now this is a mod, but later will be a library

pub use lnpbp::rgb;
pub use rgb::prelude::*;

pub mod fungible {
    use super::*;
    use core::convert::TryFrom;

    #[derive(Clone, PartialEq, Eq, Hash, Debug, Display, Default)]
    #[display_from(Display)]
    pub struct Amount(rgb::Amount, u8);

    impl Amount {
        #[inline]
        pub fn with_asset_coins(asset: &Asset, coins: f32) -> Self {
            let bits = asset.fractional_bits;
            let full = (coins.trunc() as u64) << bits as u64;
            let fract = coins.fract() as u64;
            Self(full + fract, asset.fractional_bits)
        }

        #[inline]
        fn with_asset_sats(asset: &Asset, sats: u64) -> Self {
            Self(sats, asset.fractional_bits)
        }

        #[inline]
        pub fn coins(&self) -> f32 {
            let full = self.0 >> self.1;
            let fract = self.0 ^ (full << self.1);
            full as f32 + fract as f32 / 10u64.pow(self.1 as u32) as f32
        }

        #[inline]
        pub fn sats(&self) -> u64 {
            self.0
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug, Display, Default)]
    #[display_from(Display)]
    pub struct Asset {
        ticker: String,
        name: String,
        description: Option<String>,
        supply: Supply,
        dust_limit: Option<Amount>,
        fractional_bits: u8,
    }

    impl TryFrom<Genesis> for Asset {
        type Error = String; //schema::ValidationError;

        fn try_from(genesis: Genesis) -> Result<Self, Self::Error> {
            unimplemented!()
        }
    }

    impl Asset {
        fn issue() -> Self {
            unimplemented!()
            //Genesis {}
        }

        #[inline]
        fn ticker(&self) -> &str {
            self.ticker.as_str()
        }

        #[inline]
        fn name(&self) -> &str {
            self.name.as_str()
        }

        #[inline]
        fn description(&self) -> Option<&str> {
            match &self.description {
                None => None,
                Some(s) => Some(s.as_str()),
            }
        }

        #[inline]
        fn supply(&self) -> Supply {
            self.supply.clone()
        }

        #[inline]
        fn dust_limit(&self) -> Option<Amount> {
            self.dust_limit.clone()
        }

        #[inline]
        fn fractional_bits(&self) -> u8 {
            self.fractional_bits
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug, Display, Default)]
    #[display_from(Display)]
    pub struct Supply {
        pub known_circulating: Amount,
        pub total: Option<Amount>,
    }
}
