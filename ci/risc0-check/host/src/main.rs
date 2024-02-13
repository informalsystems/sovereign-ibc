use methods::{GUEST_SOV_IBC_ELF, GUEST_SOV_IBC_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

fn main() {
    // An example input
    let input: u32 = 15 * u32::pow(2, 27) + 1;
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove(env, GUEST_SOV_IBC_ELF).unwrap();

    // An example retrieval of the output from the receipt journal.
    let _output: u32 = receipt.journal.decode().unwrap();

    receipt.verify(GUEST_SOV_IBC_ID).unwrap();
}
