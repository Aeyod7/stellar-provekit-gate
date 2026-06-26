//! Decode RISC0 Groth16 VK decimal limbs (Solidity / `verifier.rs` layout).
use anyhow::{anyhow, Result};
use ark_bn254::{G1Affine, G2Affine};
use ark_serialize::CanonicalDeserialize;
use num_bigint::BigInt;
use std::str::FromStr;

/// Matches `risc0_groth16::from_u256`.
fn from_u256(input: &[u8]) -> Result<[u8; 32]> {
    let mut fixed_array = [0u8; 32];
    let start = std::cmp::max(32, input.len()) - std::cmp::min(32, input.len());
    fixed_array[start..].copy_from_slice(&input[input.len().saturating_sub(32)..]);
    Ok(fixed_array)
}

fn from_u256_decimal(s: &str) -> Result<[u8; 32]> {
    let bytes = BigInt::from_str(s)
        .map_err(|_| anyhow!("bad decimal limb"))?
        .to_bytes_be()
        .1;
    from_u256(&bytes)
}

pub fn risc0_g1_from_limbs(x: &str, y: &str) -> Result<G1Affine> {
    let elem = [from_u256_decimal(x)?, from_u256_decimal(y)?];
    let g1_affine: Vec<u8> = elem[0]
        .iter()
        .rev()
        .chain(elem[1].iter().rev())
        .cloned()
        .collect();
    G1Affine::deserialize_uncompressed(&*g1_affine).map_err(|e| anyhow!("{e}"))
}

fn g2_from_elem(elem: &[[[u8; 32]; 2]; 2]) -> Result<G2Affine> {
    let g2_affine: Vec<u8> = elem[0][1]
        .iter()
        .rev()
        .chain(elem[0][0].iter().rev())
        .chain(elem[1][1].iter().rev())
        .chain(elem[1][0].iter().rev())
        .cloned()
        .collect();
    G2Affine::deserialize_uncompressed(&*g2_affine).map_err(|e| anyhow!("{e}"))
}

/// G2 from RISC0 Solidity-style constants (X1, X2, Y1, Y2).
pub fn risc0_g2_from_sol_const(x1: &str, x2: &str, y1: &str, y2: &str) -> Result<G2Affine> {
    let x_limbs = [from_u256_decimal(x1)?, from_u256_decimal(x2)?];
    let y_limbs = [from_u256_decimal(y1)?, from_u256_decimal(y2)?];
    g2_from_elem(&[x_limbs, y_limbs])
}