use kernel::pr_cont;
use kernel::prelude::*;

pub(crate) fn print(bytes: &Vec<u8>) {
    pr_info!("Bytes:\n");

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

pub(crate) fn print_block(bytes: &Vec<u8>, width: usize) {
    pr_info!("Bytes:\n");

    if bytes.is_empty() {
        pr_info!("<empty>");
        pr_info!("");
        return;
    }

    pr_info!("| ");

    for (i, b) in bytes.iter().enumerate() {
        pr_cont!("{:02X}", b);

        if (i + 1) % width != 0 {
            pr_cont!(" ");
        } else if i != bytes.len() - 1 {
            pr_cont!(" |");
            pr_info!("| ");
        }

        if i == bytes.len() - 1 {
            pr_cont!(" |");
        }
    }

    pr_info!("");
}
