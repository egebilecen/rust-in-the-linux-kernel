#ifndef _PRESENT80_H
#define _PRESENT80_H

#include <linux/types.h>

/* Total number of encryption rounds. */
#define PRESENT80_TOTAL_ROUNDS 32
/* Size of the key. */
#define PRESENT80_KEY_SIZE 10
/* Size of the block. */
#define PRESENT80_BLOCK_SIZE 8

void present80_encrypt(const u8 *key, const u8 *bytes, u8 *out);

#endif
