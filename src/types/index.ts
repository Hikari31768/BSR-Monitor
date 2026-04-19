export interface PlayerInfo {
  id: string;
  name: string;
  avatar_url?: string;
  live_url?: string;
  alias?: string;
  loading?: boolean;
  progress?: number;
  status_message?: string;
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
  loading?: boolean;
  progress?: number;
  status_message?: string;
}

export interface PlaylistMap {
  playlist_id: number;
  map_key: string;
  map_order: number;
  map?: Map;
}

export interface LeaderboardResponse {
  leaderboard: {
    id: string;
    song: {
      name: string;
      author: string;
      coverImage: string;
    };
    difficulty: {
      name: string;
      value: number;
    };
    mode: number;
    modeName: string;
  };
  scores: Array<{
    id: number;
    accuracy: number;
    epochTime: number;
    player: {
      id: string;
      name: string;
      avatar: string;
    };
  }>;
}

export interface PlayerWithScoreCount extends PlayerInfo {
  score_count: number;
}

// ============ 打谱查询相关类型 ============

export interface ScoreRow {
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
  scores: ScoreRow[];
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

// ============ 样式常量 ============

export const difficultyColors: Record<string, string> = {
  'Easy': 'text-green-400 bg-green-500/20',
  'Normal': 'text-blue-400 bg-blue-500/20',
  'Hard': 'text-yellow-400 bg-yellow-500/20',
  'Expert': 'text-red-400 bg-red-500/20',
  'ExpertPlus': 'text-purple-400 bg-purple-500/20',
};

export const modeNames: Record<string, string> = {
  'Standard': '标准',
  'OneSaber': '单剑',
  'NoArrows': '无箭头',
  '90Degree': '90度',
  '360Degree': '360度',
  'Lightshow': '光秀',
  'Lawless': '无限制',
  'Legacy': '旧版',
};