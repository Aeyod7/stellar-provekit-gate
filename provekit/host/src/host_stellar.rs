//! Stellar Soroban host (ark 0.4) BN254 point encoding — must match soroban-env-host.
use anyhow::{anyhow, Result};
use ark_bn254_04::{Fq, Fq2, G1Affine, G2Affine};
use ark_ec_04::AffineRepr;
use ark_serialize_04::{CanonicalDeserialize, CanonicalSerialize};

pub fn stellar_fp(elem: &Fq) -> [u8; 32] {
    let mut raw = [0u8; 32];
    elem.serialize_uncompressed(&mut raw[..]).unwrap();
    raw.reverse();
    raw
}

pub fn stellar_fp2(elem: &Fq2) -> [u8; 64] {
    let mut raw = [0u8; 64];
    elem.serialize_uncompressed(&mut raw[..]).unwrap();
    raw.reverse();
    raw
}

pub fn stellar_g1(p: &G1Affine) -> [u8; 64] {
    if p.is_zero() {
        let mut o = [0u8; 64];
        o[0] = 0x40;
        return o;
    }
    let mut out = [0u8; 64];
    out[0..32].copy_from_slice(&stellar_fp(&p.x));
    out[32..64].copy_from_slice(&stellar_fp(&p.y));
    out
}

pub fn stellar_g2(p: &G2Affine) -> [u8; 128] {
    if p.is_zero() {
        let mut o = [0u8; 128];
        o[0] = 0x40;
        return o;
    }
    let mut out = [0u8; 128];
    out[0..64].copy_from_slice(&stellar_fp2(&p.x));
    out[64..128].copy_from_slice(&stellar_fp2(&p.y));
    out
}

pub fn deserialize_g2(bytes: &[u8; 128]) -> Result<G2Affine> {
    let flags = bytes[0] & 0xC0;
    if flags == 0x40 {
        return Ok(G2Affine::zero());
    }
    if flags == 0x80 {
        return Err(anyhow!("compressed G2 not supported"));
    }
    let mut x = [0u8; 64];
    x.copy_from_slice(&bytes[0..64]);
    let mut y = [0u8; 64];
    y.copy_from_slice(&bytes[64..128]);
    x.reverse();
    y.reverse();
    let x = Fq2::deserialize_uncompressed(&x[..]).map_err(|e| anyhow!("{e}"))?;
    let y = Fq2::deserialize_uncompressed(&y[..]).map_err(|e| anyhow!("{e}"))?;
    let p = G2Affine::new(x, y);
    if !p.is_on_curve() {
        return Err(anyhow!("G2 not on curve (host ark 0.4)"));
    }
    Ok(p)
}

/// Convert ark 0.5 point to 0.4 via canonical uncompressed bytes.
pub fn g2_from_05_uncompressed(buf: &[u8]) -> Result<G2Affine> {
    G2Affine::deserialize_uncompressed(buf).map_err(|e| anyhow!("{e}"))
}

pub fn g1_from_05_uncompressed(buf: &[u8]) -> Result<G1Affine> {
    G1Affine::deserialize_uncompressed(buf).map_err(|e| anyhow!("{e}"))
}
