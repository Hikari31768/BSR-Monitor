import { invoke } from '@tauri-apps/api/core';
import { UserPrefs, defaultUserPrefs } from '../types';

// Rust 返回的数据结构（snake_case）
interface UserPrefsRs {
  score_table: {
    view_mode: string;
    selected_playlist_key: string | null;
    map_sort: string;
    diff_sort: string;
  };
  volume: number;
  starred_player_id: string | null;
}

/**
 * 将 Rust 返回的 snake_case 数据转换为前端 camelCase
 */
function fromRustFormat(rs: UserPrefsRs): UserPrefs {
  return {
    scoreTable: {
      viewMode: rs.score_table.view_mode as 'single' | 'playlist',
      selectedPlaylistKey: rs.score_table.selected_playlist_key,
      mapSort: rs.score_table.map_sort as 'asc' | 'desc',
      diffSort: rs.score_table.diff_sort as 'asc' | 'desc',
    },
    volume: rs.volume,
    starredPlayerId: rs.starred_player_id,
  };
}

/**
 * 将前端 camelCase 数据转换为 Rust snake_case
 */
function toRustFormat(prefs: UserPrefs): UserPrefsRs {
  return {
    score_table: {
      view_mode: prefs.scoreTable.viewMode,
      selected_playlist_key: prefs.scoreTable.selectedPlaylistKey,
      map_sort: prefs.scoreTable.mapSort,
      diff_sort: prefs.scoreTable.diffSort,
    },
    volume: prefs.volume,
    starred_player_id: prefs.starredPlayerId,
  };
}

/**
 * 从应用数据目录加载用户偏好
 * 文件不存在时返回默认值
 */
export async function loadUserPrefs(): Promise<UserPrefs> {
  try {
    const rs = await invoke<UserPrefsRs>('load_user_prefs');
    return fromRustFormat(rs);
  } catch (e) {
    console.error('加载用户偏好失败:', e);
    return { ...defaultUserPrefs };
  }
}

/**
 * 保存用户偏好到应用数据目录
 */
export async function saveUserPrefs(prefs: UserPrefs): Promise<void> {
  try {
    const rs = toRustFormat(prefs);
    await invoke('save_user_prefs', { prefs: rs });
  } catch (e) {
    console.error('保存用户偏好失败:', e);
  }
}

/**
 * 迁移 localStorage 中的旧数据到 JSON 文件
 * 仅在 JSON 文件不存在时执行
 */
export async function migrateFromLocalStorage(): Promise<void> {
  try {
    const prefs = await loadUserPrefs();

    // 检查是否是默认状态（说明没有 JSON 文件或文件是空的）
    let hasMigrated = false;

    // 迁移音量设置
    const savedVolume = localStorage.getItem('bsr-monitor-audio-volume');
    if (savedVolume) {
      const volume = parseFloat(savedVolume);
      if (!isNaN(volume) && volume >= 0 && volume <= 1) {
        prefs.volume = volume;
        hasMigrated = true;
      }
      localStorage.removeItem('bsr-monitor-audio-volume');
    }

    // 迁移标星玩家
    const starredId = localStorage.getItem('starred_player_id');
    if (starredId) {
      prefs.starredPlayerId = starredId;
      hasMigrated = true;
      localStorage.removeItem('starred_player_id');
    }

    if (hasMigrated) {
      await saveUserPrefs(prefs);
      console.log('用户偏好迁移完成');
    }
  } catch (e) {
    console.error('迁移用户偏好失败:', e);
  }
}
