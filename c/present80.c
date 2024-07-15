#include "present80.h"
#include "util.h"

/* Substitution box. */
static const u8 SUBSTITUTION_BOX[] = { 0xC, 0x5, 0x6, 0xB, 0x9, 0x0, 0xA, 0xD,
				       0x3, 0xE, 0xF, 0x8, 0x4, 0x7, 0x1, 0x2 };

/* Permutation box. */
static const u8 PERMUTATION_BOX[] = {
	0,  16, 32, 48, 1,  17, 33, 49, 2,  18, 34, 50, 3,  19, 35, 51,
	4,  20, 36, 52, 5,  21, 37, 53, 6,  22, 38, 54, 7,  23, 39, 55,
	8,  24, 40, 56, 9,  25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59,
	12, 28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63
};

static void generate_round_keys(const u8 *key, u8 *buff)
{
	/* Buffer to store the key. We cannot use the key directly as */
	/* it will be updated in each round. */
	u8 key_reg[PRESENT80_KEY_SIZE];
	u32 temp;

	/* Copy key into the buffer. */
	memcpy(key_reg, key, PRESENT80_KEY_SIZE);
	/* First round key is the left-most 8 bytes of the current key register. */
	memcpy(buff, key_reg, PRESENT80_BLOCK_SIZE);

	for (size_t i = 1; i < PRESENT80_TOTAL_ROUNDS; i++) {
		/* Rotate the bytes to the right by 19 bits. */
		bytes_rotate_right(key_reg, PRESENT80_KEY_SIZE, 19);

		/* Left-most 4 bits are passed through substitution box. */
		key_reg[0] = (SUBSTITUTION_BOX[key_reg[0] >> 4] << 4) |
			     (key_reg[0] & 0x0F);

		temp = cpu_to_be32(i << 15);
		/* The 5 bits at bit location 19, 18, 17, 16, 15 of the key register */
		/* are XORed with the 5-bit round_counter value i. */
		bytes_xor(key_reg + (PRESENT80_KEY_SIZE - 1) - 3, (u8 *)&temp,
			  3);

		/* Store the round key. */
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
	/* Each bytes' upper and lower 4 bits are passed through the substitution box. */
	for (size_t i = 0; i < PRESENT80_BLOCK_SIZE; i++) {
		u8 upper_nibble = (state[i] & 0xF0) >> 4;
		u8 lower_nibble = state[i] & 0x0F;

		state[i] = (SUBSTITUTION_BOX[upper_nibble] << 4) |
			   SUBSTITUTION_BOX[lower_nibble];
	}
}

static void permutation_layer(u8 *state)
{
	/* Buffer to store current state. */
	u8 permutated_state[PRESENT80_BLOCK_SIZE];
	/* Copy the current state. */
	buffer_zeros(permutated_state, PRESENT80_BLOCK_SIZE);

	/* Loop over each byte from right to left. */
	for (size_t _i = 0; _i < PRESENT80_BLOCK_SIZE; _i++) {
		size_t i = (PRESENT80_BLOCK_SIZE - 1) - _i;
		u8 byte = state[i];

		/* Loop over each bit. */
		for (size_t j = 0; j < 8; j++) {
			/* Calculate the bit position. */
			size_t pos = (_i * PRESENT80_BLOCK_SIZE) + j;
			/* Get the bit. */
			u8 bit = (byte & (0x01 << j)) != 0 ? 1 : 0;

			/* Get the new bit position from the permutation box. */
			u8 new_pos = PERMUTATION_BOX[pos];
			/* Calculate the corresponding byte position from the bit position. */
			size_t byte_pos =
				(PRESENT80_BLOCK_SIZE - 1) - (new_pos / 8);
			/* Calculate the bit position relative to the corresponding byte position. */
			size_t bit_pos = new_pos % 8;

			/* Permutate the bit. */
			permutated_state[byte_pos] |= bit << bit_pos;
		}
	}

	/* Copy the permutated state to the original state buffer. */
	memcpy(state, permutated_state, PRESENT80_BLOCK_SIZE);
}

void present80_encrypt(const u8 *key, const u8 *bytes, u8 *out)
{
	/* Buffer to store the state bytes. */
	u8 state[PRESENT80_BLOCK_SIZE];
	/* Buffer to store the round key bytes. */
	u8 round_keys[PRESENT80_BLOCK_SIZE * PRESENT80_TOTAL_ROUNDS];

	generate_round_keys(key, round_keys);
	memcpy(state, bytes, PRESENT80_BLOCK_SIZE);

	for (size_t i = 1; i <= PRESENT80_TOTAL_ROUNDS; i++) {
		add_round_key(state,
			      round_keys + ((i - 1) * PRESENT80_BLOCK_SIZE));

		/* Apply substitution and permutation layer if not last round. */
		if (i != PRESENT80_TOTAL_ROUNDS) {
			substitution_layer(state);
			permutation_layer(state);
		}
	}

	/* Copy the encryption result to the "out". */
	memcpy(out, state, PRESENT80_BLOCK_SIZE);
}
