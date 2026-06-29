//! ProveKit Gate — verify a RISC Zero Groth16 proof via the deployed verifier,
//! bind it to the expected guest claim + policy, and enforce a one-shot nullifier.
//!
//! Three independent layers (kept separate on purpose, see `docs/DEMO_POLICY.md`):
//!   1. Cryptography  — Groth16 pairing check in `contracts/risc0-verifier`.
//!   2. Program/policy — the proof's RISC Zero `claim_digest` must equal the value
//!                       baked in at build time (binds to *this* guest + output),
//!                       and the caller's `policy_commitment` must match the value
//!                       set at `initialize`.
//!   3. Anti-replay    — a proof-bound nullifier can be spent exactly once.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, vec, Address, Bytes, BytesN, Env, IntoVal, Map, Symbol,
    Vec,
};

/// RISC Zero Groth16 wrapper exposes 5 public inputs:
/// `control_root` (2 limbs), `claim_digest` (2 limbs), `bn254_control_id` (1 limb).
const RISC0_PUBLIC_INPUT_COUNT: u32 = 5;

/// Expected RISC Zero `claim_digest` limbs for the locked guest execution
/// (`artifacts/soroban_groth16_invoke.json`, public inputs index 2 and 3).
///
/// `claim_digest = SHA-256(image_id, journal_digest, ...)`, so pinning it binds the
/// gate to exactly *this* guest program and *this* attested journal — not merely
/// "some valid RISC Zero proof". Regenerate with `provekit-groth16-reencode`.
const EXPECTED_CLAIM_DIGEST_0: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 224, 226, 134, 131, 29, 52, 50, 215, 69, 48,
    163, 229, 238, 183, 17, 117,
];
const EXPECTED_CLAIM_DIGEST_1: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 106, 92, 8, 213, 43, 137, 38, 100, 52, 245,
    248, 113, 159, 166, 99, 211,
];

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

    /// Verify via the deployed RISC0 Groth16 verifier, bind the proof to the
    /// expected guest claim + policy, then spend a proof-bound nullifier (one-shot).
    ///
    /// Returns `true` only on the first successful spend of a valid, policy-matching,
    /// claim-bound proof; `false` for any rejection (wrong policy, wrong claim,
    /// wrong nullifier, already spent, or failed pairing).
    pub fn verify_and_spend_risc0(
        env: Env,
        nullifier: BytesN<32>,
        policy_commitment: BytesN<32>,
        proof_a: Bytes,
        proof_b: Bytes,
        proof_c: Bytes,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        // (1) Policy version: caller's commitment must match the initialized policy.
        let stored_policy: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::PolicyCommitment)
            .expect("call initialize first");
        if policy_commitment != stored_policy {
            return false;
        }

        // RISC Zero wrapper always carries exactly 5 public inputs.
        if public_inputs.len() != RISC0_PUBLIC_INPUT_COUNT {
            return false;
        }

        // (2) Program/output binding: the proof's claim_digest (limbs 2,3) must match
        // the guest execution this gate was built for. Without this, any valid RISC
        // Zero proof of any program would satisfy the verifier's pairing check.
        let expected_claim_0 = BytesN::from_array(&env, &EXPECTED_CLAIM_DIGEST_0);
        let expected_claim_1 = BytesN::from_array(&env, &EXPECTED_CLAIM_DIGEST_1);
        if public_inputs.get(2).unwrap() != expected_claim_0
            || public_inputs.get(3).unwrap() != expected_claim_1
        {
            return false;
        }

        // (3) Anti-replay: nullifier must be the proof's own id, and unspent.
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

        // Cryptography: delegate the Groth16 pairing check to the verifier contract.
        let verifier: Address = env
            .storage()
            .instance()
            .get(&DataKey::Risc0Verifier)
            .expect("call init_risc0_verifier first");

        let verified: bool = env.invoke_contract(
            &verifier,
            &Symbol::new(&env, "verify_proof"),
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

#[cfg(test)]
mod test;
