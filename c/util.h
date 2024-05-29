#ifndef _UTIL_H
#define _UTIL_H

#include <linux/string.h>

#define buffer_zeroes(buff, size) memset(buff, 0, size)

#define MASK_1_BITS 0x01
#define MASK_2_BITS 0x03
#define MASK_3_BITS 0x07
#define MASK_4_BITS 0x0F
#define MASK_5_BITS 0x1F
#define MASK_6_BITS 0x3F
#define MASK_7_BITS 0x7F
#define MASK_8_BITS 0xFF

void print_binary(const u8 *bytes, size_t size, size_t group);
void bytes_rotate_right(u8 *bytes, size_t size, size_t bit_count);
void bytes_shift_right(u8 *bytes, size_t size, size_t shift_count);
void bytes_shift_left(u8 *bytes, size_t size, size_t shift_count);
void bytes_xor(u8 *first, u8 *second, size_t size);

#endif
