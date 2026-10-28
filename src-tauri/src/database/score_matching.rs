use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use super::operations::{Map, MapDiff, PlayerScore};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentDiffInfo {
    pub mode: String,
    pub difficulty: String,
    pub hash: String,
    pub stars: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreRowData {
    pub mode: String,
    pub difficulty: String,
    pub is_current_version: bool,
    pub stars: Option<f32>,
    pub scores: Vec<PlayerScoreCell>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerScoreCell {
    pub player_id: String,
    pub score_id: i64,
    pub accuracy: f32,
    pub is_expired: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapScoresResponse {
    pub map: Map,
    pub current_diffs: Vec<CurrentDiffInfo>,
    pub scores: Vec<ScoreRowData>,
}

/// Matching is pure: queries are batched by the caller. A current-version score
/// takes priority even when an older version has a higher accuracy.
pub fn build_map_scores_response(map: Map, diffs: &[MapDiff], scores: &[PlayerScore]) -> MapScoresResponse {
    let current: HashSet<_> = diffs.iter().filter(|d| d.is_current)
        .map(|d| (d.hash.to_lowercase(), d.mode.clone(), d.difficulty.clone())).collect();
    let mut rows: HashMap<(String, String), ScoreRowData> = HashMap::new();
    let mut current_diffs = Vec::new();
    for diff in diffs.iter().filter(|d| d.is_current) {
        current_diffs.push(CurrentDiffInfo {
            mode: diff.mode.clone(), difficulty: diff.difficulty.clone(),
            hash: diff.hash.clone(), stars: diff.stars_rating,
        });
        rows.entry((diff.mode.clone(), diff.difficulty.clone())).or_insert_with(|| ScoreRowData {
            mode: diff.mode.clone(), difficulty: diff.difficulty.clone(),
            is_current_version: true, stars: diff.stars_rating, scores: Vec::new(),
        });
    }
    let mut best: HashMap<(String, String, String), PlayerScoreCell> = HashMap::new();
    for score in scores {
        let expired = !current.contains(&(score.hash.to_lowercase(), score.mode.clone(), score.difficulty.clone()));
        let cell = best.entry((score.mode.clone(), score.difficulty.clone(), score.player_id.clone()))
            .or_insert_with(|| PlayerScoreCell {
                player_id: score.player_id.clone(), score_id: score.score_id, accuracy: score.accuracy, is_expired: expired,
            });
        if (cell.is_expired && !expired) || (cell.is_expired == expired && score.accuracy > cell.accuracy) {
            cell.accuracy = score.accuracy;
            cell.score_id = score.score_id;
            cell.is_expired = expired;
        }
    }
    for ((mode, difficulty, _), cell) in best {
        rows.entry((mode.clone(), difficulty.clone())).or_insert_with(|| ScoreRowData {
            mode, difficulty, is_current_version: false, stars: None, scores: Vec::new(),
        }).scores.push(cell);
    }
    let mut scores: Vec<_> = rows.into_values().collect();
    scores.sort_by(|a, b| (&a.mode, &a.difficulty).cmp(&(&b.mode, &b.difficulty)));
    for row in &mut scores { row.scores.sort_by(|a, b| a.player_id.cmp(&b.player_id)); }
    MapScoresResponse { map, current_diffs, scores }
}
