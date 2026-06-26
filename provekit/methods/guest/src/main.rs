use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Private inputs (not in journal).
#[derive(Debug, Deserialize, Serialize)]
pub struct ProveGateInput {
    pub secret_score: u32,
    pub threshold: u32,
}

/// Public outputs committed to the journal (visible after prove).
#[derive(Debug, Deserialize, Serialize)]
pub struct ProveGateOutput {
    pub threshold_met: u32,
    pub commitment: [u8; 32],
}

/// SHA-256 over canonical little-endian `(score, threshold, threshold_met)`.
pub fn commitment_for(score: u32, threshold: u32, threshold_met: u32) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(score.to_le_bytes());
    hasher.update(threshold.to_le_bytes());
    hasher.update(threshold_met.to_le_bytes());
    hasher.finalize().into()
}

fn main() {
    let input: ProveGateInput = env::read();

    let met = if input.secret_score >= input.threshold {
        1u32
    } else {
        0u32
    };

    let output = ProveGateOutput {
        threshold_met: met,
        commitment: commitment_for(input.secret_score, input.threshold, met),
    };

    env::commit(&output);
}