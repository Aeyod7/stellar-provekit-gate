//! Encode RISC Zero Groth16 seal + public inputs for Soroban BN254 verifier.
use risc0_groth16::Seal;
use risc0_zkp::core::digest::Digest;
use serde::Serialize;

const PROOF_A_LEN: usize = 64;
const PROOF_B_LEN: usize = 128;
const PROOF_C_LEN: usize = 64;

#[derive(Debug, Serialize)]
pub struct SorobanGroth16Invoke {
    pub proof_a_hex: String,
    pub proof_b_hex: String,
    pub proof_c_hex: String,
    pub public_inputs_hex: Vec<String>,
}

/// Split a 32-byte digest into two BN254 field elements (RISC Zero Groth16 layout).
pub fn split_digest_to_fr_bytes(digest: Digest) -> ([u8; 32], [u8; 32]) {
    let bytes: [u8; 32] = digest.into();
    let (a_bytes, b_bytes) = bytes.split_at(16);
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    a[16..].copy_from_slice(a_bytes);
    b[16..].copy_from_slice(b_bytes);
    (a, b)
}

/// Field element bytes as Soroban `Fr::from_bytes` expects (matches risc0-groth16 `fr_from_bytes` input).
pub fn fr_repr_to_soroban_bytes(mut repr_be: [u8; 32]) -> [u8; 32] {
    repr_be.reverse();
    repr_be
}

pub fn public_inputs_from_claim(
    control_root: Digest,
    claim_digest: Digest,
    bn254_control_id: Digest,
) -> [[u8; 32]; 5] {
    let (a0, a1) = split_digest_to_fr_bytes(control_root);
    let (c0, c1) = split_digest_to_fr_bytes(claim_digest);
    let (_id0, id1) = split_digest_to_fr_bytes(bn254_control_id);
    [
        fr_repr_to_soroban_bytes(a0),
        fr_repr_to_soroban_bytes(a1),
        fr_repr_to_soroban_bytes(c0),
        fr_repr_to_soroban_bytes(c1),
        fr_repr_to_soroban_bytes(id1),
    ]
}

/// RISC Zero seal is 256 bytes: G1 A || G2 B || G1 C (big-endian limbs).
pub fn seal_to_soroban_proof(
    seal: &Seal,
) -> ([u8; PROOF_A_LEN], [u8; PROOF_B_LEN], [u8; PROOF_C_LEN]) {
    let raw = seal.to_vec();
    anyhow::ensure!(raw.len() == 256, "unexpected seal size {}", raw.len());
    let mut a = [0u8; PROOF_A_LEN];
    let mut b = [0u8; PROOF_B_LEN];
    let mut c = [0u8; PROOF_C_LEN];
    a.copy_from_slice(&raw[0..PROOF_A_LEN]);
    b.copy_from_slice(&raw[PROOF_A_LEN..PROOF_A_LEN + PROOF_B_LEN]);
    c.copy_from_slice(&raw[PROOF_A_LEN + PROOF_B_LEN..]);
    (a, b, c)
}

pub fn build_soroban_invoke(
    seal: &Seal,
    control_root: Digest,
    claim_digest: Digest,
    bn254_control_id: Digest,
) -> SorobanGroth16Invoke {
    let (a, b, c) = seal_to_soroban_proof(seal).expect("seal layout");
    let inputs = public_inputs_from_claim(control_root, claim_digest, bn254_control_id);
    SorobanGroth16Invoke {
        proof_a_hex: hex::encode(a),
        proof_b_hex: hex::encode(b),
        proof_c_hex: hex::encode(c),
        public_inputs_hex: inputs.iter().map(hex::encode).collect(),
    }
}