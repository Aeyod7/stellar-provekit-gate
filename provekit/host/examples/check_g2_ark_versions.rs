//! One-off: compare ark 0.4 (Soroban host) vs 0.5 (risc0) G2 deserialize on proof_b bytes.
use std::fs;

fn reverse32(b: &[u8]) -> [u8; 32] {
    let mut o = [0u8; 32];
    for i in 0..32 {
        o[i] = b[31 - i];
    }
    o
}

fn host_fp2_deserialize_04(chunk: &[u8; 64]) -> Result<ark_bn254_04::Fq2, String> {
    use ark_serialize_04::CanonicalDeserialize;
    let mut buf = *chunk;
    buf.reverse();
    ark_bn254_04::Fq2::deserialize_uncompressed(&buf[..]).map_err(|e| format!("{e}"))
}

fn host_g2_deserialize_04(bytes: &[u8; 128]) -> Result<ark_bn254_04::G2Affine, String> {
    use ark_ec_04::AffineRepr;
    use ark_serialize_04::CanonicalDeserialize;
    let flags = bytes[0] & 0xC0;
    if flags == 0x40 {
        return Ok(ark_bn254_04::G2Affine::zero());
    }
    if flags == 0x80 {
        return Err("compressed G2 not supported".into());
    }
    let mut x = [0u8; 64];
    x.copy_from_slice(&bytes[0..64]);
    let mut y = [0u8; 64];
    y.copy_from_slice(&bytes[64..128]);
    let x = host_fp2_deserialize_04(&x)?;
    let y = host_fp2_deserialize_04(&y)?;
    let p = ark_bn254_04::G2Affine::new(x, y);
    if !p.is_on_curve() {
        return Err("not on curve (ark 0.4)".into());
    }
    Ok(p)
}

fn stellar_g2_deserialize_05(bytes: &[u8; 128]) -> Result<ark_bn254::G2Affine, String> {
    use ark_ec::AffineRepr;
    use ark_serialize::CanonicalDeserialize;
    let flags = bytes[0] & 0xC0;
    if flags == 0x40 {
        return Ok(ark_bn254::G2Affine::zero());
    }
    if flags == 0x80 {
        return Err("compressed G2 not supported".into());
    }
    let mut x = [0u8; 64];
    x.copy_from_slice(&bytes[0..64]);
    let mut y = [0u8; 64];
    y.copy_from_slice(&bytes[64..128]);
    let mut xb = x;
    xb.reverse();
    let mut yb = y;
    yb.reverse();
    let x = ark_bn254::Fq2::deserialize_uncompressed(&xb[..]).map_err(|e| format!("{e}"))?;
    let y = ark_bn254::Fq2::deserialize_uncompressed(&yb[..]).map_err(|e| format!("{e}"))?;
    let p = ark_bn254::G2Affine::new(x, y);
    if !p.is_on_curve() {
        return Err("not on curve (ark 0.5)".into());
    }
    Ok(p)
}

fn main() {
    let json = fs::read_to_string("../../artifacts/soroban_groth16_invoke.json").unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let hex = v["proof_b_hex"].as_str().unwrap();
    let h = hex.strip_prefix("0x").unwrap_or(hex);
    let mut proof_b = [0u8; 128];
    for (i, c) in h.as_bytes().chunks(2).enumerate() {
        proof_b[i] = u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap();
    }

    println!("proof_b ark0.5: {:?}", stellar_g2_deserialize_05(&proof_b));
    println!(
        "proof_b ark0.4 host: {:?}",
        host_g2_deserialize_04(&proof_b)
    );

    // VK_BETA first 16 bytes from vk_constants - hardcode slice from file
    let vk_beta_hex = include_str!("../../../contracts/risc0-verifier/src/vk_constants.rs");
    // parse first G2 array line - skip, read file in main
    let vk = fs::read_to_string("../../../contracts/risc0-verifier/src/vk_constants.rs").unwrap();
    let start = vk.find("pub const VK_BETA_G2").unwrap();
    let bracket = vk[start..].find('[').unwrap() + start;
    let end = vk[bracket..].find("];").unwrap() + bracket;
    let nums: Vec<u8> = vk[bracket + 1..end]
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    let mut beta = [0u8; 128];
    beta.copy_from_slice(&nums[..128]);
    println!("VK_BETA ark0.5: {:?}", stellar_g2_deserialize_05(&beta));
    println!("VK_BETA ark0.4 host: {:?}", host_g2_deserialize_04(&beta));
}
