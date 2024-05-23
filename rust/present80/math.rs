fn bit_ones(size: u128) -> u128 {
    let mut num = 0x00;

    for i in 0..size {
        num |= 1 << i;
    }

    num
}

pub(crate) fn rotate_right(num: u128, bits: u128, width: u128) -> u128 {
    let bits = if bits > width { bits % width } else { bits };

    let preserved_bits = num & bit_ones(bits);
    let bits_to_shift = num & (bit_ones(width - bits) << bits);

    (bits_to_shift >> bits) | (preserved_bits << (width - bits))
}
