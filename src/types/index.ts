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
  /** 更新操作时的临时提示（覆盖在成绩数量旁），2秒后自动清除 */
  updatingMessage?: string;
}

export interface PlaylistUIState {
  loading: boolean;
  progress: number;
  statusMessage: string;
}

export type PlaylistSource = 'beatsaver' | 'mapper' | 'local';

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
  source: PlaylistSource;
  identifier: string;
  name: string;
  description?: string;
  playlist_image?: string;
  owner_id?: string;
  owner_name?: string;
  total_maps?: number;
  created_at?: string;
  songs_changed_at?: string;
  download_url?: string;
  local_path?: string;
  last_bplist_fetched_at?: number;
  updated_at?: number;
}

export interface PlayerWithScoreCount extends PlayerInfo {
  score_count: number;
}

// ============ 后台任务 ============

export interface BackgroundTask {
  task_id: string;
  task_type: string;
  target_id: string;
  progress: number;
  message: string;
  display_name: string;
}

// ============ 打谱查询相关类型 ============

export interface ScoreRowData {
  mode: string;
  difficulty: string;
  is_current_version: boolean;
  stars?: number;
  scores: PlayerScoreCell[];
}

export interface PlayerScoreCell {
  player_id: string;
  accuracy: number;
  is_expired: boolean;
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

// ============ 用户偏好 ============

export interface UserPrefs {
  scoreTable: {
    viewMode: 'single' | 'playlist';
    selectedPlaylistKey: string | null;
    mapSort: 'asc' | 'desc';
    diffSort: 'asc' | 'desc';
  };
  volume: number;
  starredPlayerId: string | null;
}

export const defaultUserPrefs: UserPrefs = {
  scoreTable: {
    viewMode: 'single',
    selectedPlaylistKey: null,
    mapSort: 'asc',
    diffSort: 'desc',
  },
  volume: 1,
  starredPlayerId: null,
};
