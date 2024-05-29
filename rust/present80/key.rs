use kernel::error::code;
use kernel::prelude::*;

pub(crate) struct Key<'a> {
    pub(crate) bytes: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for Key<'a> {
    type Error = Error;

    fn try_from(value: &'a [u8]) -> Result<Self> {
        if value.len() != 10 {
            pr_err!(
                "Supplied PRESENT-80 key is not 80 bits! Len: {} bits.",
                value.len() * 8
            );
            return Err(code::EINVAL);
        }

        Ok(Self { bytes: value })
    }
}

impl From<&Key<'_>> for u128 {
    fn from(value: &Key<'_>) -> Self {
        let mut bytes: [u8; 16] = [0; 16];
        let offset = 6;

        for (i, &byte) in value.bytes.iter().enumerate() {
            bytes[offset + i] = byte;
        }

        u128::from_be_bytes(bytes)
    }
}
