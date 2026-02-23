use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use uuid::Uuid;

use crate::models::*;
use crate::utils::{decode_token, generate_grid_description, mint_token, score_grid_answer, score_word_frequencies};

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ChallengeQuery {
    pub sitekey: Option<String>,
}

// ── GET /api/challenge ────────────────────────────────────────────────────────

/// Generate a combined word-frequency + grid-ordering challenge.
/// An optional `?sitekey=` query param identifies the host embedding the widget.
/// If site_keys is non-empty in config, only registered keys are accepted.
pub async fn create_challenge(
    State(state): State<AppState>,
    Query(query): Query<ChallengeQuery>,
) -> Result<Json<ChallengeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut rng = rand::thread_rng();

    let sitekey = query.sitekey.unwrap_or_default();

    // Validate sitekey if the config has a non-empty allow-list
    {
        let state_guard = state.read().unwrap();
        let allowed = &state_guard.config.auth.site_keys;
        if !allowed.is_empty() && !sitekey.is_empty() && !allowed.contains(&sitekey) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: format!("Unknown sitekey: {}", sitekey),
                }),
            ));
        }
    }

    let state_guard = state.read().unwrap();
    let cfg = &state_guard.config.challenge;

    // ── Word frequency puzzle ────────────────────────────────────────────────
    let word_count = rng.gen_range(cfg.word_count_min..=cfg.word_count_max);
    let duration_ms = rng.gen_range(cfg.duration_ms_min..=cfg.duration_ms_max);

    let words: Vec<String> = (0..word_count)
        .map(|_| cfg.word_pool[rng.gen_range(0..cfg.word_pool.len())].clone())
        .collect();

    let mut word_frequencies: HashMap<String, usize> = HashMap::new();
    for word in &words {
        *word_frequencies.entry(word.clone()).or_insert(0) += 1;
    }
    let unique_word_count = word_frequencies.len();

    let text_content = words
        .chunks(15)
        .map(|chunk| chunk.join(" "))
        .collect::<Vec<_>>()
        .join("\n");

    // ── Grid ordering puzzle ─────────────────────────────────────────────────
    let grid_size = cfg.grid_size;
    let num_coords = rng.gen_range(cfg.grid_coords_min..=cfg.grid_coords_max);

    let mut all_cells: Vec<GridCoord> = (0..grid_size)
        .flat_map(|row| {
            (0..grid_size).map(move |col| GridCoord {
                col: col as u8,
                row: row as u8,
            })
        })
        .collect();
    all_cells.shuffle(&mut rng);
    let selected: Vec<GridCoord> = all_cells.into_iter().take(num_coords).collect();

    // Correct solution: reading order (row asc, then col asc)
    let mut grid_solution = selected.clone();
    grid_solution.sort_by(|a, b| a.row.cmp(&b.row).then(a.col.cmp(&b.col)));

    // Jumbled version for the description
    let mut grid_coords_jumbled = selected;
    loop {
        grid_coords_jumbled.shuffle(&mut rng);
        if grid_coords_jumbled != grid_solution {
            break;
        }
    }

    let grid_coords_text = generate_grid_description(&grid_coords_jumbled, &mut rng);

    // ── Store challenge ──────────────────────────────────────────────────────
    let challenge_id = Uuid::new_v4().to_string();
    let created_at = now_ms();

    let challenge = Challenge {
        word_frequencies,
        grid_solution,
        created_at,
        sitekey: sitekey.clone(),
    };

    drop(state_guard);

    state
        .write()
        .unwrap()
        .challenges
        .insert(challenge_id.clone(), challenge);

    info!(
        "Created challenge {} (sitekey={:?}) — {} words ({} unique), {} grid coords",
        challenge_id,
        if sitekey.is_empty() { "demo" } else { &sitekey },
        word_count,
        unique_word_count,
        num_coords
    );

    Ok(Json(ChallengeResponse {
        challenge_id,
        text_content,
        duration_ms,
        grid_size: grid_size as u8,
        grid_coords_text,
    }))
}

// ── POST /api/submit ──────────────────────────────────────────────────────────

