#include "math.h"

u128 bit_ones(u128 size)
{
	u128 num = 0x00;

	for (size_t i = 0; i < size; i++)
		num |= 1 << i;

	return num;
}

u128 rotate_right(u128 num, u8 bits, u8 width)
{
    const u8 MAX_WIDTH = 128;

    if(width > MAX_WIDTH)
        width = MAX_WIDTH;

    if(bits > width)
        bits = bits % width;

    return ((num & bit_ones(bits)) << (width - bits)) | (num >> bits);
}
