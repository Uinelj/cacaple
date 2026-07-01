use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

const TARGET: &str = "pipi";
const WORDS_DATA: &str = include_str!("../../data/words.txt");
const DISTANCES_DATA: &str = include_str!("../../data/distances.csv");
const START_WORDS_DATA: &str = include_str!("../../data/start_words.csv");

/// Epoch: 2026-07-01 08:00 UTC (milliseconds)
const EPOCH_MS: u64 = 1_751_356_800_000;
const DAY_MS: u64 = 86_400_000;

struct GameData {
    words: HashSet<String>,
    distances: HashMap<String, u32>,
    start_words: Vec<(String, u32)>,
}

impl GameData {
    fn load() -> Self {
        let words: HashSet<String> = WORDS_DATA
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| l.len() == 4)
            .collect();

        let distances: HashMap<String, u32> = DISTANCES_DATA
            .lines()
            .filter_map(|l| {
                let mut parts = l.trim().splitn(2, ',');
                let word = parts.next()?.to_lowercase();
                let dist: u32 = parts.next()?.parse().ok()?;
                Some((word, dist))
            })
            .collect();

        let start_words: Vec<(String, u32)> = START_WORDS_DATA
            .lines()
            .filter_map(|l| {
                let mut parts = l.trim().splitn(2, ',');
                let word = parts.next()?.to_lowercase();
                let dist: u32 = parts.next()?.parse().ok()?;
                Some((word, dist))
            })
            .collect();

        GameData {
            words,
            distances,
            start_words,
        }
    }

    fn is_in_word_list(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }

    fn get_distance(&self, word: &str) -> Option<u32> {
        self.distances.get(&word.to_lowercase()).copied()
    }

    fn get_adjacent_words(&self, word: &str) -> Vec<String> {
        let word = word.to_lowercase();
        let bytes = word.as_bytes();
        let mut result = Vec::new();
        for i in 0..4 {
            for c in b'a'..=b'z' {
                if bytes[i] == c {
                    continue;
                }
                let mut candidate = bytes.to_vec();
                candidate[i] = c;
                let s = String::from_utf8(candidate).unwrap();
                if self.words.contains(&s) {
                    result.push(s);
                }
            }
        }
        result
    }
}

fn one_letter_different(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let diff = a
        .chars()
        .zip(b.chars())
        .filter(|(ca, cb)| ca.to_lowercase().next() != cb.to_lowercase().next())
        .count();
    diff == 1
}

fn day_number(now_ms: u64) -> u32 {
    if now_ms <= EPOCH_MS {
        return 0;
    }
    ((now_ms - EPOCH_MS) / DAY_MS) as u32
}

// --- Build a shortest-path tree and find a good path ---

fn build_tree(data: &GameData, start_word: &str) -> Vec<HashMap<String, Vec<String>>> {
    let start_dist = match data.get_distance(start_word) {
        Some(d) => d as usize,
        None => return vec![],
    };

    // layers[i] maps word -> list of predecessors (words at layer i-1 that lead to it)
    let mut layers: Vec<HashMap<String, Vec<String>>> = Vec::with_capacity(start_dist + 1);

    // Layer 0: target word
    let mut layer0 = HashMap::new();
    layer0.insert(TARGET.to_string(), vec![]);
    layers.push(layer0);

    // Build layers 1..start_dist-1
    for d in 1..start_dist {
        let mut layer = HashMap::new();
        for word in layers[d - 1].keys() {
            for adj in data.get_adjacent_words(word) {
                if data.get_distance(&adj) == Some(d as u32) {
                    layer.entry(adj).or_insert_with(Vec::new).push(word.clone());
                }
            }
        }
        layers.push(layer);
    }

    // Last layer: the start word itself
    let mut last_layer = HashMap::new();
    let mut preds = vec![];
    if start_dist > 0 {
        for word in layers[start_dist - 1].keys() {
            if one_letter_different(word, start_word) {
                preds.push(word.clone());
            }
        }
    }
    last_layer.insert(start_word.to_lowercase(), preds);
    layers.push(last_layer);

    // Prune: remove words that don't lead forward
    for d in (1..start_dist).rev() {
        let next_layer_words: HashSet<String> = layers[d + 1]
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect();
        layers[d].retain(|k, _| next_layer_words.contains(k));
    }

    layers
}

fn get_shortest_path(data: &GameData, start_word: &str) -> Vec<String> {
    let layers = build_tree(data, start_word);
    if layers.is_empty() {
        return vec![start_word.to_string()];
    }

    let start = start_word.to_lowercase();

    // Traverse from start_word back to target
    let mut path = vec![start.clone()];
    let total = layers.len();

    for d in (0..total - 1).rev() {
        let current = path.last().unwrap().clone();
        if let Some(preds) = layers[d + 1].get(&current) {
            if let Some(pred) = preds.first() {
                path.push(pred.clone());
            }
        }
    }

    path
}

