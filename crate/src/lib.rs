use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

// ============================================================
// Per-target static data (embedded at compile time)
// ============================================================

struct TargetData {
    name: &'static str,
    words_raw: &'static str,
    distances_raw: &'static str,
    start_words_raw: &'static str,
}

const TARGETS: &[TargetData] = &[
    TargetData {
        name: "pipi",
        words_raw: include_str!("../../data/words_pipi.txt"),
        distances_raw: include_str!("../../data/distances_pipi.csv"),
        start_words_raw: include_str!("../../data/start_words_pipi.csv"),
    },
    TargetData {
        name: "caca",
        words_raw: include_str!("../../data/words_caca.txt"),
        distances_raw: include_str!("../../data/distances_caca.csv"),
        start_words_raw: include_str!("../../data/start_words_caca.csv"),
    },
    TargetData {
        name: "vomi",
        words_raw: include_str!("../../data/words_vomi.txt"),
        distances_raw: include_str!("../../data/distances_vomi.csv"),
        start_words_raw: include_str!("../../data/start_words_vomi.csv"),
    },
];

/// Epoch: 2026-07-01 08:00 UTC (milliseconds)
const EPOCH_MS: u64 = 1_751_356_800_000;
const DAY_MS: u64 = 86_400_000;

// ============================================================
// Parsed game data for one target
// ============================================================

struct GameData {
    target: String,
    words: HashSet<String>,
    distances: HashMap<String, u32>,
    start_words: Vec<(String, u32)>,
}

impl GameData {
    fn load(td: &TargetData) -> Self {
        let words: HashSet<String> = td
            .words_raw
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| l.len() == 4)
            .collect();

        let distances: HashMap<String, u32> = td
            .distances_raw
            .lines()
            .filter_map(|l| {
                let mut parts = l.trim().splitn(2, ',');
                let word = parts.next()?.to_lowercase();
                let dist: u32 = parts.next()?.parse().ok()?;
                Some((word, dist))
            })
            .collect();

        let start_words: Vec<(String, u32)> = td
            .start_words_raw
            .lines()
            .filter_map(|l| {
                let mut parts = l.trim().splitn(2, ',');
                let word = parts.next()?.to_lowercase();
                let dist: u32 = parts.next()?.parse().ok()?;
                Some((word, dist))
            })
            .collect();

