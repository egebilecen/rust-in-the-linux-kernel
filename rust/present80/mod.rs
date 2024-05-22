use crate::present80::key::Key;
use crate::present80::math::rotate_right;
use core::todo;
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

pub(crate) struct Present80 {
    pub(crate) key: Key,
}

impl Present80 {
    pub(crate) fn new(key: Key) -> Self {
        Self { key }
    }

    fn generate_round_keys(&self) -> [u64; TOTAL_ROUNDS] {
        let mut round_keys: [u64; TOTAL_ROUNDS] = [0; TOTAL_ROUNDS];
        let mut key_reg = u128::from(&self.key);

        for i in 1..=TOTAL_ROUNDS {
            let round_key = (key_reg >> 16) as u64;
            round_keys[i - 1] = round_key;

            key_reg = rotate_right(key_reg, 19, 80);

            // TODO: Optimize the shifts by using constant hex value.
            key_reg = (key_reg & !(0x0F << 76))
                | ((SUBSTITUTION_BOX[(key_reg >> 76) as usize] as u128) << 76);

            key_reg ^= 1 << 15;
        }

        round_keys
    }

    #[inline]
    fn add_round_key(&self, state: u64, key: u64) -> u64 {
        state ^ key
    }

    fn substitution_layer(&self, state: u64) -> u64 {
        todo!()
    }

    fn permutation_layer(&self, state: u64) -> u64 {
        todo!()
    }

    pub(crate) fn encrypt(&self, bytes: &[u8; 64]) -> Result<&[u8; 64]> {
        todo!()
    }
}
