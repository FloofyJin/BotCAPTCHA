use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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

// Submit response
#[derive(Serialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub message: String,
    pub score: Option<f32>,
    pub word_score: Option<f32>,
    pub grid_score: Option<f32>,
}

// Application state
pub struct AppStateData {
    pub challenges: HashMap<String, Challenge>,
    pub config: SharedConfig,
}

// Shared application state
pub type AppState = Arc<RwLock<AppStateData>>;
