#[macro_export]
macro_rules! type_map {
    { } =>  {
        {
            ::std::collections::BTreeMap::new()
        }
    };

    { $($key:expr => $value:expr),+ } => {
        {
            use ::num_traits::ToPrimitive;
            let mut m = ::std::collections::BTreeMap::new();
            $(
                m.insert($key.to_usize().unwrap(), $value);
            )+
            m
        }
    }
}
