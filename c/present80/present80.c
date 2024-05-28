#include "present80.h"
#include "math.h"
#include "../util.h"

static const u8 SUBSTITUTION_BOX[] = { 0xC, 0x5, 0x6, 0xB, 0x9, 0x0, 0xA, 0xD,
				       0x3, 0xE, 0xF, 0x8, 0x4, 0x7, 0x1, 0x2 };

static const u8 PERMUTATION_BOX[] = {
	0,  16, 32, 48, 1,  17, 33, 49, 2,  18, 34, 50, 3,  19, 35, 51,
	4,  20, 36, 52, 5,  21, 37, 53, 6,  22, 38, 54, 7,  23, 39, 55,
	8,  24, 40, 56, 9,  25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59,
	12, 28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63
};

union plaintext {
	u64 val;
	u8 bytes[64 / 8];
};

static void generate_round_keys(const union present80_key *key, u64 *buff)
{
	u128 key_reg = key->val;

	for (size_t i = 1; i <= PRESENT80_TOTAL_ROUNDS; i++) {
		buff[i - 1] = key_reg >> 16;

		key_reg = rotate_right(key_reg, 19, 80);

		key_reg = (key_reg & ~((u128)0x0F << 76)) |
			  ((u128)SUBSTITUTION_BOX[key_reg >> 76] << 76);

		key_reg = (key_reg ^ (i << 15)) & bit_ones(80);
	}
}

static inline u64 add_round_key(u64 state, u64 key)
{
	return state ^ key;
}

static u64 substitution_layer(u64 state)
{
	u64 substituted_state = 0x00;

	for (size_t i = 0; i < 16; i++) {
		u64 shift = i * 4;
		u64 mask = 0x0F << shift;
		u8 nibble = (state & mask) >> shift;

		substituted_state |= SUBSTITUTION_BOX[nibble] << shift;
	}

	return substituted_state;
}

static u64 permutation_layer(u64 state)
{
	u64 permutated_state = 0x00;

	for (size_t i = 0; i < 8; i++) {
		size_t shift = i * 8;
		u8 byte = (state & (0xFF << shift)) >> shift;

		for (size_t j = 0; j < 8; j++) {
			size_t pos = (i * 8) + j;
			u8 bit = (byte & (0x01 << j)) != 0 ? 1 : 0;
			u8 new_pos = PERMUTATION_BOX[pos];

			permutated_state |= bit << new_pos;
		}
	}

	return permutated_state;
}

static void create_plaintext(const u8 *bytes, union plaintext *plaintext)
{
	buffer_zeroes(plaintext->bytes, sizeof(u64));
	memcpy(plaintext->bytes, bytes, PRESENT80_BLOCK_SIZE);
}

void present80_create_key(const u8 *bytes, union present80_key *key)
{
	buffer_zeroes(key->bytes, sizeof(u128));
	memcpy(key->bytes, bytes, PRESENT80_KEY_SIZE);
}

void present80_encrypt(const union present80_key *key, const u8 *bytes, u8 *out)
{
	union plaintext state;
	u64 round_keys[PRESENT80_TOTAL_ROUNDS];

	create_plaintext(bytes, &state);
	generate_round_keys(key, round_keys);

    pr_info("Round Keys:\n");

    for(size_t i = 0; i < PRESENT80_TOTAL_ROUNDS; i++)
    {
        pr_info("%llu\n", round_keys[i]);
    }

	for (size_t i = 1; i <= PRESENT80_TOTAL_ROUNDS; i++) {
        pr_info("Round %d - state: %llu", i, state.val);

		state.val = add_round_key(state.val, round_keys[i - 1]);
        pr_info("add_round_key - state: %llu\n", state.val);

		if (i != PRESENT80_TOTAL_ROUNDS) {
			state.val = substitution_layer(state.val);
            pr_info("substitution_layer - state: %llu", state.val);
			state.val = permutation_layer(state.val);
            pr_info("permutation_layer - state: %llu", state.val);
		}
	}

	memcpy(out, state.bytes, PRESENT80_BLOCK_SIZE);
}
