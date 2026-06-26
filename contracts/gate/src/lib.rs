//! ProveKit Gate — verify Groth16 proof + enforce one-shot nullifier.
//!
//! Verifier VK is embedded in `verifier` contract; this contract composes app logic
//! for hackathon demo (policy gate without revealing private inputs).

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    crypto::bn254::{
        Bn254G1Affine, Bn254G2Affine, Fr, BN254_G1_SERIALIZED_SIZE, BN254_G2_SERIALIZED_SIZE,
    },
    vec, Address, Bytes, BytesN, Env, IntoVal, Map, TryFromVal, Vec,
};

const PROOF_A_LEN: usize = BN254_G1_SERIALIZED_SIZE;
const PROOF_B_LEN: usize = BN254_G2_SERIALIZED_SIZE;
const PUBLIC_INPUT_COUNT: u32 = 1;
/// RISC Zero Groth16 wrapper (must match `contracts/risc0-verifier`).
const RISC0_PUBLIC_INPUT_COUNT: u32 = 5;

// Placeholder VK — matches soroban-zk Poseidon preimage circuit until RISC Zero Groth16 VK is wired.
include!("vk_constants.rs");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PolicyCommitment,
    Risc0Verifier,
    Spent,
}

#[contract]
pub struct ProveKitGate;

#[contractimpl]
impl ProveKitGate {
    /// One-time: set admin and expected guest policy commitment (SHA-256 of journal binding).
    pub fn initialize(env: Env, admin: Address, policy_commitment: BytesN<32>) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PolicyCommitment, &policy_commitment);
    }

    /// Admin-only, one-time: store deployed `risc0-verifier` contract address.
    pub fn init_risc0_verifier(env: Env, risc0_verifier: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("call initialize first");
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Risc0Verifier) {
            panic!("risc0 verifier already set");
        }
        env.storage()
            .instance()
            .set(&DataKey::Risc0Verifier, &risc0_verifier);
    }

    /// Verify via deployed RISC0 Groth16 verifier, then spend proof-bound nullifier (one-shot).
    pub fn verify_and_spend_risc0(
        env: Env,
        nullifier: BytesN<32>,
        policy_commitment: BytesN<32>,
        proof_a: Bytes,
        proof_b: Bytes,
        proof_c: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        let stored_policy: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::PolicyCommitment)
            .expect("call initialize first");
        if policy_commitment != stored_policy {
            return false;
        }

        let proof_id = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &public_inputs);
        if nullifier != proof_id {
            return false;
        }

        let mut spent: Map<BytesN<32>, bool> = env
            .storage()
            .persistent()
            .get(&DataKey::Spent)
            .unwrap_or(Map::new(&env));

        if spent.get(nullifier.clone()).unwrap_or(false) {
            return false;
        }

        let verifier: Address = env
            .storage()
            .instance()
            .get(&DataKey::Risc0Verifier)
            .expect("call init_risc0_verifier first");

        if public_inputs.len() != RISC0_PUBLIC_INPUT_COUNT {
            return false;
        }

        let verified: bool = env.invoke_contract(
            &verifier,
            &soroban_sdk::Symbol::new(&env, "verify_proof"),
            vec![
                &env,
                proof_a.into_val(&env),
                proof_b.into_val(&env),
                proof_c.into_val(&env),
                public_inputs.into_val(&env),
            ],
        );

        if !verified {
            return false;
        }

        spent.set(nullifier, true);
        env.storage().persistent().set(&DataKey::Spent, &spent);
        true
    }

    /// Returns true if proof verifies and nullifier has not been spent.
    pub fn verify_and_spend(
        env: Env,
        nullifier: BytesN<32>,
        proof_a: Bytes,
        proof_b: Bytes,
        proof_c: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        let mut spent: Map<BytesN<32>, bool> = env
            .storage()
            .persistent()
            .get(&DataKey::Spent)
            .unwrap_or(Map::new(&env));

        if spent.get(nullifier.clone()).unwrap_or(false) {
            return false;
        }

        if !Self::verify_proof(env.clone(), proof_a, proof_b, proof_c, public_inputs) {
            return false;
        }

        spent.set(nullifier, true);
        env.storage().persistent().set(&DataKey::Spent, &spent);
        true
    }

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
        let vk_ic0 = Bn254G1Affine::from_array(&env, &VK_IC0_G1);
        let vk_ic1 = Bn254G1Affine::from_array(&env, &VK_IC1_G1);

        let public_input = Fr::from_bytes(public_inputs.get(0).unwrap());
        let vk_x = vk_ic0 + (vk_ic1 * public_input);

        env.crypto().bn254().pairing_check(
            vec![&env, proof_a, -vk_alpha, -vk_x, -proof_c],
            vec![&env, proof_b, vk_beta, vk_gamma, vk_delta],
        )
    }
}

/// SHA-256(proof_a || proof_b || proof_c || each public input 32-byte limb).
pub fn compute_proof_id(
    env: &Env,
    proof_a: &Bytes,
    proof_b: &Bytes,
    proof_c: &Bytes,
    public_inputs: &Vec<BytesN<32>>,
) -> BytesN<32> {
    let mut combined = Bytes::new(env);
    combined.append(proof_a);
    combined.append(proof_b);
    combined.append(proof_c);
    for i in 0..public_inputs.len() {
        let pi = public_inputs.get(i).unwrap();
        let chunk = Bytes::from_array(env, &pi.to_array());
        combined.append(&chunk);
    }
    env.crypto().sha256(&combined).into()
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
mod test;
