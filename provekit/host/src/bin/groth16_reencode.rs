//! Re-export Soroban invoke JSON from saved groth16_seal.bin (no Docker reprove).
use provekit_host::soroban_groth16::build_soroban_invoke;
use risc0_groth16::Seal;
use risc0_zkvm::Groth16ReceiptVerifierParameters;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Demo policy: secret_score=42, threshold=100 → threshold_met=0.
fn demo_policy_commitment() -> [u8; 32] {
    let score: u32 = 42;
    let threshold: u32 = 100;
    let threshold_met: u32 = 0;
    let mut hasher = Sha256::new();
    hasher.update(score.to_le_bytes());
    hasher.update(threshold.to_le_bytes());
    hasher.update(threshold_met.to_le_bytes());
    hasher.finalize().into()
}

fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let seal_path = root.join("artifacts/groth16_seal.bin");
    let seal_bytes = fs::read(&seal_path)?;
    let seal = Seal::decode(&seal_bytes)?;
    let params = Groth16ReceiptVerifierParameters::default();
    let claim_path = root.join("artifacts/groth16_claim_digest.hex");
    let claim_digest = if claim_path.exists() {
        let h = fs::read_to_string(&claim_path)?.trim().to_string();
        let bytes = hex::decode(h)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes.as_slice());
        risc0_zkvm::sha::Digest::from(arr)
    } else {
        anyhow::bail!(
            "missing {}; re-run provekit-groth16-soroban once to create seal + claim sidecar",
            claim_path.display()
        );
    };

    let invoke = build_soroban_invoke(
        &seal,
        params.control_root,
        claim_digest,
        params.bn254_control_id,
    )?;
    let out = root.join("artifacts/soroban_groth16_invoke.json");
    fs::write(&out, serde_json::to_string_pretty(&invoke)?)?;
    println!("wrote {}", out.display());

    let policy_hex = hex::encode(demo_policy_commitment());
    let policy_path = root.join("artifacts/policy_commitment.hex");
    fs::write(&policy_path, format!("{policy_hex}\n"))?;
    println!("wrote {}", policy_path.display());
    Ok(())
}