//! Groth16 verifier for RISC Zero's fixed BN254 wrapper circuit (5 public inputs).
#![no_std]

use soroban_sdk::{
    contract, contractimpl,
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr, BN254_G1_SERIALIZED_SIZE, BN254_G2_SERIALIZED_SIZE},
    vec, Bytes, BytesN, Env, TryFromVal, Vec,
};

const PROOF_A_LEN: usize = BN254_G1_SERIALIZED_SIZE;
const PROOF_B_LEN: usize = BN254_G2_SERIALIZED_SIZE;
/// RISC Zero Groth16: control_root (2), claim_digest (2), bn254_control_id (1).
pub const PUBLIC_INPUT_COUNT: u32 = 5;

include!("vk_constants.rs");

#[contract]
pub struct Risc0VerifierContract;

#[contractimpl]
impl Risc0VerifierContract {
    pub fn verify_proof(
        env: Env,
        proof_a: Bytes,
        proof_b: Bytes,
        proof_c: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        let proof_a = read_g1(&env, &proof_a, "proof_a");
        let proof_b = read_g2(&env, &proof_b, "proof_b");
        let proof_c = read_g1(&env, &proof_c, "proof_c");

        if public_inputs.len() != PUBLIC_INPUT_COUNT {
            return false;
        }

        let vk_alpha = Bn254G1Affine::from_array(&env, &VK_ALPHA_G1);
        let vk_beta = Bn254G2Affine::from_array(&env, &VK_BETA_G2);
        let vk_gamma = Bn254G2Affine::from_array(&env, &VK_GAMMA_G2);
        let vk_delta = Bn254G2Affine::from_array(&env, &VK_DELTA_G2);

        let ic = [
            Bn254G1Affine::from_array(&env, &VK_IC0_G1),
            Bn254G1Affine::from_array(&env, &VK_IC1_G1),
            Bn254G1Affine::from_array(&env, &VK_IC2_G1),
            Bn254G1Affine::from_array(&env, &VK_IC3_G1),
            Bn254G1Affine::from_array(&env, &VK_IC4_G1),
            Bn254G1Affine::from_array(&env, &VK_IC5_G1),
        ];

        let mut vk_x = ic[0].clone();
        for i in 0..PUBLIC_INPUT_COUNT as usize {
            let scalar = Fr::from_bytes(public_inputs.get(i as u32).unwrap());
            vk_x = vk_x + (ic[i + 1].clone() * scalar);
        }

        env.crypto().bn254().pairing_check(
            vec![&env, proof_a, -vk_alpha, -vk_x, -proof_c],
            vec![&env, proof_b, vk_beta, vk_gamma, vk_delta],
        )
    }
}

fn read_g1(env: &Env, bytes: &Bytes, label: &str) -> Bn254G1Affine {
    assert_eq!(bytes.len(), PROOF_A_LEN as u32, "{label} must be 64 bytes");
    let bytesn = BytesN::<PROOF_A_LEN>::try_from_val(env, bytes.as_val())
        .expect("proof bytes must be convertible to BytesN<64>");
    Bn254G1Affine::from_bytes(bytesn)
}

fn read_g2(env: &Env, bytes: &Bytes, label: &str) -> Bn254G2Affine {
    assert_eq!(bytes.len(), PROOF_B_LEN as u32, "{label} must be 128 bytes");
    let bytesn = BytesN::<PROOF_B_LEN>::try_from_val(env, bytes.as_val())
        .expect("proof bytes must be convertible to BytesN<128>");
    Bn254G2Affine::from_bytes(bytesn)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use soroban_sdk::{Bytes, Env, Vec};

    fn hex_to_bytes<const N: usize>(hex: &str) -> [u8; N] {
        let h = hex.strip_prefix("0x").unwrap_or(hex);
        let mut out = [0u8; N];
        for (i, chunk) in h.as_bytes().chunks(2).enumerate() {
            let s = std::str::from_utf8(chunk).unwrap();
            out[i] = u8::from_str_radix(s, 16).unwrap();
        }
        out
    }

    #[test]
    fn verify_artifacts_groth16_invoke_json() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../artifacts/soroban_groth16_invoke.json");
        let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("missing {}: {e}", path.display());
        });
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let proof_a = hex_to_bytes::<64>(v["proof_a_hex"].as_str().unwrap());
        let proof_b = hex_to_bytes::<128>(v["proof_b_hex"].as_str().unwrap());
        let proof_c = hex_to_bytes::<64>(v["proof_c_hex"].as_str().unwrap());

        let env = Env::default();
        let contract_id = env.register(Risc0VerifierContract, ());
        let client = Risc0VerifierContractClient::new(&env, &contract_id);

        let proof_a_b = Bytes::from_array(&env, &proof_a);
        let proof_b_b = Bytes::from_array(&env, &proof_b);
        let proof_c_b = Bytes::from_array(&env, &proof_c);
        let mut pub_vec = Vec::new(&env);
        for pi_hex in v["public_inputs_hex"].as_array().unwrap() {
            let arr = hex_to_bytes::<32>(pi_hex.as_str().unwrap());
            pub_vec.push_back(BytesN::from_array(&env, &arr));
        }

        assert!(
            client.verify_proof(&proof_a_b, &proof_b_b, &proof_c_b, &pub_vec),
            "expected artifact proof to verify in local Soroban env"
        );
    }
}