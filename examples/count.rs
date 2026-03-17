#![allow(missing_docs)]

use std::io::Read;

use ah_ah_ah::{Backend, count_tokens};

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let claude = count_tokens(&input, None, Backend::Claude, None);
    println!("{}", claude.count);
}
