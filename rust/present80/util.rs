use kernel::error::code;
use kernel::pr_cont;
use kernel::prelude::*;
use kernel::random::getrandom;

pub(crate) fn bytes_rotate_right(bytes: &mut [u8], bit_count: usize) {
    if bytes.is_empty() {
        return;
    }

    let size = bytes.len();
    let size_in_bits = size * 8;
    let mut bit_count = bit_count;

    if bit_count > size_in_bits {
        bit_count %= size_in_bits;
    };

    while bit_count > 0 {
        let shift_count = if bit_count % 8 == 0 { 8 } else { bit_count % 8 };
        let preserve_mask = match shift_count {
            1 => 0x01,
            2 => 0x03,
            3 => 0x07,
            4 => 0x0F,
            5 => 0x1F,
            6 => 0x3F,
            7 => 0x7F,
            _ => 0xFF,
        };

        let fpb = bytes[0];
        let mut pb;
        let mut npb = 0;

        for ib in 0..size {
            let inb = if ib == size - 1 { 0 } else { ib + 1 };

            if ib == 0 {
                pb = bytes[ib];
                npb = bytes[inb];
            } else if ib == size - 1 {
                pb = npb;
                npb = fpb;
            } else {
                pb = npb;
                npb = bytes[inb];
            }

            bytes[inb] = ((pb & preserve_mask).checked_shl((8 - shift_count) as u32)).unwrap_or(0)
                | (npb.checked_shr(shift_count as u32).unwrap_or(0));
        }

        bit_count = bit_count.saturating_sub(shift_count);
    }
}

pub(crate) fn bytes_xor(first: &mut [u8], second: &[u8]) {
    if first.len() != second.len() {
        return;
    }

    for (i, b) in first.iter_mut().enumerate() {
        *b ^= second[i];
    }
}

// Functions below were used for debug purposes. They are not commented out unlike in the C module.
// This is because Rust will not include these functions in the final object file output as they are 
// not used in the code.
#[allow(dead_code)]
pub(crate) fn rand_bytes(size: usize) -> Result<Vec<u8>> {
    const MAX_SIZE: usize = 128;

    if size > MAX_SIZE {
        pr_err!(
            "rand_bytes() - Argument `size` ({}) cannot be bigger than `MAX_SIZE` ({})",
            size,
            MAX_SIZE
        );
        return Err(code::EINVAL);
    }

    let mut rand_bytes: [u8; MAX_SIZE] = [0; MAX_SIZE];
    getrandom(&mut rand_bytes[..size])?;

    let mut bytes = Vec::new();
    bytes.try_extend_from_slice(&rand_bytes[..size])?;

    Ok(bytes)
}

#[allow(dead_code)]
pub(crate) fn print_hex(bytes: &[u8]) {
    pr_info!("Bytes ({})\n", bytes.len());

    if bytes.is_empty() {
        pr_info!("<empty>");
        pr_info!("");
        return;
    }

    pr_info!("");

    for (i, b) in bytes.iter().enumerate() {
        pr_cont!("{:02X}", b);

        if i != bytes.len() - 1 {
            pr_cont!(" ");
        }
    }

    pr_info!("");
}

#[allow(dead_code)]
pub(crate) fn print_hex_block(bytes: &[u8], width: usize) {
    pr_info!("Bytes ({})\n", bytes.len());

    if bytes.is_empty() {
        pr_info!("<empty>");
        pr_info!("");
        return;
    }

    pr_info!("| ");

    for (i, b) in bytes.iter().enumerate() {
        pr_cont!("{:02X}", b);

        if (i + 1) % width != 0 && i != bytes.len() - 1 {
            pr_cont!(" ");
        } else if i != bytes.len() - 1 {
            pr_cont!(" |");
            pr_info!("| ");
        }
    }

    if bytes.len() % width != 0 {
        for _ in 0..(width - (bytes.len() % width)) {
            pr_cont!(" --");
        }

        pr_cont!(" |");
    } else {
        pr_cont!(" |");
    }

    pr_info!("");
}
