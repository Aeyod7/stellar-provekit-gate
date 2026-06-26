//! Off-chain check: RISC0 Groth16 verifier accepts seal + claim.
use risc0_groth16::{verifying_key, Verifier};
use risc0_zkvm::Groth16ReceiptVerifierParameters;
use std::fs;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let seal = fs::read(root.join("artifacts/groth16_seal.bin"))?;
    let claim_hex = fs::read_to_string(root.join("artifacts/groth16_claim_digest.hex"))?;
    let claim_hex = claim_hex.trim().trim_start_matches("0x");
    let claim_bytes = hex::decode(claim_hex)?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(claim_bytes.as_slice());
    let claim_digest = risc0_zkvm::sha::Digest::from(arr);

    let params = Groth16ReceiptVerifierParameters::default();
    let control_root = params.control_root;
    let bn254_control_id = params.bn254_control_id;
    let vk = verifying_key();

    let v = Verifier::new(&seal, control_root, claim_digest, bn254_control_id, &vk)?;
    v.verify()?;
    println!("risc0_groth16::Verifier::verify OK");
    Ok(())
}