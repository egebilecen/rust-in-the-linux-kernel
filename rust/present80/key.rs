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

