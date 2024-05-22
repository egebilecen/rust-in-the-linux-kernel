use crate::rand_bytes;
use kernel::prelude::*;

pub(crate) struct Key {
    pub(crate) bytes: Vec<u8>,
}

impl Key {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            bytes: rand_bytes(10)?,
        })
    }
}

impl From<&Key> for u64 {
    fn from(value: &Key) -> Self {
        let mut bytes: [u8; 8] = [0; 8];
        bytes.clone_from_slice(&value.bytes.as_slice()[..8]);

        u64::from_be_bytes(bytes)
    }
}

impl From<&Key> for u128 {
    fn from(value: &Key) -> Self {
        let mut bytes: [u8; 16] = [0; 16];
        bytes.clone_from_slice(&value.bytes.as_slice()[..16]);

        u128::from_be_bytes(bytes)
    }
}
