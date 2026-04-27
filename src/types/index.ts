export interface PlayerInfo {
  id: string;
  name: string;
  avatar_url?: string;
  live_url?: string;
  alias?: string;
}

export interface PlayerUIState {
  loading: boolean;
  progress: number;
  statusMessage: string;
}

export interface PlaylistUIState {
  loading: boolean;
  progress: number;
  statusMessage: string;
}

export interface PlayerScore {
  player_id: string;
  score_id: number;
  leaderboard_id: string;
  map_key?: string;
  hash: string;
  mode: string;
  difficulty: string;
  accuracy: number;
  epoch_time: number;
}

export interface Map {
  key: string;
  song_name: string;
  song_author_name: string;
  level_author_name: string;
  rating?: number;
  cover_url?: string;
  preview_url?: string;
}

export interface MapDiff {
  key: string;
  hash: string;
  is_current: boolean;
  mode: string;
  difficulty: string;
}

export interface Playlist {
  playlist_id: number;
  name: string;
  description?: string;
  playlist_image?: string;
  owner_id?: number;
  owner_name?: string;
  total_maps?: number;
  created_at?: string;
  songs_changed_at?: string;
  download_url?: string;
}

export interface PlaylistMap {
  playlist_id: number;
  map_key: string;
  map_order: number;
  map?: Map;
}

export interface PlayerWithScoreCount extends PlayerInfo {
  score_count: number;
}

// ============ 打谱查询相关类型 ============

export interface ScoreRowData {
  mode: string;
  difficulty: string;
  is_current_version: boolean;
  scores: PlayerScoreCell[];
}

export interface PlayerScoreCell {
  player_id: string;
  accuracy: number;
}

export interface MapScoresResponse {
  map: Map;
  current_diffs: CurrentDiffInfo[];
  scores: ScoreRowData[];
}

export interface CurrentDiffInfo {
  mode: string;
  difficulty: string;
  hash: string;
}

export interface PlaylistScoresResponse {
  playlist: Playlist;
  page: number;
  total_pages: number;
  maps: MapScoresResponse[];
}

// ============ 样式常量（从 constants 重新导出） ============

import { difficultyColors, modeNames } from '../constants';
export { difficultyColors, modeNames };