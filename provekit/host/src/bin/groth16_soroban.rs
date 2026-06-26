//! Prove ProveKit guest, compress to Groth16, export Soroban invoke JSON.
#[path = "../host_stellar.rs"]
mod host_stellar;
#[path = "../soroban_groth16.rs"]
mod soroban_groth16;

use methods::{PROVEKIT_GUEST_ELF, PROVEKIT_GUEST_ID};
use risc0_groth16::{Seal, Verifier};
use risc0_zkvm::{
    default_prover, sha::Digest, sha::Digestible, ExecutorEnv, Groth16ReceiptVerifierParameters,
    ProverOpts, Receipt,
};
use serde::{Deserialize, Serialize};
use soroban_groth16::build_soroban_invoke;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProveGateInput {
    pub secret_score: u32,
    pub threshold: u32,
}

fn groth16_parts(receipt: &Receipt) -> anyhow::Result<(Vec<u8>, Seal, Digest)> {
    let g16 = receipt.inner.groth16().map_err(|_| {
        anyhow::anyhow!(
            "receipt is not Groth16; use ProverOpts::groth16() + risc0-groth16 (Docker or CUDA)"
        )
    })?;
    let seal_bytes = g16.seal.clone();
    let seal = Seal::decode(&seal_bytes)?;
    let claim_digest = g16.claim.digest();
    Ok((seal_bytes, seal, claim_digest))
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();
    let (secret_score, threshold) = if args.len() >= 3 {
        (args[1].parse::<u32>()?, args[2].parse::<u32>()?)
    } else {
        (42u32, 100u32)
    };

    let input = ProveGateInput {
        secret_score,
        threshold,
    };

    let env = ExecutorEnv::builder().write(&input)?.build()?;
    let prover = default_prover();
    let opts = ProverOpts::groth16();
    println!(
        "proving guest score={} threshold={} (Groth16; needs Docker or CUDA)...",
        secret_score, threshold
    );
    let prove_info = prover.prove_with_opts(env, PROVEKIT_GUEST_ELF, &opts)?;
    let receipt = prove_info.receipt;
    receipt.verify(PROVEKIT_GUEST_ID)?;
    println!("✓ receipt verified (image id)");

    let params = Groth16ReceiptVerifierParameters::default();
    let (seal_bytes, seal, claim_digest) = groth16_parts(&receipt)?;
    let verifier = Verifier::new(
        &seal_bytes,
        params.control_root,
        claim_digest,
        params.bn254_control_id,
        &params.verifying_key,
    )?;
    verifier.verify()?;
    println!("✓ Groth16 verified locally against RISC Zero VK");

    let invoke = build_soroban_invoke(
        &seal,
        params.control_root,
        claim_digest,
        params.bn254_control_id,
    )?;
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../artifacts");
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join("soroban_groth16_invoke.json");
    fs::write(&out_path, serde_json::to_string_pretty(&invoke)?)?;
    let seal_path = out_dir.join("groth16_seal.bin");
    fs::write(&seal_path, &seal_bytes)?;
    let claim_path = out_dir.join("groth16_claim_digest.hex");
    fs::write(&claim_path, hex::encode(claim_digest.as_bytes()))?;
    println!("wrote {}", out_path.display());
    Ok(())
}
