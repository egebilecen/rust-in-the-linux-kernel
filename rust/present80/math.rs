pub(crate) fn bit_ones(size: u128) -> u128 {
    let mut num = 0x00;

    for i in 0..size {
        num |= 1 << i;
    }

    num
}

pub(crate) fn rotate_right(num: u128, bits: u128, width: u128) -> u128 {
    const MAX_WIDTH: u128 = 128;

    let width = if width > MAX_WIDTH { MAX_WIDTH } else { width };

    let bits = if bits > MAX_WIDTH {
        bits % MAX_WIDTH
    } else {
        bits
    };

    ((num & bit_ones(bits)) << (width - bits)) | (num >> bits)
}
