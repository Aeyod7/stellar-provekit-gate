extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

fn hex32(s: &str) -> [u8; 32] {
    let h = s.strip_prefix("0x").unwrap_or(s);
    let mut out = [0u8; 32];
    for (i, chunk) in h.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
    }
    out
}

#[test]
fn proof_id_is_deterministic_for_fixed_inputs() {
    let env = Env::default();
    let proof_a = Bytes::from_array(&env, &[1u8; 64]);
    let proof_b = Bytes::from_array(&env, &[2u8; 128]);
    let proof_c = Bytes::from_array(&env, &[3u8; 64]);
    let mut pis = Vec::new(&env);
    pis.push_back(BytesN::from_array(
        &env,
        &hex32("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    ));
    let id1 = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &pis);
    let id2 = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &pis);
    assert_eq!(id1, id2);
}

#[test]
fn proof_id_changes_when_public_input_changes() {
    let env = Env::default();
    let proof_a = Bytes::from_array(&env, &[1u8; 64]);
    let proof_b = Bytes::from_array(&env, &[2u8; 128]);
    let proof_c = Bytes::from_array(&env, &[3u8; 64]);
    let mut pis_a = Vec::new(&env);
    pis_a.push_back(BytesN::from_array(
        &env,
        &hex32("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    ));
    let mut pis_b = Vec::new(&env);
    pis_b.push_back(BytesN::from_array(
        &env,
        &hex32("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    ));
    let id_a = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &pis_a);
    let id_b = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &pis_b);
    assert_ne!(id_a, id_b);
}

#[test]
fn init_risc0_verifier_admin_gated_and_one_time() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let policy = BytesN::from_array(&env, &[9u8; 32]);

    let contract_id = env.register(ProveKitGate, ());
    let client = ProveKitGateClient::new(&env, &contract_id);

    client.initialize(&admin, &policy);
    client.init_risc0_verifier(&verifier);

    let stored: Address = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&DataKey::Risc0Verifier)
            .unwrap()
    });
    assert_eq!(stored, verifier);
}

#[test]
#[should_panic(expected = "risc0 verifier already set")]
fn init_risc0_verifier_panics_if_called_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let v1 = Address::generate(&env);
    let v2 = Address::generate(&env);
    let policy = BytesN::from_array(&env, &[9u8; 32]);

    let contract_id = env.register(ProveKitGate, ());
    let client = ProveKitGateClient::new(&env, &contract_id);

    client.initialize(&admin, &policy);
    client.init_risc0_verifier(&v1);
    client.init_risc0_verifier(&v2);
}

fn artifacts_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/soroban_groth16_invoke.json")
}

fn policy_commitment_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/policy_commitment.hex")
}

fn load_artifact_invoke() -> serde_json::Value {
    let raw = std::fs::read_to_string(artifacts_path()).unwrap_or_else(|e| {
        panic!("missing artifacts: {e}");
    });
    serde_json::from_str(&raw).unwrap()
}

fn hex_to_bytes<const N: usize>(s: &str) -> [u8; N] {
    let h = s.strip_prefix("0x").unwrap_or(s);
    let mut out = [0u8; N];
    for (i, chunk) in h.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
    }
    out
}

fn load_policy_commitment(env: &Env) -> BytesN<32> {
    let raw = std::fs::read_to_string(policy_commitment_path()).unwrap();
    let h = raw.trim().strip_prefix("0x").unwrap_or(raw.trim());
    BytesN::from_array(env, &hex_to_bytes::<32>(h))
}

fn register_risc0_verifier_wasm(env: &Env) -> Address {
    let wasm_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../risc0-verifier/target/wasm32v1-none/release/soroban_risc0_verifier.wasm");
    let wasm = std::fs::read(&wasm_path).unwrap_or_else(|e| {
        panic!(
            "missing risc0 verifier wasm at {}: {e}. Run: cd contracts/risc0-verifier && stellar contract build",
            wasm_path.display()
        );
    });
    let wasm_bytes = Bytes::from_slice(env, &wasm);
    env.register_contract_wasm(None, wasm_bytes)
}

