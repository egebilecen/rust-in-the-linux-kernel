use crate::present80::byte_vec::ByteVec;
use kernel::prelude::*;

pub(crate) struct Key {
    pub(crate) bytes: ByteVec,
}

impl From<Vec<u8>> for Key {
    fn from(value: Vec<u8>) -> Self {
        Self {
            bytes: ByteVec::new(value),
        }
    }
}