/// Validate both puzzle answers. Returns a signed token on success.
pub async fn submit_answer(
    State(state): State<AppState>,
    Json(payload): Json<SubmitRequest>,
) -> Json<SubmitResponse> {
    let now = now_ms();

    let state_guard = state.read().unwrap();
    let Some(challenge) = state_guard.challenges.get(&payload.challenge_id) else {
        return Json(SubmitResponse {
            success: false,
            message: "Challenge not found".to_string(),
            score: None,
            word_score: None,
            grid_score: None,
            token: None,
        });
    };

    let validation_cfg = &state_guard.config.validation;

    // ── Timing check ─────────────────────────────────────────────────────────
    let elapsed = now - challenge.created_at;
    if validation_cfg.min_time_ms > 0 && elapsed < validation_cfg.min_time_ms {
        return Json(SubmitResponse {
            success: false,
            message: format!(
                "Too fast: {}ms (minimum {}ms)",
                elapsed, validation_cfg.min_time_ms
            ),
            score: None,
            word_score: None,
            grid_score: None,
            token: None,
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
            word_score: None,
            grid_score: None,
            token: None,
        });
    }

    // ── Score both puzzles ────────────────────────────────────────────────────
    let word_score = score_word_frequencies(&payload.answer, &challenge.word_frequencies);
    let grid_score = score_grid_answer(&payload.grid_answer, &challenge.grid_solution);

    let threshold = validation_cfg.success_threshold;
    let success = word_score >= threshold && grid_score >= threshold;
    let combined_score = (word_score + grid_score) / 2.0;

    let threshold_pct = threshold * 100.0;
    let message = if success {
        format!(
            "Success! Word: {:.1}%, Grid: {:.1}%",
            word_score * 100.0,
            grid_score * 100.0
        )
    } else {
        let mut parts = Vec::new();
        if word_score < threshold {
            parts.push(format!(
                "Word {:.1}% (need {:.0}%)",
                word_score * 100.0,
                threshold_pct
            ));
        }
        if grid_score < threshold {
            parts.push(format!(
                "Grid {:.1}% (need {:.0}%)",
                grid_score * 100.0,
                threshold_pct
            ));
        }
        format!("Failed: {}", parts.join(", "))
    };

    info!(
        "Challenge {} — {}ms — word {:.1}% grid {:.1}%: {}",
        payload.challenge_id,
        elapsed,
        word_score * 100.0,
        grid_score * 100.0,
        if success { "PASSED" } else { "FAILED" }
    );

    // ── Mint token on success ─────────────────────────────────────────────────
    let token = if success {
        let auth_cfg = &state_guard.config.auth;
        let token_payload = TokenPayload {
            challenge_id: payload.challenge_id.clone(),
            sitekey: challenge.sitekey.clone(),
            score: combined_score,
            iat: now,
            exp: now + auth_cfg.token_ttl_secs * 1000,
        };
        Some(mint_token(&token_payload, &auth_cfg.token_secret))
    } else {
        None
    };

    Json(SubmitResponse {
        success,
        message,
        score: Some(combined_score),
        word_score: Some(word_score),
        grid_score: Some(grid_score),
        token,
    })
}

// ── POST /api/verify ──────────────────────────────────────────────────────────

/// Called by the host's backend to confirm a token is genuine and unused.
/// Tokens are single-use — once verified, a replay attempt returns invalid.
pub async fn verify_token(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    let now = now_ms();

    let secret = {
        let guard = state.read().unwrap();
        guard.config.auth.token_secret.clone()
    };

    // Decode and verify signature
    let claims = match decode_token(&payload.token, &secret) {
        Some(c) => c,
        None => {
            return Json(VerifyResponse {
                valid: false,
                message: "Invalid token signature".to_string(),
                sitekey: None,
                score: None,
            });
        }
    };

    // Check expiry
    if now > claims.exp {
        return Json(VerifyResponse {
            valid: false,
            message: "Token has expired".to_string(),
            sitekey: Some(claims.sitekey),
            score: Some(claims.score),
        });
    }

    // Optional: assert sitekey matches what the host expects
    if let Some(expected_key) = &payload.sitekey {
        if *expected_key != claims.sitekey {
            return Json(VerifyResponse {
                valid: false,
                message: "Sitekey mismatch".to_string(),
                sitekey: Some(claims.sitekey),
                score: Some(claims.score),
            });
        }
    }

    // Check one-time use: mark token consumed via its challenge_id
    {
        let mut guard = state.write().unwrap();
        if guard.used_tokens.contains(&claims.challenge_id) {
            return Json(VerifyResponse {
                valid: false,
                message: "Token has already been used".to_string(),
                sitekey: Some(claims.sitekey),
                score: Some(claims.score),
            });
        }
        guard.used_tokens.insert(claims.challenge_id.clone());
    }

    info!(
        "Token verified — challenge_id={} sitekey={:?} score={:.1}%",
        claims.challenge_id,
        if claims.sitekey.is_empty() { "demo" } else { &claims.sitekey },
        claims.score * 100.0
    );

    Json(VerifyResponse {
        valid: true,
        message: "Token is valid".to_string(),
        sitekey: Some(claims.sitekey),
        score: Some(claims.score),
    })
}
