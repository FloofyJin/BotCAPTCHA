use std::collections::HashMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

use crate::models::{GridCoord, TokenPayload};

type HmacSha256 = Hmac<Sha256>;

// ── Token helpers ──────────────────────────────────────────────────────────────

/// Mint a signed token string from a payload.
/// Format: `<base64url(JSON)>.<base64url(HMAC-SHA256 signature)>`
pub fn mint_token(payload: &TokenPayload, secret: &str) -> String {
    let json = serde_json::to_string(payload).expect("token payload serialisation");
    let b64 = URL_SAFE_NO_PAD.encode(json.as_bytes());

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(b64.as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes().as_slice());

    format!("{}.{}", b64, sig)
}

/// Verify the signature of a token and return its payload if valid.
/// Returns `None` if the token is malformed or the signature doesn't match.
pub fn decode_token(token: &str, secret: &str) -> Option<TokenPayload> {
    let dot = token.find('.')?;
    let b64 = &token[..dot];
    let sig = &token[dot + 1..];

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(b64.as_bytes());
    let expected_sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes().as_slice());

    if sig != expected_sig {
        return None;
    }

    let json_bytes = URL_SAFE_NO_PAD.decode(b64).ok()?;
    serde_json::from_slice(&json_bytes).ok()
}

fn describe_coord(c: &GridCoord, rng: &mut impl Rng) -> String {
    let col = c.col;
    let row = c.row;
    match rng.gen_range(0u8..5) {
        0 => format!("column {}, row {}", col, row),
        1 => format!("the cell at column {} and row {}", col, row),
        2 => format!("the cell where column {} meets row {}", col, row),
        3 => format!("the square at column {} in row {}", col, row),
        _ => format!("{} columns from the left, {} rows from the top", col, row),
    }
}

/// Generate a natural-language paragraph describing the target grid cells in the
/// given (jumbled) order. The phrasing varies per coordinate so that a simple
/// regex or pattern matcher cannot reliably extract all of them — an LLM is
/// needed to parse the full passage.
pub fn generate_grid_description(coords: &[GridCoord], rng: &mut impl Rng) -> String {
    let n = coords.len();

    // Include the exact count so the LLM knows how many coordinates to extract
    let opening = match rng.gen_range(0u8..5) {
        0 => format!("The {} cells you need to select are located at: ", n),
        1 => format!("Select exactly {} squares found at ", n),
        2 => format!("Identify these {} positions on the grid: ", n),
        3 => format!("Mark the following {} cells: ", n),
        _ => format!("Your {} target squares can be found at ", n),
    };

    let transitions: &[&str] = &[
        ", then ",
        "; next, ",
        ", followed by ",
        ". Also mark ",
        ", and then ",
        "; continuing with ",
        ". Another target is at ",
        ", as well as ",
        ". Do not overlook ",
    ];

    let mut result = opening;

    for (i, coord) in coords.iter().enumerate() {
        if i > 0 {
            let t = transitions[rng.gen_range(0..transitions.len())];
            result.push_str(t);
        }
        result.push_str(&describe_coord(coord, rng));
    }

    result.push_str(
        ". Note: the positions above are listed in scrambled order — \
        you must select them on the grid from top to bottom, left to right.",
    );
    result
}

/// Score how well the submitted word frequencies match the correct answer.
/// Returns 0.0–1.0. Extra words in the submission that don't appear in
/// the correct map are ignored.
pub fn score_word_frequencies(
    submitted: &HashMap<String, usize>,
    correct: &HashMap<String, usize>,
) -> f32 {
    if correct.is_empty() {
        return 1.0;
    }

    let matched = correct
        .iter()
        .filter(|(word, count)| submitted.get(*word) == Some(count))
        .count();

    matched as f32 / correct.len() as f32
}

/// Score the submitted grid ordering against the correct reading-order solution.
/// The submitted list must have the same length and each position must match exactly.
/// Returns 0.0–1.0.
pub fn score_grid_answer(submitted: &[GridCoord], correct: &[GridCoord]) -> f32 {
    if correct.is_empty() {
        return 1.0;
    }
    if submitted.len() != correct.len() {
        return 0.0;
    }

    let matched = submitted
        .iter()
        .zip(correct.iter())
        .filter(|(s, c)| s == c)
        .count();

    matched as f32 / correct.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn coord(col: u8, row: u8) -> GridCoord {
        GridCoord { col, row }
    }

    // ── word frequency tests ────────────────────────────────────

    #[test]
    fn test_word_perfect_match() {
        let correct = make_map(&[("hello", 3), ("world", 2)]);
        assert_eq!(score_word_frequencies(&correct, &correct), 1.0);
    }

    #[test]
    fn test_word_half_match() {
        let correct = make_map(&[("hello", 3), ("world", 2)]);
        let submitted = make_map(&[("hello", 3), ("world", 5)]);
        assert_eq!(score_word_frequencies(&submitted, &correct), 0.5);
    }

    #[test]
    fn test_word_missing_word() {
        let correct = make_map(&[("hello", 3), ("world", 2)]);
        let submitted = make_map(&[("hello", 3)]);
        assert_eq!(score_word_frequencies(&submitted, &correct), 0.5);
    }

    #[test]
    fn test_word_extra_words_ignored() {
        let correct = make_map(&[("hello", 3)]);
        let submitted = make_map(&[("hello", 3), ("extra", 99)]);
        assert_eq!(score_word_frequencies(&submitted, &correct), 1.0);
    }

    #[test]
    fn test_word_empty_correct() {
        let correct: HashMap<String, usize> = HashMap::new();
        assert_eq!(score_word_frequencies(&HashMap::new(), &correct), 1.0);
    }

    // ── grid answer tests ───────────────────────────────────────

    #[test]
    fn test_grid_perfect_match() {
        let correct = vec![coord(0, 0), coord(3, 1), coord(7, 2)];
        assert_eq!(score_grid_answer(&correct, &correct), 1.0);
    }

    #[test]
    fn test_grid_wrong_order() {
        let correct = vec![coord(0, 0), coord(3, 1), coord(7, 2)];
        let submitted = vec![coord(7, 2), coord(0, 0), coord(3, 1)]; // same cells, wrong order
        assert_eq!(score_grid_answer(&submitted, &correct), 0.0);
    }

    #[test]
    fn test_grid_partial_order() {
        let correct = vec![coord(0, 0), coord(3, 1), coord(7, 2)];
        let submitted = vec![coord(0, 0), coord(7, 2), coord(3, 1)]; // first right, rest wrong
        let score = score_grid_answer(&submitted, &correct);
        assert!((score - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_grid_wrong_length() {
        let correct = vec![coord(0, 0), coord(3, 1)];
        let submitted = vec![coord(0, 0)];
        assert_eq!(score_grid_answer(&submitted, &correct), 0.0);
    }

    #[test]
    fn test_grid_empty_correct() {
        assert_eq!(score_grid_answer(&[], &[]), 1.0);
    }
}
