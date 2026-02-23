use axum::{extract::State, response::Json};
use rand::Rng;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use uuid::Uuid;

use crate::models::*;
use crate::utils::score_word_frequencies;

/// GET /api/challenge - Generate a new text frequency challenge
pub async fn create_challenge(State(state): State<AppState>) -> Json<ChallengeResponse> {
    let mut rng = rand::thread_rng();

    let state_guard = state.read().unwrap();
    let cfg = &state_guard.config.challenge;

    let word_count = rng.gen_range(cfg.word_count_min..=cfg.word_count_max);
    let duration_ms = rng.gen_range(cfg.duration_ms_min..=cfg.duration_ms_max);

    // Sample words randomly from the pool
    let words: Vec<String> = (0..word_count)
        .map(|_| cfg.word_pool[rng.gen_range(0..cfg.word_pool.len())].clone())
        .collect();

    // Compute ground-truth frequency map
    let mut word_frequencies: HashMap<String, usize> = HashMap::new();
    for word in &words {
        *word_frequencies.entry(word.clone()).or_insert(0) += 1;
    }
    let unique_count = word_frequencies.len();

    // Build display text: wrap into lines of 15 words each
    let text_content = words
        .chunks(15)
        .map(|chunk| chunk.join(" "))
        .collect::<Vec<_>>()
        .join("\n");

    let challenge_id = Uuid::new_v4().to_string();
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let challenge = Challenge {
        word_frequencies,
        created_at,
    };

    drop(state_guard);

    state
        .write()
        .unwrap()
        .challenges
        .insert(challenge_id.clone(), challenge);

    info!(
        "Created challenge {} with {} words, {} unique",
        challenge_id, word_count, unique_count
    );

    Json(ChallengeResponse {
        challenge_id,
        text_content,
        duration_ms,
    })
}

/// POST /api/submit - Validate submitted word frequency answer
pub async fn submit_answer(
    State(state): State<AppState>,
    Json(payload): Json<SubmitRequest>,
) -> Json<SubmitResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let state_guard = state.read().unwrap();
    let Some(challenge) = state_guard.challenges.get(&payload.challenge_id) else {
        return Json(SubmitResponse {
            success: false,
            message: "Challenge not found".to_string(),
            score: None,
        });
    };

    let validation_cfg = &state_guard.config.validation;

    // Check timing
    let elapsed = now - challenge.created_at;
    if validation_cfg.min_time_ms > 0 && elapsed < validation_cfg.min_time_ms {
        return Json(SubmitResponse {
            success: false,
            message: format!(
                "Too fast: {}ms (minimum {}ms)",
                elapsed, validation_cfg.min_time_ms
            ),
            score: None,
        });
    }
    if elapsed > validation_cfg.max_time_ms {
        return Json(SubmitResponse {
            success: false,
            message: format!(
                "Too slow: {}ms (maximum {}ms)",
                elapsed, validation_cfg.max_time_ms
            ),
            score: None,
        });
    }

    // Score the submitted frequency map against the ground truth
    let score = score_word_frequencies(&payload.answer, &challenge.word_frequencies);
    let success = score >= validation_cfg.success_threshold;
    let threshold_percent = validation_cfg.success_threshold * 100.0;

    let message = if success {
        format!("Success! Score: {:.2}%", score * 100.0)
    } else {
        format!(
            "Failed: Score {:.2}% (need {:.0}%)",
            score * 100.0,
            threshold_percent
        )
    };

    info!(
        "Challenge {} submitted after {}ms with score {:.2}%: {}",
        payload.challenge_id,
        elapsed,
        score * 100.0,
        if success { "PASSED" } else { "FAILED" }
    );

    Json(SubmitResponse {
        success,
        message,
        score: Some(score),
    })
}
