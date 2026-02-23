use axum::{extract::State, response::Json};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use uuid::Uuid;

use crate::models::*;
use crate::utils::{generate_grid_description, score_grid_answer, score_word_frequencies};

/// GET /api/challenge - Generate a combined word-frequency + grid-ordering challenge
pub async fn create_challenge(State(state): State<AppState>) -> Json<ChallengeResponse> {
    let mut rng = rand::thread_rng();

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

    // Build all cells and pick a random subset
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

    // Jumbled version for the description — shuffle until it differs from the solution
    let mut grid_coords_jumbled = selected;
    loop {
        grid_coords_jumbled.shuffle(&mut rng);
        if grid_coords_jumbled != grid_solution {
            break;
        }
    }

    // Convert to natural language; structured coords are never sent to the client
    let grid_coords_text = generate_grid_description(&grid_coords_jumbled, &mut rng);

    // ── Store challenge ──────────────────────────────────────────────────────
    let challenge_id = Uuid::new_v4().to_string();
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let challenge = Challenge {
        word_frequencies,
        grid_solution,
        created_at,
    };

    drop(state_guard);

    state
        .write()
        .unwrap()
        .challenges
        .insert(challenge_id.clone(), challenge);

    info!(
        "Created challenge {} — {} words ({} unique), {} grid coords",
        challenge_id, word_count, unique_word_count, num_coords
    );

    Json(ChallengeResponse {
        challenge_id,
        text_content,
        duration_ms,
        grid_size: grid_size as u8,
        grid_coords_text,
    })
}

/// POST /api/submit - Validate both puzzle answers
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
            word_score: None,
            grid_score: None,
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
        });
    }

    // ── Score both puzzles ────────────────────────────────────────────────────
    let word_score =
        score_word_frequencies(&payload.answer, &challenge.word_frequencies);
    let grid_score =
        score_grid_answer(&payload.grid_answer, &challenge.grid_solution);

    let threshold = validation_cfg.success_threshold;
    let word_ok = word_score >= threshold;
    let grid_ok = grid_score >= threshold;
    let success = word_ok && grid_ok;

    // Combined score is the average of both
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
        if !word_ok {
            parts.push(format!(
                "Word {:.1}% (need {:.0}%)",
                word_score * 100.0,
                threshold_pct
            ));
        }
        if !grid_ok {
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

    Json(SubmitResponse {
        success,
        message,
        score: Some(combined_score),
        word_score: Some(word_score),
        grid_score: Some(grid_score),
    })
}
