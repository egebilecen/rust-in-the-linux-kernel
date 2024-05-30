use crate::present80::key::Key;
use crate::present80::math::{bit_ones, rotate_right};
use kernel::error::code;
use kernel::prelude::*;

pub(crate) mod key;
pub(crate) mod math;
pub(crate) mod util;

const TOTAL_ROUNDS: usize = 32;

const SUBSTITUTION_BOX: &[u8] = &[
    0xC, 0x5, 0x6, 0xB, 0x9, 0x0, 0xA, 0xD, 0x3, 0xE, 0xF, 0x8, 0x4, 0x7, 0x1, 0x2,
];

const PERMUTATION_BOX: &[u8] = &[
    0, 16, 32, 48, 1, 17, 33, 49, 2, 18, 34, 50, 3, 19, 35, 51, 4, 20, 36, 52, 5, 21, 37, 53, 6,
    22, 38, 54, 7, 23, 39, 55, 8, 24, 40, 56, 9, 25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59, 12,
    28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63,
];

pub(crate) struct Present80<'a> {
    pub(crate) key: Key<'a>,
}

impl<'a> Present80<'a> {
    pub(crate) fn new(key: Key<'a>) -> Self {
        Self { key }
    }

    fn generate_round_keys(&self) -> Result<[u64; TOTAL_ROUNDS]> {
        let mut round_keys: [u64; TOTAL_ROUNDS] = [0; TOTAL_ROUNDS];
        let mut key_reg = u128::from(&self.key);

        round_keys[0] = (key_reg >> 16) as u64;

        for (i, round_key) in round_keys.iter_mut().enumerate().take(TOTAL_ROUNDS).skip(1) {
            key_reg = rotate_right(key_reg, 19, 80);

            key_reg = (key_reg & !(0x0F << 76))
                | ((SUBSTITUTION_BOX[(key_reg >> 76) as usize] as u128) << 76);

            key_reg ^= u128::try_from(i)? << 15;

            *round_key = (key_reg >> 16) as u64;
        }

        Ok(round_keys)
    }

    #[inline]
    fn add_round_key(&self, state: u64, key: u64) -> u64 {
        state ^ key
    }

    fn substitution_layer(&self, state: u64) -> u64 {
        let mut substituted_state: u64 = 0x00;

        for i in 0..16 {
            let shift = i * 4;
            let mask = 0x0F << shift;
            let nibble = (state & mask) >> shift;

            substituted_state |= (SUBSTITUTION_BOX[nibble as usize] as u64) << shift;
        }

        substituted_state
    }

    fn permutation_layer(&self, state: u64) -> u64 {
        let mut permutated_state: u64 = 0x00;

        for i in 0..8 {
            let shift = i * 8;
            let byte = (state & (0xFF << shift)) >> shift;

            for j in 0..8 {
                let pos = (i * 8) + j;
                let bit = if (byte & (0x01 << j)) != 0 { 1 } else { 0 };
                let new_pos = PERMUTATION_BOX[pos];

                permutated_state |= bit << new_pos;
            }
        }

        permutated_state
    }

    pub(crate) fn encrypt(&self, bytes: &[u8]) -> Result<[u8; 8]> {
        if bytes.len() != 8 {
            pr_err!(
                "Given bits doesn't match with the block size of 64 bits! Len: {}",
                bytes.len() * 8
            );
            return Err(code::EINVAL);
        }

        let mut fixed_bytes: [u8; 8] = [0; 8];
        fixed_bytes.copy_from_slice(&bytes[..8]);

        let mut state = u64::from_be_bytes(fixed_bytes);
        let round_keys = self.generate_round_keys()?;

        for i in 1..=TOTAL_ROUNDS {
            state = self.add_round_key(state, round_keys[i - 1]);

            if i != TOTAL_ROUNDS {
                state = self.substitution_layer(state);
                state = self.permutation_layer(state);
            }
        }

        Ok(state.to_be_bytes())
    }
}
