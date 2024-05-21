use crate::present80::byte_vec::ByteVec;
use kernel::prelude::*;
use kernel::random::getrandom;

pub(crate) struct Key {
    pub(crate) bytes: ByteVec,
}

impl Key {
    pub(crate) fn new() -> Result<Self> {
        let mut rand_bytes: [u8; 80] = [0; 80];
        getrandom(&mut rand_bytes)?;

        let mut bytes_vec = Vec::new();
        bytes_vec.try_extend_from_slice(&rand_bytes)?;

        Ok(Self {
            bytes: ByteVec::new(bytes_vec),
        })
    }
}

impl From<Vec<u8>> for Key {
    fn from(value: Vec<u8>) -> Self {
        Self {
            bytes: ByteVec::new(value),
        }
    }
}
