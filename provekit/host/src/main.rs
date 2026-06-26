// RISC Zero host: prove private score >= threshold, verify receipt locally.
use methods::{PROVEKIT_GUEST_ELF, PROVEKIT_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ProveGateInput {
    pub secret_score: u32,
    pub threshold: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProveGateOutput {
    pub threshold_met: u32,
    pub commitment: [u8; 32],
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let input = ProveGateInput {
        secret_score: 750,
        threshold: 600,
    };

    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = prover.prove(env, PROVEKIT_GUEST_ELF).unwrap();
    let receipt = prove_info.receipt;

    let output: ProveGateOutput = receipt.journal.decode().unwrap();
    println!("journal threshold_met: {}", output.threshold_met);
    println!("journal commitment: {:?}", output.commitment);
    println!("image_id: {:?}", PROVEKIT_GUEST_ID);

    receipt.verify(PROVEKIT_GUEST_ID).unwrap();
    println!("✓ RISC Zero receipt verified locally");

    // TODO: compress to Groth16 via risc0-groth16 + export calldata for Soroban verifier.
    println!("next: rzup install risc0-groth16 → compress receipt → update contracts/verifier VK");
}
