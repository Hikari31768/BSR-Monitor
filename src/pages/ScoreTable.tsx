import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  PlayerInfo,
  Map,
  Playlist,
  MapScoresResponse,
  PlaylistScoresResponse,
} from '../types';
import ScoreTableHeader from '../components/ScoreTableHeader';
import ScoreTableRow from '../components/ScoreTableRow';
import NoScoreMapRow from '../components/NoScoreMapRow';
import PlayerSelectModal from '../components/PlayerSelectModal';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';

type ViewMode = 'single' | 'playlist';

interface SelectedPlayer extends PlayerInfo {
  order: number;
}

export default function ScoreTable() {
  // 视图模式
  const [viewMode, setViewMode] = useState<ViewMode>('single');

  // 玩家管理
  const [selectedPlayers, setSelectedPlayers] = useState<SelectedPlayer[]>([]);
  const [isPlayerModalOpen, setIsPlayerModalOpen] = useState(false);

  // 单谱面模式
  const [mapInput, setMapInput] = useState('');
  const [currentMapData, setCurrentMapData] = useState<MapScoresResponse | null>(null);

  // 歌单模式
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [selectedPlaylistId, setSelectedPlaylistId] = useState<number | null>(null);
  const [playlistData, setPlaylistData] = useState<PlaylistScoresResponse | null>(null);
  const [currentPage, setCurrentPage] = useState(1);

  // 加载状态
  const [loadingPlayers, setLoadingPlayers] = useState(true);
  const [, setLoadingPlaylists] = useState(false);
  const [loadingData, setLoadingData] = useState(false);

  // 初始化加载
  useEffect(() => {
    loadPlayers();
    loadPlaylists();
  }, []);

  async function loadPlayers() {
    setLoadingPlayers(true);
    try {
      const list = await invoke<PlayerInfo[]>('get_player_list');
      // 默认选择第一个玩家
      if (list.length > 0 && selectedPlayers.length === 0) {
        setSelectedPlayers([{ ...list[0], order: 0 }]);
      }
    } catch (e) {
      console.error('加载玩家失败:', e);
    } finally {
      setLoadingPlayers(false);
    }
  }

  async function loadPlaylists() {
    setLoadingPlaylists(true);
    try {
      const list = await invoke<Playlist[]>('get_playlist_list');
      setPlaylists(list);
    } catch (e) {
      console.error('加载歌单失败:', e);
    } finally {
      setLoadingPlaylists(false);
    }
  }

  // 查询单谱面
  const searchMap = useCallback(async () => {
    if (!mapInput.trim() || selectedPlayers.length === 0) return;

    setLoadingData(true);
    setCurrentMapData(null);

    try {
      // 先获取/更新谱面信息（支持 key 或 hash）
      let key = mapInput.trim();
      if (key.length === 40) {
        // 是 hash，先获取谱面
        const map = await invoke<Map>('get_map_by_hash', { hash: key });
        key = map.key;
      } else {
        // 是 key，获取谱面信息（会自动存入数据库）
        await invoke<Map>('get_map_by_key', { key });
      }

      // 查询成绩
      const playerIds = selectedPlayers.map(p => p.id);
      const result = await invoke<MapScoresResponse>('get_map_scores', {
        key,
        playerIds,
      });

      setCurrentMapData(result);
    } catch (e) {
      console.error('查询谱面失败:', e);
      alert(`查询失败: ${e}`);
    } finally {
      setLoadingData(false);
    }
  }, [mapInput, selectedPlayers]);

  // 查询歌单
  const searchPlaylist = useCallback(async () => {
    if (!selectedPlaylistId || selectedPlayers.length === 0) return;

    setLoadingData(true);
    setPlaylistData(null);

    try {
      const playerIds = selectedPlayers.map(p => p.id);
      const result = await invoke<PlaylistScoresResponse>('get_playlist_scores', {
        playlistId: selectedPlaylistId,
        page: currentPage,
        playerIds,
      });

      setPlaylistData(result);
    } catch (e) {
      console.error('查询歌单失败:', e);
      alert(`查询失败: ${e}`);
    } finally {
      setLoadingData(false);
    }
  }, [selectedPlaylistId, currentPage, selectedPlayers]);

  // 切换歌单时重置页码
  useEffect(() => {
    setCurrentPage(1);
    setPlaylistData(null);
  }, [selectedPlaylistId]);

  // 页码变化时自动查询
  useEffect(() => {
    if (viewMode === 'playlist' && selectedPlaylistId) {
      searchPlaylist();
    }
  }, [currentPage, viewMode, selectedPlaylistId, searchPlaylist]);

  // 添加玩家
  function handleAddPlayer(player: PlayerInfo) {
    const maxOrder = selectedPlayers.length > 0
      ? Math.max(...selectedPlayers.map(p => p.order))
      : -1;
    setSelectedPlayers([...selectedPlayers, { ...player, order: maxOrder + 1 }]);
  }

  // 移除玩家
  function handleRemovePlayer(playerId: string) {
    setSelectedPlayers(selectedPlayers.filter(p => p.id !== playerId));
  }

  // 拖拽排序
  function handleReorderPlayers(draggedId: string, targetId: string) {
    const draggedIndex = selectedPlayers.findIndex(p => p.id === draggedId);
    const targetIndex = selectedPlayers.findIndex(p => p.id === targetId);

    if (draggedIndex === -1 || targetIndex === -1) return;

    const newPlayers = [...selectedPlayers];
    const [removed] = newPlayers.splice(draggedIndex, 1);
    newPlayers.splice(targetIndex, 0, removed);

    // 重新分配 order
    setSelectedPlayers(newPlayers.map((p, i) => ({ ...p, order: i })));
  }

  // 获取当前显示的数据
  const displayData = viewMode === 'single'
    ? (currentMapData ? [currentMapData] : [])
    : (playlistData?.maps || []);

  // 构建表头数据
  const sortedPlayers = [...selectedPlayers].sort((a, b) => a.order - b.order);

  if (loadingPlayers) {
    return <LoadingSpinner />;
  }

  return (
    <div className="animate-fade-in">
      {/* 标题 */}
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-[#f0f0f5] tracking-tight">🎯 打谱查询</h1>
      </div>

      {/* 模式切换 */}
      <div className="bg-[#15151c] rounded-xl p-4 border border-[#2a2a3a] shadow-lg mb-6">
        <div className="flex gap-2">
          <button
            onClick={() => setViewMode('single')}
            className={`px-4 py-2 rounded-lg font-medium transition-all ${
              viewMode === 'single'
                ? 'bg-[#ff6b6b] text-white'
                : 'bg-[#1e1e28] text-[#a0a0b0] hover:bg-[#2a2a3a]'
            }`}
          >
            单谱面
          </button>
          <button
            onClick={() => setViewMode('playlist')}
            className={`px-4 py-2 rounded-lg font-medium transition-all ${
              viewMode === 'playlist'
                ? 'bg-[#ff6b6b] text-white'
                : 'bg-[#1e1e28] text-[#a0a0b0] hover:bg-[#2a2a3a]'
            }`}
          >
            歌单
          </button>
        </div>
      </div>

      {/* 单谱面输入 */}
      {viewMode === 'single' && (
        <div className="bg-[#15151c] rounded-xl p-6 border border-[#2a2a3a] shadow-lg mb-6">
          <label className="block text-sm text-[#a0a0b0] mb-2">谱面查询</label>
          <div className="flex gap-4">
            <input
              type="text"
              value={mapInput}
              onChange={(e) => setMapInput(e.target.value)}
              placeholder="输入谱面 Key 或 Hash..."
              className="flex-1 px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] placeholder-[#6b7280] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
              onKeyDown={(e) => e.key === 'Enter' && searchMap()}
            />
            <button
              onClick={searchMap}
              disabled={loadingData || !mapInput.trim()}
              className="px-6 py-3 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all shadow-lg hover:shadow-[#ff6b6b]/30 disabled:opacity-50"
            >
              {loadingData ? '⏳' : '🔍 查询'}
            </button>
          </div>
          <p className="mt-2 text-xs text-[#6b7280]">
            支持输入 Key（如 1a2b3）或 Hash（40位十六进制）
          </p>
        </div>
      )}

      {/* 歌单选择 */}
      {viewMode === 'playlist' && (
        <div className="bg-[#15151c] rounded-xl p-6 border border-[#2a2a3a] shadow-lg mb-6">
          <label className="block text-sm text-[#a0a0b0] mb-2">选择歌单</label>
          <select
            value={selectedPlaylistId || ''}
            onChange={(e) => setSelectedPlaylistId(e.target.value ? parseInt(e.target.value) : null)}
            className="w-full px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
          >
            <option value="">请选择歌单</option>
            {playlists.map(playlist => (
              <option key={playlist.playlist_id} value={playlist.playlist_id}>
                {playlist.name} ({playlist.total_maps || 0} 首)
              </option>
            ))}
          </select>

          {/* 分页控制 */}
          {playlistData && playlistData.total_pages > 1 && (
            <div className="flex items-center justify-between mt-4 pt-4 border-t border-[#2a2a3a]">
              <button
                onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                disabled={currentPage <= 1 || loadingData}
                className="px-4 py-2 rounded-lg bg-[#1e1e28] text-[#f0f0f5] hover:bg-[#2a2a3a] disabled:opacity-50 transition-all"
              >
                ← 上一页
              </button>
              <span className="text-[#a0a0b0]">
                第 {currentPage} / {playlistData.total_pages} 页
              </span>
              <button
                onClick={() => setCurrentPage(p => Math.min(playlistData.total_pages, p + 1))}
                disabled={currentPage >= playlistData.total_pages || loadingData}
                className="px-4 py-2 rounded-lg bg-[#1e1e28] text-[#f0f0f5] hover:bg-[#2a2a3a] disabled:opacity-50 transition-all"
              >
                下一页 →
              </button>
            </div>
          )}
        </div>
      )}

      {/* 成绩表格 */}
      {(displayData.length > 0 || loadingData) && (
        <div className="bg-[#15151c] rounded-xl border border-[#2a2a3a] shadow-lg overflow-hidden">
          {/* 表头 */}
          <ScoreTableHeader
            players={sortedPlayers}
            onAddPlayer={() => setIsPlayerModalOpen(true)}
            onRemovePlayer={handleRemovePlayer}
            onReorderPlayers={handleReorderPlayers}
          />

          {/* 表格内容 */}
          {loadingData ? (
            <div className="p-8">
              <LoadingSpinner />
            </div>
          ) : displayData.length === 0 ? (
            <div className="p-8">
              <EmptyState message="暂无数据" icon="📊" />
            </div>
          ) : (
            <div className="divide-y divide-[#2a2a3a]">
              {displayData.map((mapData, mapIndex) => (
                <div key={mapData.map.key}>
                  {mapData.scores.length > 0 ? (
                    mapData.scores.map((scoreRow, rowIndex) => (
                      <ScoreTableRow
                        key={`${mapData.map.key}-${scoreRow.mode}-${scoreRow.difficulty}`}
                        map={mapData.map}
                        scoreRow={scoreRow}
                        players={sortedPlayers}
                        isFirstRow={rowIndex === 0}
                        totalRows={mapData.scores.length}
                        noScoreDiffs={mapData.no_score_diffs}
                        showMapDivider={viewMode === 'playlist' && mapIndex < displayData.length - 1}
                      />
                    ))
                  ) : (
                    <NoScoreMapRow
                      map={mapData.map}
                      noScoreDiffs={mapData.no_score_diffs}
                      players={sortedPlayers}
                      showMapDivider={viewMode === 'playlist' && mapIndex < displayData.length - 1}
                    />
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* 空状态提示 */}
      {!loadingData && displayData.length === 0 && !loadingData && (
        <div className="mt-8">
          {viewMode === 'single' ? (
            <EmptyState
              message={mapInput ? '该谱面无玩家成绩数据' : '请输入谱面 Key 或 Hash 开始查询'}
              icon="🎵"
            />
          ) : (
            <EmptyState
              message={selectedPlaylistId ? '该歌单暂无玩家成绩数据' : '请选择歌单开始查询'}
              icon="📋"
            />
          )}
        </div>
      )}

      {/* 玩家选择弹窗 */}
      <PlayerSelectModal
        isOpen={isPlayerModalOpen}
        onClose={() => setIsPlayerModalOpen(false)}
        onSelect={handleAddPlayer}
        excludeIds={selectedPlayers.map(p => p.id)}
      />
    </div>
  );
}
