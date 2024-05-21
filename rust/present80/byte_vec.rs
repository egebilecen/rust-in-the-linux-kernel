use kernel::pr_cont;
use kernel::prelude::*;

pub(crate) struct ByteVec {
    vec: Vec<u8>,
}

impl ByteVec {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self { vec: bytes }
    }

    pub(crate) fn get(&self) -> &Vec<u8> {
        &self.vec
    }

    pub(crate) fn print(&self) {
        pr_info!("Bytes:\n");

        if self.vec.is_empty() {
            pr_info!("<empty>");
            pr_info!("");
            return;
        }

        pr_info!("");

        for (i, b) in self.vec.iter().enumerate() {
            pr_cont!("{:02X}", b);

            if i != self.vec.len() - 1 {
                pr_cont!(" ");
            }
        }

        pr_info!("");
    }

    pub(crate) fn print_block(&self, width: usize) {
        pr_info!("Bytes:\n");

        if self.vec.is_empty() {
            pr_info!("<empty>");
            pr_info!("");
            return;
        }

        pr_info!("| ");

        for (i, b) in self.vec.iter().enumerate() {
            pr_cont!("{:02X}", b);

            if (i + 1) % width != 0 {
                pr_cont!(" ");
            } else if i != self.vec.len() - 1 {
                pr_cont!(" |");
                pr_info!("| ");
            }

            if i == self.vec.len() - 1 {
                pr_cont!(" |");
            }
        }

        pr_info!("");
    }
}