fn setup_gate_with_verifier(env: &Env) -> (ProveKitGateClient<'_>, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let verifier_id = register_risc0_verifier_wasm(env);
    let gate_id = env.register(ProveKitGate, ());
    let client = ProveKitGateClient::new(env, &gate_id);
    let policy = load_policy_commitment(env);
    client.initialize(&admin, &policy);
    client.init_risc0_verifier(&verifier_id);
    (client, admin)
}

fn invoke_args_from_artifacts(env: &Env, v: &serde_json::Value) -> (Bytes, Bytes, Bytes, Vec<BytesN<32>>) {
    let proof_a = Bytes::from_array(env, &hex_to_bytes::<64>(v["proof_a_hex"].as_str().unwrap()));
    let proof_b = Bytes::from_array(env, &hex_to_bytes::<128>(v["proof_b_hex"].as_str().unwrap()));
    let proof_c = Bytes::from_array(env, &hex_to_bytes::<64>(v["proof_c_hex"].as_str().unwrap()));
    let mut pub_vec = Vec::new(env);
    for pi_hex in v["public_inputs_hex"].as_array().unwrap() {
        let arr = hex_to_bytes::<32>(pi_hex.as_str().unwrap());
        pub_vec.push_back(BytesN::from_array(env, &arr));
    }
    (proof_a, proof_b, proof_c, pub_vec)
}

#[test]
fn verify_and_spend_risc0_e2e_with_locked_artifacts() {
    let env = Env::default();
    let (client, _admin) = setup_gate_with_verifier(&env);
    let v = load_artifact_invoke();
    let (proof_a, proof_b, proof_c, pub_vec) = invoke_args_from_artifacts(&env, &v);
    let policy = load_policy_commitment(&env);
    let nullifier = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &pub_vec);

    assert!(client.verify_and_spend_risc0(
        &nullifier,
        &policy,
        &proof_a,
        &proof_b,
        &proof_c,
        &pub_vec,
    ));
}

#[test]
fn verify_and_spend_risc0_replay_returns_false() {
    let env = Env::default();
    let (client, _admin) = setup_gate_with_verifier(&env);
    let v = load_artifact_invoke();
    let (proof_a, proof_b, proof_c, pub_vec) = invoke_args_from_artifacts(&env, &v);
    let policy = load_policy_commitment(&env);
    let nullifier = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &pub_vec);

    assert!(client.verify_and_spend_risc0(
        &nullifier, &policy, &proof_a, &proof_b, &proof_c, &pub_vec,
    ));
    assert!(!client.verify_and_spend_risc0(
        &nullifier, &policy, &proof_a, &proof_b, &proof_c, &pub_vec,
    ));
}

#[test]
fn verify_and_spend_risc0_wrong_policy_returns_false() {
    let env = Env::default();
    let (client, _admin) = setup_gate_with_verifier(&env);
    let v = load_artifact_invoke();
    let (proof_a, proof_b, proof_c, pub_vec) = invoke_args_from_artifacts(&env, &v);
    let wrong_policy = BytesN::from_array(&env, &[0xee; 32]);
    let nullifier = compute_proof_id(&env, &proof_a, &proof_b, &proof_c, &pub_vec);

    assert!(!client.verify_and_spend_risc0(
        &nullifier,
        &wrong_policy,
        &proof_a,
        &proof_b,
        &proof_c,
        &pub_vec,
    ));
}

#[test]
fn verify_and_spend_risc0_wrong_nullifier_returns_false() {
    let env = Env::default();
    let (client, _admin) = setup_gate_with_verifier(&env);
    let v = load_artifact_invoke();
    let (proof_a, proof_b, proof_c, pub_vec) = invoke_args_from_artifacts(&env, &v);
    let policy = load_policy_commitment(&env);
    let bad_nullifier = BytesN::from_array(&env, &[0x11; 32]);

    assert!(!client.verify_and_spend_risc0(
        &bad_nullifier,
        &policy,
        &proof_a,
        &proof_b,
        &proof_c,
        &pub_vec,
    ));
}
