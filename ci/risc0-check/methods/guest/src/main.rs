#![no_main]
#![no_std] // std support is experimental

use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

fn main() {
    // read the input
    let input: u32 = env::read();

    // write public output to the journal
    env::commit(&input);
}
