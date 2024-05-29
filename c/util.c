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

void bytes_rotate_right(const u8 *bytes, size_t size, size_t bits, u8 *out)
{
	size_t size_in_bits = size * 8;
	u64 preserve_mask;
	size_t rel_shift;

	if (bits > size_in_bits)
		bits = bits % size_in_bits;

	preserve_mask = bits == 1 ? MASK_1_BITS :
			bits == 2 ? MASK_2_BITS :
			bits == 3 ? MASK_3_BITS :
			bits == 4 ? MASK_4_BITS :
			bits == 5 ? MASK_5_BITS :
			bits == 6 ? MASK_6_BITS :
			bits == 7 ? MASK_7_BITS :
				    MASK_8_BITS;

	rel_shift = bits % 8;

	for (size_t i = 0; i < size; i++) {
		const u8 b = bytes[i];
		const u8 *nb = i == size - 1 ? bytes : (bytes + (i + 1));

		u8 *nob = i == size - 1 ? out : (out + (i + 1));
		*nob = ((b & preserve_mask) << (8 - rel_shift)) |
		       (*nb >> rel_shift);
	}
}
