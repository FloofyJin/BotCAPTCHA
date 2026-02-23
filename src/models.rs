use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::config::SharedConfig;

// Challenge stored on server (ground truth, never sent to client)
#[derive(Clone, Debug)]
pub struct Challenge {
    pub word_frequencies: HashMap<String, usize>,
    pub created_at: u64,
}

// Challenge response sent to client
#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub text_content: String,
    pub duration_ms: u64,
}

// Submit request from client
#[derive(Deserialize)]
pub struct SubmitRequest {
    pub challenge_id: String,
    pub answer: HashMap<String, usize>,
}

// Submit response
#[derive(Serialize)]
pub struct SubmitResponse {
    pub success: bool,
    pub message: String,
    pub score: Option<f32>,
}

// Application state
pub struct AppStateData {
    pub challenges: HashMap<String, Challenge>,
    pub config: SharedConfig,
}

// Shared application state
pub type AppState = Arc<RwLock<AppStateData>>;