// ============================================================
// WASM-exported API
// ============================================================

thread_local! {
    static DATA: GameData = GameData::load();
}

#[wasm_bindgen]
pub fn init() {
    // Force lazy init
    DATA.with(|_| {});
}

#[wasm_bindgen]
pub fn get_target() -> String {
    TARGET.to_uppercase()
}

#[wasm_bindgen]
pub fn is_in_word_list(word: &str) -> bool {
    DATA.with(|d| d.is_in_word_list(word))
}

#[wasm_bindgen]
pub fn is_valid_move(word: &str, previous: &str) -> bool {
    let w = word.to_lowercase();
    let p = previous.to_lowercase();
    DATA.with(|d| d.is_in_word_list(&w)) && w.len() == 4 && one_letter_different(&w, &p)
}

#[wasm_bindgen]
pub fn get_distance(word: &str) -> i32 {
    DATA.with(|d| d.get_distance(word).map(|v| v as i32).unwrap_or(-1))
}

#[wasm_bindgen]
pub fn get_start_word(now_ms: f64) -> String {
    DATA.with(|d| {
        let day = day_number(now_ms as u64);
        let idx = (day as usize) % d.start_words.len();
        d.start_words[idx].0.clone()
    })
}

#[wasm_bindgen]
pub fn get_start_word_distance(now_ms: f64) -> i32 {
    DATA.with(|d| {
        let day = day_number(now_ms as u64);
        let idx = (day as usize) % d.start_words.len();
        d.start_words[idx].1 as i32
    })
}

#[wasm_bindgen]
pub fn get_day_number(now_ms: f64) -> u32 {
    day_number(now_ms as u64)
}

/// Get the optimal shortest path as a JSON array of strings.
#[wasm_bindgen]
pub fn get_optimal_path(start_word: &str) -> String {
    DATA.with(|d| {
        let path = get_shortest_path(d, start_word);
        serde_json::to_string(&path).unwrap_or_else(|_| "[]".to_string())
    })
}

#[wasm_bindgen]
pub fn word_count() -> usize {
    DATA.with(|d| d.words.len())
}

#[wasm_bindgen]
pub fn validate_error(word: &str, previous: &str) -> String {
    let w = word.to_lowercase();
    let p = previous.to_lowercase();
    if w.len() != 4 {
        return String::new();
    }
    let in_list = DATA.with(|d| d.is_in_word_list(&w));
    if !in_list {
        return "Mot inconnu".to_string();
    }
    if !one_letter_different(&w, &p) {
        return "Une seule lettre doit changer".to_string();
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_loads() {
        let data = GameData::load();
        assert!(data.words.len() > 2000, "Should have >2000 words");
        assert!(data.words.contains("pipi"));
        assert!(data.distances.contains_key("pipi"));
        assert_eq!(data.distances["pipi"], 0);
        assert!(!data.start_words.is_empty());
    }

    #[test]
    fn test_one_letter_different() {
        assert!(one_letter_different("pipi", "pipa"));
        assert!(one_letter_different("pipi", "tipi"));
        assert!(!one_letter_different("pipi", "papa"));
        assert!(!one_letter_different("pipi", "pipi"));
        assert!(!one_letter_different("pip", "pipi"));
    }

    #[test]
    fn test_adjacency() {
        let data = GameData::load();
        let adj = data.get_adjacent_words("pipi");
        assert!(!adj.is_empty(), "PIPI should have neighbors");
        for a in &adj {
            assert!(one_letter_different("pipi", a));
            assert!(data.words.contains(a));
        }
    }

    #[test]
    fn test_get_distance() {
        let data = GameData::load();
        assert_eq!(data.get_distance("pipi"), Some(0));
        for (w, d) in &data.distances {
            if *d == 1 {
                assert!(
                    one_letter_different("pipi", w),
                    "{w} at distance 1 should be adjacent to pipi"
                );
            }
        }
    }

    #[test]
    fn test_start_words_valid() {
        let data = GameData::load();
        for (word, dist) in &data.start_words {
            assert!(
                data.words.contains(word),
                "Start word {word} should be in word list"
            );
            assert_eq!(
                data.get_distance(word),
                Some(*dist),
                "Start word {word} distance mismatch"
            );
            assert!(
                *dist >= 4 && *dist <= 8,
                "Start word {word} distance {dist} out of range"
            );
        }
    }

    #[test]
    fn test_shortest_path() {
        let data = GameData::load();
        if let Some((word, dist)) = data.start_words.first() {
            let path = get_shortest_path(&data, word);
            assert_eq!(
                path.len(),
                (*dist as usize) + 1,
                "Path length should be distance + 1"
            );
            assert_eq!(path.first().unwrap(), word);
            assert_eq!(path.last().unwrap(), TARGET);
            for pair in path.windows(2) {
                assert!(
                    one_letter_different(&pair[0], &pair[1]),
                    "Adjacent path words should differ by one letter: {} -> {}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }
}
