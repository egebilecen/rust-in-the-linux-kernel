#ifndef _PRESENT80_H
#define _PRESENT80_H

#include <linux/types.h>
#include "types.h"

#define TOTAL_ROUNDS 32
#define KEY_SIZE 10
#define BLOCK_SIZE 8

union present80_key {
	u128 val;
	u8 *bytes;
};

void present80_create_key(const u8 *bytes, union present80_key *key);
void present80_encrypt(const union present80_key *key, const u8 *bytes,
		       u8 *out);

#endif
