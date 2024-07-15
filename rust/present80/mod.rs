use self::util::{bytes_rotate_right, bytes_xor};
use kernel::error::code;
use kernel::prelude::*;

pub(crate) mod util;

// Size of the key.
pub(crate) const KEY_SIZE: usize = 10;
// Size of the block.
pub(crate) const BLOCK_SIZE: usize = 8;
// Total number of encryption rounds.
const TOTAL_ROUNDS: usize = 32;

// Substitution box.
const SUBSTITUTION_BOX: &[u8] = &[
    0xC, 0x5, 0x6, 0xB, 0x9, 0x0, 0xA, 0xD, 0x3, 0xE, 0xF, 0x8, 0x4, 0x7, 0x1, 0x2,
];

// Permutation box.
const PERMUTATION_BOX: &[u8] = &[
    0, 16, 32, 48, 1, 17, 33, 49, 2, 18, 34, 50, 3, 19, 35, 51, 4, 20, 36, 52, 5, 21, 37, 53, 6,
    22, 38, 54, 7, 23, 39, 55, 8, 24, 40, 56, 9, 25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59, 12,
    28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63,
];

// PRESENT-80 cipher struct.
pub(crate) struct Present80<'a> {
    pub(crate) key: &'a [u8; KEY_SIZE],
}

impl<'a> Present80<'a> {
    pub(crate) fn new(key: &'a [u8; KEY_SIZE]) -> Self {
        Self { key }
    }

    fn generate_round_keys(&self) -> [u8; BLOCK_SIZE * TOTAL_ROUNDS] {
        // Buffer to store the round keys.
        let mut round_keys: [u8; BLOCK_SIZE * TOTAL_ROUNDS] = [0; BLOCK_SIZE * TOTAL_ROUNDS];
        // Buffer to store the key. We cannot use the key directly as
        // it will be updated in each round.
        let mut key_reg = *self.key;

        // First round key is the left-most 8 bytes of the current key register.
        round_keys[..BLOCK_SIZE].copy_from_slice(&key_reg[..BLOCK_SIZE]);

        for i in 1..TOTAL_ROUNDS {
            // Rotate the bytes to the right by 19 bits.
            bytes_rotate_right(&mut key_reg, 19);

            // Left-most 4 bits are passed through substitution box.
            key_reg[0] = (SUBSTITUTION_BOX[(key_reg[0] >> 4) as usize] << 4) | (key_reg[0] & 0x0F);

            // Round counter.
            let round_counter = &(i.checked_shl(15).unwrap_or(0) as u32).to_be_bytes()[..3];

            // The 5 bits at bit location 19, 18, 17, 16, 15 of the key register
            // are XORed with the 5-bit round_counter value i.
            let start_index = (KEY_SIZE - 1) - 3;
            let end_index = start_index + 3;
            bytes_xor(&mut key_reg[start_index..end_index], round_counter);

            // Store the round key.
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
        // Each bytes' upper and lower 4 bits are passed through the substitution box.
        for b in state.iter_mut() {
            let upper_nibble = (*b & 0xF0) >> 4;
            let lower_nibble = *b & 0x0F;

            *b = (SUBSTITUTION_BOX[upper_nibble as usize] << 4)
                | SUBSTITUTION_BOX[lower_nibble as usize];
        }
    }

    fn permutation_layer(&self, state: &mut [u8; BLOCK_SIZE]) {
        // Buffer to store current state.
        let mut permutated_state: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];

        // Loop over each byte from right to left.
        for i in (0..BLOCK_SIZE).rev() {
            let byte = state[i];

            // Loop over each bit.
            for j in 0..8 {
                // Get the bit.
                let bit: u8 = if byte & (0x01 << j) != 0 { 1 } else { 0 };
                // Calculate the bit position.
                let bit_pos = ((BLOCK_SIZE - 1 - i) * BLOCK_SIZE) + j;

                // Get the new bit position from the permutation box.
                let new_bit_pos = PERMUTATION_BOX[bit_pos];
                // Calculate the corresponding byte position from the bit position.
                let byte_pos = (BLOCK_SIZE - 1) - ((new_bit_pos / 8) as usize);
                // Calculate the bit position relative to the corresponding byte position.
                let bit_shift = new_bit_pos % 8;

                // Permutate the bit.
                permutated_state[byte_pos] |= bit << bit_shift;
            }
        }

        // Copy the permutated state to the original state buffer.
        state.copy_from_slice(&permutated_state);
    }

    pub(crate) fn encrypt(&self, bytes: &[u8; BLOCK_SIZE]) -> Result<[u8; BLOCK_SIZE]> {
        // Store the state bytes.
        let mut state: [u8; BLOCK_SIZE] = *bytes;

        // Store the round key bytes.
        let round_keys = self.generate_round_keys();

        for i in 1..=TOTAL_ROUNDS {
            // Get the round key.
            let start_index = (i - 1) * BLOCK_SIZE;
            let end_index = start_index + BLOCK_SIZE;
            let round_key: [u8; BLOCK_SIZE] =
                if let Ok(val) = round_keys[start_index..end_index].try_into() {
                    val
                } else {
                    return Err(code::ENOMEM);
                };

            self.add_round_key(&mut state, &round_key);

            // Apply substitution and permutation layer if not last round.
            if i != TOTAL_ROUNDS {
                self.substitution_layer(&mut state);
                self.permutation_layer(&mut state);
            }
        }

        Ok(state)
    }
}
