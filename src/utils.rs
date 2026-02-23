use std::collections::HashMap;

/// Score how well the submitted word frequencies match the correct answer.
/// Returns a value between 0.0 (nothing correct) and 1.0 (all word counts exact).
/// Extra words in the submission that don't appear in correct are ignored.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(pairs: &[(&str, usize)]) -> HashMap<String, usize> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_perfect_match() {
        let correct = make_map(&[("hello", 3), ("world", 2)]);
        assert_eq!(score_word_frequencies(&correct, &correct), 1.0);
    }

    #[test]
    fn test_half_match() {
        let correct = make_map(&[("hello", 3), ("world", 2)]);
        let submitted = make_map(&[("hello", 3), ("world", 5)]); // wrong count for "world"
        assert_eq!(score_word_frequencies(&submitted, &correct), 0.5);
    }

    #[test]
    fn test_missing_word() {
        let correct = make_map(&[("hello", 3), ("world", 2)]);
        let submitted = make_map(&[("hello", 3)]); // "world" absent = wrong
        assert_eq!(score_word_frequencies(&submitted, &correct), 0.5);
    }

    #[test]
    fn test_extra_words_ignored() {
        let correct = make_map(&[("hello", 3)]);
        let submitted = make_map(&[("hello", 3), ("extra", 99)]);
        assert_eq!(score_word_frequencies(&submitted, &correct), 1.0);
    }

    #[test]
    fn test_empty_correct() {
        let correct: HashMap<String, usize> = HashMap::new();
        let submitted: HashMap<String, usize> = HashMap::new();
        assert_eq!(score_word_frequencies(&submitted, &correct), 1.0);
    }

    #[test]
    fn test_all_wrong() {
        let correct = make_map(&[("foo", 1), ("bar", 2)]);
        let submitted = make_map(&[("foo", 99), ("bar", 99)]);
        assert_eq!(score_word_frequencies(&submitted, &correct), 0.0);
    }
}
