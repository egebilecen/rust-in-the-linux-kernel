use self::util::{bytes_rotate_right, bytes_xor};
use kernel::error::code;
use kernel::prelude::*;

pub(crate) mod util;

pub(crate) const KEY_SIZE: usize = 10;
pub(crate) const BLOCK_SIZE: usize = 8;
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
    pub(crate) key: &'a [u8; KEY_SIZE],
}

impl<'a> Present80<'a> {
    pub(crate) fn new(key: &'a [u8; KEY_SIZE]) -> Self {
        Self { key }
    }

    fn generate_round_keys(&self) -> [u8; BLOCK_SIZE * TOTAL_ROUNDS] {
        let mut round_keys: [u8; BLOCK_SIZE * TOTAL_ROUNDS] = [0; BLOCK_SIZE * TOTAL_ROUNDS];
        let mut key_reg = *self.key;

        round_keys[..BLOCK_SIZE].copy_from_slice(&key_reg[..BLOCK_SIZE]);

        for i in 1..TOTAL_ROUNDS {
            bytes_rotate_right(&mut key_reg, 19);

            key_reg[0] = (SUBSTITUTION_BOX[(key_reg[0] >> 4) as usize] << 4) | (key_reg[0] & 0x0F);

            let round_counter = &(i.checked_shl(15).unwrap_or(0) as u32).to_be_bytes()[..3];

            let start_index = (KEY_SIZE - 1) - 3;
            let end_index = start_index + 3;
            bytes_xor(&mut key_reg[start_index..end_index], round_counter);

            let start_index = BLOCK_SIZE * i;
            let end_index = start_index + BLOCK_SIZE;
            round_keys[start_index..end_index].copy_from_slice(&key_reg[..BLOCK_SIZE]);
        }

        round_keys
    }

    #[inline]
    fn add_round_key(&self, state: &mut [u8; BLOCK_SIZE], key: &[u8; BLOCK_SIZE]) {
        bytes_xor(state, key)
    }

    fn substitution_layer(&self, state: &mut [u8; BLOCK_SIZE]) {
        for b in state.iter_mut() {
            let upper_nibble = (*b & 0xF0) >> 4;
            let lower_nibble = *b & 0x0F;

            *b = (SUBSTITUTION_BOX[upper_nibble as usize] << 4)
                | SUBSTITUTION_BOX[lower_nibble as usize];
        }
    }

    fn permutation_layer(&self, state: &mut [u8; BLOCK_SIZE]) {
        let mut permutated_state: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];

        for i in (0..BLOCK_SIZE).rev() {
            let byte = state[i];

            for j in 0..8 {
                let bit: u8 = if byte & (0x01 << j) != 0 { 1 } else { 0 };
                let bit_pos = ((BLOCK_SIZE - 1 - i) * BLOCK_SIZE) + j;

                let new_bit_pos = PERMUTATION_BOX[bit_pos];
                let byte_pos = (BLOCK_SIZE - 1) - ((new_bit_pos / 8) as usize);
                let bit_shift = new_bit_pos % 8;

                permutated_state[byte_pos] |= bit << bit_shift;
            }
        }

        state.copy_from_slice(&permutated_state);
    }

    pub(crate) fn encrypt(&self, bytes: &[u8; BLOCK_SIZE]) -> Result<[u8; BLOCK_SIZE]> {
        let mut state: [u8; BLOCK_SIZE] = *bytes;

        let round_keys = self.generate_round_keys();

        for i in 1..=TOTAL_ROUNDS {
            let start_index = (i - 1) * BLOCK_SIZE;
            let end_index = start_index + BLOCK_SIZE;
            let round_key: [u8; BLOCK_SIZE] =
                if let Ok(val) = round_keys[start_index..end_index].try_into() {
                    val
                } else {
                    return Err(code::ENOMEM);
                };

            self.add_round_key(&mut state, &round_key);

            if i != TOTAL_ROUNDS {
                self.substitution_layer(&mut state);
                self.permutation_layer(&mut state);
            }
        }

        Ok(state)
    }
}
