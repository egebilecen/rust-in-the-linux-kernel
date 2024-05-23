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

impl TryFrom<[u8; 10]> for Key {
    type Error = Error;

    fn try_from(value: [u8; 10]) -> Result<Self> {
        let mut bytes = Vec::new();
        bytes.try_extend_from_slice(&value)?;

        Ok(Self { bytes })
    }
}

impl From<&Key> for u128 {
    fn from(value: &Key) -> Self {
        let mut bytes: [u8; 16] = [0; 16];
        bytes.clone_from_slice(&value.bytes.as_slice()[..16]);

        u128::from_be_bytes(bytes)
    }
}
