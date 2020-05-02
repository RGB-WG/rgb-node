pub trait Store {
    type Error: Error;
}
pub trait Error: std::error::Error + Sized {}

pub struct DiskStorage {}
