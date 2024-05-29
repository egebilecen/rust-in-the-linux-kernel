#include "util.h"

void print_binary(const u8 *bytes, size_t size, size_t group)
{
	size_t group_counter = 0;
	pr_info("");

	for (size_t i = 0; i < size; i++) {
		u8 b = bytes[i];

		for (size_t j = 0; j < 8; j++) {
			size_t mask = 0x01 << (7 - j);
			pr_cont("%d", (b & mask) > 0 ? 1 : 0);

			group_counter++;

			if (group_counter == group) {
				group_counter = 0;
				pr_cont(" ");
			}
		}
	}

	pr_cont("\n");
}

void bytes_rotate_right(u8 *bytes, size_t size, size_t bit_count)
{
	size_t size_in_bits = size * 8;

	if (bit_count > size_in_bits)
		bit_count = bit_count % size_in_bits;

	while (bit_count > 0) {
		size_t shift_count = bit_count % 8 == 0 ? 8 : bit_count % 8;
		u64 preserve_mask = shift_count == 1 ? MASK_1_BITS :
				    bit_count == 2   ? MASK_2_BITS :
				    bit_count == 3   ? MASK_3_BITS :
				    bit_count == 4   ? MASK_4_BITS :
				    bit_count == 5   ? MASK_5_BITS :
				    bit_count == 6   ? MASK_6_BITS :
				    bit_count == 7   ? MASK_7_BITS :
						       MASK_8_BITS;

		u8 fpb = bytes[0];
		u8 pb;
		u8 npb;

		for (size_t i = 0; i < size; i++) {
			u8 *b = bytes + i;
			u8 *nb = i == size - 1 ? bytes : (bytes + (i + 1));

			if (i == 0) {
				pb = *b;
				npb = *nb;
			} else if (i == size - 1) {
				pb = npb;
				npb = fpb;
			} else {
				pb = npb;
				npb = *nb;
			}

			*nb = ((pb & preserve_mask) << (8 - shift_count)) |
			      (npb >> shift_count);
		}

		bit_count -= shift_count;
	}
}
