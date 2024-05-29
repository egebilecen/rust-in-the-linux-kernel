#include "present80.h"
#include "../util.h"

/* TODO: Move present80 files to the main folder. */
/* TODO: Change zeroes to zeros. */
static const u8 SUBSTITUTION_BOX[] = { 0xC, 0x5, 0x6, 0xB, 0x9, 0x0, 0xA, 0xD,
				       0x3, 0xE, 0xF, 0x8, 0x4, 0x7, 0x1, 0x2 };

static const u8 PERMUTATION_BOX[] = {
	0,  16, 32, 48, 1,  17, 33, 49, 2,  18, 34, 50, 3,  19, 35, 51,
	4,  20, 36, 52, 5,  21, 37, 53, 6,  22, 38, 54, 7,  23, 39, 55,
	8,  24, 40, 56, 9,  25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59,
	12, 28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63
};

static void generate_round_keys(const u8 *key, u8 *buff)
{
	u8 key_reg[PRESENT80_KEY_SIZE];
	u32 temp;

	memcpy(key_reg, key, PRESENT80_KEY_SIZE);
	memcpy(buff, key_reg, PRESENT80_BLOCK_SIZE);

	for (size_t i = 1; i < PRESENT80_TOTAL_ROUNDS; i++) {
		bytes_rotate_right(key_reg, PRESENT80_KEY_SIZE, 19);

		key_reg[0] = (SUBSTITUTION_BOX[key_reg[0] >> 4] << 4) |
			     (key_reg[0] & 0x0F);

		temp = cpu_to_be32(i << 15);
		bytes_xor(key_reg + (PRESENT80_KEY_SIZE - 1) - 3, (u8 *)&temp,
			  3);

		memcpy(buff + (PRESENT80_BLOCK_SIZE * i), key_reg,
		       PRESENT80_BLOCK_SIZE);
	}
}

static inline void add_round_key(u8 *state, const u8 *key)
{
	bytes_xor(state, key, PRESENT80_BLOCK_SIZE);
}

static void substitution_layer(u8 *state)
{
	for (size_t i = 0; i < PRESENT80_BLOCK_SIZE; i++) {
		u8 upper_nibble = (state[i] & 0xF0) >> 4;
		u8 lower_nibble = state[i] & 0x0F;

		state[i] = (SUBSTITUTION_BOX[upper_nibble] << 4) |
			   SUBSTITUTION_BOX[lower_nibble];
	}
}

static void permutation_layer(u8 *state)
{
	u8 permutated_state[PRESENT80_BLOCK_SIZE];
	buffer_zeroes(permutated_state, PRESENT80_BLOCK_SIZE);

	for (size_t _i = 0; _i < PRESENT80_BLOCK_SIZE; _i++) {
		size_t i = (PRESENT80_BLOCK_SIZE - 1) - _i;
		u8 byte = state[i];

		for (size_t j = 0; j < 8; j++) {
			size_t pos = (_i * PRESENT80_BLOCK_SIZE) + j;
			u8 bit = (byte & (0x01 << j)) != 0 ? 1 : 0;

			u8 new_pos = PERMUTATION_BOX[pos];
			size_t byte_pos =
				(PRESENT80_BLOCK_SIZE - 1) - (new_pos / 8);
			size_t bit_pos = new_pos % 8;

			permutated_state[byte_pos] |= bit << bit_pos;
		}
	}

	memcpy(state, permutated_state, PRESENT80_BLOCK_SIZE);
}

void present80_encrypt(const u8 *key, const u8 *bytes, u8 *out)
{
	u8 state[PRESENT80_BLOCK_SIZE];
	u8 round_keys[8 * PRESENT80_TOTAL_ROUNDS];

	generate_round_keys(key, round_keys);
	memcpy(state, bytes, PRESENT80_BLOCK_SIZE);

	for (size_t i = 1; i <= PRESENT80_TOTAL_ROUNDS; i++) {
		add_round_key(state,
			      round_keys + ((i - 1) * PRESENT80_BLOCK_SIZE));

		if (i != PRESENT80_TOTAL_ROUNDS) {
			substitution_layer(state);
			permutation_layer(state);
		}
	}

	memcpy(out, state, PRESENT80_BLOCK_SIZE);
}
