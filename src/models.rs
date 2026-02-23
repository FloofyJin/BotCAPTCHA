use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use crate::config::SharedConfig;

// A single cell in the grid puzzle, 0-indexed
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GridCoord {
    pub col: u8,
    pub row: u8,
}

// Challenge stored on server (ground truth, never sent to client)
#[derive(Clone, Debug)]
pub struct Challenge {
    pub word_frequencies: HashMap<String, usize>,
    pub grid_solution: Vec<GridCoord>, // correct reading order (row asc, col asc)
    pub created_at: u64,
    pub sitekey: String, // sitekey that requested this challenge (empty = demo mode)
}

// Challenge response sent to client
#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub text_content: String,
    pub duration_ms: u64,
    pub grid_size: u8,
    // Natural-language description of the target cells in scrambled order.
    // Structured coordinates are intentionally withheld — parsing this requires an LLM.
    pub grid_coords_text: String,
}

// Submit request from client
#[derive(Deserialize)]
pub struct SubmitRequest {
    pub challenge_id: String,
    pub answer: HashMap<String, usize>,
    #[serde(default)]
    pub grid_answer: Vec<GridCoord>, // should be in reading order (row asc, col asc)
}

// Submit response — includes a signed token on success
#[derive(Serialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub message: String,
    pub score: Option<f32>,
    pub word_score: Option<f32>,
    pub grid_score: Option<f32>,
    /// Short-lived HMAC-signed token. Present only on success.
    /// The host backend should POST this to /api/verify to confirm validity.
    pub token: Option<String>,
}

// ── Token ─────────────────────────────────────────────────────────────────────

/// Claims embedded in a signed verification token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPayload {
    pub challenge_id: String,
    pub sitekey: String,
    pub score: f32,
    /// Issued-at (Unix ms)
    pub iat: u64,
    /// Expires-at (Unix ms)
    pub exp: u64,
}

// ── Verify endpoint ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub token: String,
    /// Optional: host can assert which sitekey the token should belong to.
    pub sitekey: Option<String>,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub message: String,
    pub sitekey: Option<String>,
    pub score: Option<f32>,
}

// ── Error response ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ── Application state ─────────────────────────────────────────────────────────

pub struct AppStateData {
    pub challenges: HashMap<String, Challenge>,
    /// Consumed token IDs (challenge_ids) — prevents replay attacks.
    pub used_tokens: HashSet<String>,
    pub config: SharedConfig,
}

// Shared application state
pub type AppState = Arc<RwLock<AppStateData>>;
