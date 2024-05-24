use kernel::error::code;
use kernel::pr_cont;
use kernel::prelude::*;
use kernel::random::getrandom;

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
pub(crate) fn print(bytes: &[u8]) {
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
pub(crate) fn print_block(bytes: &[u8], width: usize) {
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
