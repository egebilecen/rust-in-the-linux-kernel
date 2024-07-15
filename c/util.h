#ifndef _UTIL_H
#define _UTIL_H

#include <linux/string.h>

/* Set the buffer with all zeros. */
#define buffer_zeros(buff, size) memset(buff, 0, size)

/* void print_binary(const u8 *bytes, size_t size, size_t group); */
void bytes_rotate_right(u8 *bytes, size_t size, size_t bit_count);
/* void bytes_shift_right(u8 *bytes, size_t size, size_t shift_count); */
/* void bytes_shift_left(u8 *bytes, size_t size, size_t shift_count); */
void bytes_xor(u8 *first, const u8 *second, size_t size);

#endif
