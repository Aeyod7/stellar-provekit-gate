//! Soroban BN254 layout (Ethereum-compatible), matching soroban-env-host `bn254.rs`.

use anyhow::{anyhow, Context, Result};
use ark_bn254::{Fq, Fq2, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use risc0_groth16::Seal;
use risc0_zkvm::sha::Digest;
use serde::Serialize;

fn risc0_g1_from_seal(a: &[Vec<u8>]) -> Result<G1Affine> {
    if a.len() != 2 {
        return Err(anyhow!("Malformed G1"));
    }
    let g1_affine: Vec<u8> = a[0]
        .iter()
        .rev()
        .chain(a[1].iter().rev())
        .cloned()
        .collect();
    G1Affine::deserialize_uncompressed(&*g1_affine).map_err(|e| anyhow!("{e}"))
}

fn risc0_g2_from_seal(b: &[Vec<Vec<u8>>]) -> Result<G2Affine> {
    if b.len() != 2 || b[0].len() != 2 || b[1].len() != 2 {
        return Err(anyhow!("Malformed G2"));
    }
    let g2_affine: Vec<u8> = b[0][1]
        .iter()
        .rev()
        .chain(b[0][0].iter().rev())
        .chain(b[1][1].iter().rev())
        .chain(b[1][0].iter().rev())
        .cloned()
        .collect();
    G2Affine::deserialize_uncompressed(&*g2_affine).map_err(|e| anyhow!("{e}"))
}

fn stellar_deserialize_fp2(chunk: &[u8; 64]) -> Result<Fq2> {
    let mut buf = *chunk;
    buf.reverse();
    Fq2::deserialize_uncompressed(&buf[..]).map_err(|e| anyhow!("Fp2 deserialize: {e}"))
}

fn stellar_serialize_fp2(f: Fq2) -> Result<[u8; 64]> {
    let mut buf = [0u8; 64];
    f.serialize_uncompressed(&mut buf[..])
        .map_err(|e| anyhow!("Fp2 serialize: {e}"))?;
    buf.reverse();
    Ok(buf)
}

fn stellar_deserialize_g1(bytes: &[u8; 64]) -> Result<G1Affine> {
    let mut xb = [0u8; 32];
    let mut yb = [0u8; 32];
    xb.copy_from_slice(&bytes[0..32]);
    yb.copy_from_slice(&bytes[32..64]);
    let mut xbuf = xb;
    xbuf.reverse();
    let mut ybuf = yb;
    ybuf.reverse();
    let x = Fq::deserialize_uncompressed(&xbuf[..]).map_err(|e| anyhow!("Fp x: {e}"))?;
    let y = Fq::deserialize_uncompressed(&ybuf[..]).map_err(|e| anyhow!("Fp y: {e}"))?;
    let pt = G1Affine::new_unchecked(x, y);
    if !pt.is_on_curve() {
        return Err(anyhow!("G1 not on curve"));
    }
    Ok(pt)
}

pub fn stellar_deserialize_g2(bytes: &[u8; 128]) -> Result<G2Affine> {
    let mut x_chunk = [0u8; 64];
    let mut y_chunk = [0u8; 64];
    x_chunk.copy_from_slice(&bytes[0..64]);
    y_chunk.copy_from_slice(&bytes[64..128]);
    let x = stellar_deserialize_fp2(&x_chunk)?;
    let y = stellar_deserialize_fp2(&y_chunk)?;
    let pt = G2Affine::new_unchecked(x, y);
    if !pt.is_on_curve() {
        return Err(anyhow!("G2 not on curve"));
    }
    if !pt.is_in_correct_subgroup_assuming_on_curve() {
        return Err(anyhow!("G2 not in subgroup"));
    }
    Ok(pt)
}

fn stellar_serialize_g1(p: &G1Affine) -> Result<[u8; 64]> {
    let mut out = [0u8; 64];
    let mut xb = [0u8; 32];
    let mut yb = [0u8; 32];
    p.x.serialize_uncompressed(&mut xb[..])
        .map_err(|e| anyhow!("G1 x ser: {e}"))?;
    p.y.serialize_uncompressed(&mut yb[..])
        .map_err(|e| anyhow!("G1 y ser: {e}"))?;
    xb.reverse();
    yb.reverse();
    out[0..32].copy_from_slice(&xb);
    out[32..64].copy_from_slice(&yb);
    Ok(out)
}

fn stellar_serialize_g2(p: &G2Affine) -> Result<[u8; 128]> {
    let mut out = [0u8; 128];
    out[0..64].copy_from_slice(&stellar_serialize_fp2(p.x)?);
    out[64..128].copy_from_slice(&stellar_serialize_fp2(p.y)?);
    Ok(out)
}

fn to_fixed_be32(bytes: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let start = 32usize.saturating_sub(bytes.len());
    out[start..].copy_from_slice(bytes);
    out
}

/// RISC0 `fr_from_bytes`: BE-fixed 32-byte limb → ark Fr.
fn risc0_fr_from_be_padded(padded_be: [u8; 32]) -> Result<ark_bn254::Fr> {
    let mut le = padded_be;
    le.reverse();
    ark_bn254::Fr::deserialize_uncompressed(&le[..]).map_err(|e| anyhow!("{e}"))
}

/// Soroban `Fr::from_bytes` = big-endian field element.
fn ark_fr_to_soroban_bytes(fr: &ark_bn254::Fr) -> [u8; 32] {
    let mut le = [0u8; 32];
    fr.serialize_uncompressed(&mut le[..]).expect("Fr serialize");
    le.reverse();
    le
}

/// Five public inputs for `risc0-verifier` (matches `risc0_groth16::Verifier::new`).
pub fn risc0_five_public_inputs(
    control_root: Digest,
    claim_digest: Digest,
    bn254_control_id: Digest,
) -> Result<Vec<[u8; 32]>> {
    let (a0, a1) = split_digest_fr_limbs(control_root);
    let (c0, c1) = split_digest_fr_limbs(claim_digest);
    let id_limb = bn254_control_id_fr(bn254_control_id);
    Ok(vec![
        ark_fr_to_soroban_bytes(&risc0_fr_from_be_padded(a0)?),
        ark_fr_to_soroban_bytes(&risc0_fr_from_be_padded(a1)?),
        ark_fr_to_soroban_bytes(&risc0_fr_from_be_padded(c0)?),
        ark_fr_to_soroban_bytes(&risc0_fr_from_be_padded(c1)?),
        ark_fr_to_soroban_bytes(&risc0_fr_from_be_padded(id_limb)?),
    ])
}

/// RISC0 `split_digest`: reverse digest bytes, split 16+16, map each half to Fr (BE u256 limb).
fn split_digest_fr_limbs(digest: Digest) -> ([u8; 32], [u8; 32]) {
    let mut be: Vec<u8> = digest.as_bytes().to_vec();
    be.reverse();
    let (first, second) = be.split_at(16);
    let mut limb_second = [0u8; 32];
    let mut limb_first = [0u8; 32];
    limb_second[16..].copy_from_slice(second);
    limb_first[16..].copy_from_slice(first);
    (limb_second, limb_first)
}

/// `bn254_control_id.as_mut_bytes().reverse()` then single Fr (5th public input).
fn bn254_control_id_fr(mut id: Digest) -> [u8; 32] {
    id.as_mut_bytes().reverse();
    let mut repr = [0u8; 32];
    repr.copy_from_slice(id.as_bytes());
    repr
}

pub fn seal_to_soroban(seal: &Seal) -> Result<([u8; 64], [u8; 128], [u8; 64])> {
    use crate::host_stellar::{
        deserialize_g2, g1_from_05_uncompressed, g2_from_05_uncompressed, stellar_g1, stellar_g2,
    };
    use ark_serialize::CanonicalSerialize;

    let g1a = risc0_g1_from_seal(&seal.a).context("decode proof A")?;
    let g2b = risc0_g2_from_seal(&seal.b).context("decode proof B")?;
    let g1c = risc0_g1_from_seal(&seal.c).context("decode proof C")?;

    let mut buf = vec![];
    g1a.serialize_uncompressed(&mut buf)
        .context("serialize A")?;
    let g1a04 = g1_from_05_uncompressed(&buf).context("A ark0.4")?;
    buf.clear();
    g2b.serialize_uncompressed(&mut buf)
        .context("serialize B")?;
    let g2b04 = g2_from_05_uncompressed(&buf).context("B ark0.4")?;
    buf.clear();
    g1c.serialize_uncompressed(&mut buf)
        .context("serialize C")?;
    let g1c04 = g1_from_05_uncompressed(&buf).context("C ark0.4")?;

    let proof_a = stellar_g1(&g1a04);
    let proof_b = stellar_g2(&g2b04);
    let proof_c = stellar_g1(&g1c04);

    let _ = deserialize_g2(&proof_b).context("proof_b host stellar check")?;

    Ok((proof_a, proof_b, proof_c))
}

#[derive(Debug, Serialize)]
pub struct SorobanInvokeJson {
    pub proof_a_hex: String,
    pub proof_b_hex: String,
    pub proof_c_hex: String,
    pub public_inputs_hex: Vec<String>,
}

pub fn build_soroban_invoke(
    seal: &Seal,
    control_root: Digest,
    claim_digest: Digest,
    bn254_control_id: Digest,
) -> Result<SorobanInvokeJson> {
    let (proof_a, proof_b, proof_c) = seal_to_soroban(seal)?;
    let public_inputs = risc0_five_public_inputs(control_root, claim_digest, bn254_control_id)?;
    Ok(SorobanInvokeJson {
        proof_a_hex: hex::encode(proof_a),
        proof_b_hex: hex::encode(proof_b),
        proof_c_hex: hex::encode(proof_c),
        public_inputs_hex: public_inputs.iter().map(hex::encode).collect(),
    })
}