        GameData {
            target: td.name.to_string(),
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

// ============================================================
// All three targets, loaded once
// ============================================================

struct AllData {
    pipi: GameData,
    caca: GameData,
    vomi: GameData,
}

impl AllData {
    fn load() -> Self {
        AllData {
            pipi: GameData::load(&TARGETS[0]),
            caca: GameData::load(&TARGETS[1]),
            vomi: GameData::load(&TARGETS[2]),
        }
    }

    fn get(&self, mode: &str) -> &GameData {
        match mode {
            "caca" => &self.caca,
            "vomi" => &self.vomi,
            _ => &self.pipi,
        }
    }
}

thread_local! {
    static DATA: AllData = AllData::load();
}

// ============================================================
// Helpers
// ============================================================

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

fn get_shortest_path(data: &GameData, start_word: &str) -> Vec<String> {
    let target = &data.target;
    let start_dist = match data.get_distance(start_word) {
        Some(d) => d as usize,
        None => return vec![start_word.to_string()],
    };

    // layers[i] maps word -> list of predecessors (words at layer i-1 that lead to it)
    let mut layers: Vec<HashMap<String, Vec<String>>> = Vec::with_capacity(start_dist + 1);

    // Layer 0: target word
    let mut layer0 = HashMap::new();
    layer0.insert(target.to_string(), vec![]);
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

    // Traverse from start_word back to target
    let start = start_word.to_lowercase();
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
// WASM-exported API — all functions take a `mode` parameter
// ============================================================

#[wasm_bindgen]
pub fn init() {
    DATA.with(|_| {});
}

#[wasm_bindgen]
pub fn get_target(mode: &str) -> String {
    mode.to_lowercase().chars().take(4).collect::<String>().to_uppercase()
}

/// Returns a JSON array of available modes: ["pipi","caca","vomi"]
#[wasm_bindgen]
pub fn get_modes() -> String {
    r#"["pipi","caca","vomi"]"#.to_string()
}

#[wasm_bindgen]
pub fn is_in_word_list(mode: &str, word: &str) -> bool {
    DATA.with(|d| d.get(mode).is_in_word_list(word))
}

#[wasm_bindgen]
pub fn is_valid_move(mode: &str, word: &str, previous: &str) -> bool {
    let w = word.to_lowercase();
    let p = previous.to_lowercase();
    DATA.with(|d| d.get(mode).is_in_word_list(&w)) && w.len() == 4 && one_letter_different(&w, &p)
}

#[wasm_bindgen]
pub fn get_distance(mode: &str, word: &str) -> i32 {
    DATA.with(|d| d.get(mode).get_distance(word).map(|v| v as i32).unwrap_or(-1))
}

#[wasm_bindgen]
pub fn get_start_word(mode: &str, now_ms: f64) -> String {
    DATA.with(|d| {
        let gd = d.get(mode);
        let day = day_number(now_ms as u64);
        let idx = (day as usize) % gd.start_words.len();
        gd.start_words[idx].0.clone()
    })
}

#[wasm_bindgen]
pub fn get_start_word_distance(mode: &str, now_ms: f64) -> i32 {
    DATA.with(|d| {
        let gd = d.get(mode);
        let day = day_number(now_ms as u64);
        let idx = (day as usize) % gd.start_words.len();
        gd.start_words[idx].1 as i32
    })
}

#[wasm_bindgen]
pub fn get_day_number(now_ms: f64) -> u32 {
    day_number(now_ms as u64)
}

#[wasm_bindgen]
pub fn get_optimal_path(mode: &str, start_word: &str) -> String {
    DATA.with(|d| {
        let path = get_shortest_path(d.get(mode), start_word);
        serde_json::to_string(&path).unwrap_or_else(|_| "[]".to_string())
    })
}

#[wasm_bindgen]
pub fn word_count(mode: &str) -> usize {
    DATA.with(|d| d.get(mode).words.len())
}

#[wasm_bindgen]
pub fn validate_error(mode: &str, word: &str, previous: &str) -> String {
    let w = word.to_lowercase();
    let p = previous.to_lowercase();
    if w.len() != 4 {
        return String::new();
    }
    let in_list = DATA.with(|d| d.get(mode).is_in_word_list(&w));
    if !in_list {
        return "Mot inconnu".to_string();
    }
    if !one_letter_different(&w, &p) {
        return "Une seule lettre doit changer".to_string();
    }
    String::new()
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> AllData {
        AllData::load()
    }

    #[test]
    fn test_data_loads_all_targets() {
        let data = all();
        for mode in &["pipi", "caca", "vomi"] {
            let gd = data.get(mode);
            assert!(
                gd.words.len() > 1000,
                "{}: should have >1000 words, got {}",
                mode,
                gd.words.len()
            );
            assert!(gd.words.contains(*mode), "{mode} should be in its own word list");
            assert_eq!(gd.distances[*mode], 0, "{mode} should be at distance 0");
            assert!(!gd.start_words.is_empty(), "{mode} should have start words");
        }
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
        let data = all();
        for mode in &["pipi", "caca", "vomi"] {
            let gd = data.get(mode);
            let adj = gd.get_adjacent_words(mode);
            assert!(!adj.is_empty(), "{mode} should have neighbors");
            for a in &adj {
                assert!(one_letter_different(mode, a));
                assert!(gd.words.contains(a));
            }
        }
    }

    #[test]
    fn test_get_distance_all() {
        let data = all();
        for mode in &["pipi", "caca", "vomi"] {
            let gd = data.get(mode);
            assert_eq!(gd.get_distance(mode), Some(0));
            for (w, d) in &gd.distances {
                if *d == 1 {
                    assert!(
                        one_letter_different(mode, w),
                        "{w} at distance 1 from {mode} should be adjacent"
                    );
                }
            }
        }
    }

    #[test]
    fn test_start_words_valid_all() {
        let data = all();
        for mode in &["pipi", "caca", "vomi"] {
            let gd = data.get(mode);
            for (word, dist) in &gd.start_words {
                assert!(
                    gd.words.contains(word),
                    "{mode}: start word {word} should be in word list"
                );
                assert_eq!(
                    gd.get_distance(word),
                    Some(*dist),
                    "{mode}: start word {word} distance mismatch"
                );
                assert!(
                    *dist >= 4 && *dist <= 8,
                    "{mode}: start word {word} distance {dist} out of range"
                );
            }
        }
    }

    #[test]
    fn test_no_duplicate_start_words() {
        let data = all();
        for mode in &["pipi", "caca", "vomi"] {
            let gd = data.get(mode);
            let mut seen = HashSet::new();
            for (word, _) in &gd.start_words {
                assert!(seen.insert(word.clone()), "{mode}: duplicate start word {word}");
            }
        }
    }

    #[test]
    fn test_shortest_path_all() {
        let data = all();
        for mode in &["pipi", "caca", "vomi"] {
            let gd = data.get(mode);
            if let Some((word, dist)) = gd.start_words.first() {
                let path = get_shortest_path(gd, word);
                assert_eq!(
                    path.len(),
                    (*dist as usize) + 1,
                    "{mode}: path length should be distance + 1"
                );
                assert_eq!(path.first().unwrap(), word);
                assert_eq!(path.last().unwrap(), *mode);
                for pair in path.windows(2) {
                    assert!(
                        one_letter_different(&pair[0], &pair[1]),
                        "{mode}: {} -> {} should differ by one letter",
                        pair[0],
                        pair[1]
                    );
                }
            }
        }
    }
}
