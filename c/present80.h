#ifndef _PRESENT80_H
#define _PRESENT80_H

#include <linux/types.h>

#define PRESENT80_TOTAL_ROUNDS 32
#define PRESENT80_KEY_SIZE 10
#define PRESENT80_BLOCK_SIZE 8

void present80_encrypt(const u8 *key, const u8 *bytes, u8 *out);

#endif
