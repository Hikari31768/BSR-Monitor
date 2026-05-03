import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  PlayerInfo,
  Map,
  Playlist,
  MapScoresResponse,
  PlaylistScoresResponse,
} from '../types';
import ScoreTableHeader from '../components/ScoreTableHeader';
import MapScoreGroup from '../components/MapScoreGroup';
import PlayerSelectModal from '../components/PlayerSelectModal';
import PlaylistSelect from '../components/PlaylistSelect';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';
import { Search, ChevronLeft, ChevronRight } from 'lucide-react';
import { loadUserPrefs, saveUserPrefs } from '../utils/prefs';

type ViewMode = 'single' | 'playlist';

interface SelectedPlayer extends PlayerInfo {
  order: number;
}

const SOURCE_OPTIONS: { value: string; label: string }[] = [
  { value: 'all', label: '全部' },
  { value: 'beatsaver', label: 'Saver' },
  { value: 'mapper', label: '谱师' },
  { value: 'local', label: '本地' },
];

/** 歌单类型筛选下拉框（自定义样式，与 PlaylistSelect 风格一致） */
function SourceSelect({ selected, onSelect }: { selected: string; onSelect: (v: string) => void }) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div ref={containerRef} className="relative w-32 shrink-0">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-4 py-3 pr-10 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary text-left hover:border-bs-border-hover transition-all"
        style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E")`,
          backgroundRepeat: 'no-repeat',
          backgroundPosition: 'right 0.75rem center',
          backgroundSize: '1.25rem',
        }}
      >
        {SOURCE_OPTIONS.find(o => o.value === selected)?.label ?? '全部'}
      </button>

      {isOpen && (
        <div
          ref={listRef}
          className="absolute top-full left-0 right-0 mt-1 bg-bs-bg-tertiary border border-bs-border rounded-lg shadow-lg overflow-hidden z-50 max-h-60 overflow-y-auto"
        >
          {SOURCE_OPTIONS.map(option => (
            <div
              key={option.value}
              onClick={() => {
                onSelect(option.value);
                setIsOpen(false);
              }}
              className={`px-4 py-3 cursor-pointer transition-colors ${
                selected === option.value
                  ? 'bg-bs-accent-primary/20 text-bs-accent-primary'
                  : 'text-bs-text-secondary hover:text-bs-text-primary hover:bg-bs-bg-hover'
              }`}
            >
              {option.label}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default function ScoreTable() {
  // 视图模式（null 表示尚未加载）
  const [viewMode, setViewMode] = useState<ViewMode | null>(null);

  // 玩家管理
  const [selectedPlayers, setSelectedPlayers] = useState<SelectedPlayer[]>([]);
  const [isPlayerModalOpen, setIsPlayerModalOpen] = useState(false);

  // 单谱面模式
  const [mapInput, setMapInput] = useState('');
  const [currentMapData, setCurrentMapData] = useState<MapScoresResponse | null>(null);

  // 歌单模式
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [selectedPlaylistKey, setSelectedPlaylistKey] = useState<string | null>(null);
  const [selectedSource, setSelectedSource] = useState<string>('all');
  const [playlistData, setPlaylistData] = useState<PlaylistScoresResponse | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [mapSort, setMapSort] = useState<'asc' | 'desc'>('asc');
  const [diffSort, setDiffSort] = useState<'asc' | 'desc'>('desc');

  // 加载状态
  const [, setLoadingPlayers] = useState(false);
  const [, setLoadingPlaylists] = useState(false);
  const [loadingData, setLoadingData] = useState(false);
  const [dataError, setDataError] = useState<string>('');

  // 偏好是否已加载
  const [prefsLoaded, setPrefsLoaded] = useState(false);

  // 所有可变值都用 ref 存储，确保 callback 和 effect 不依赖 state
  const selectedPlayersRef = useRef(selectedPlayers);
  useEffect(() => { selectedPlayersRef.current = selectedPlayers; }, [selectedPlayers]);

  const mapInputRef = useRef(mapInput);
  useEffect(() => { mapInputRef.current = mapInput; }, [mapInput]);

  const currentMapKeyRef = useRef<string | null>(null);
  useEffect(() => { currentMapKeyRef.current = currentMapData?.map.key ?? null; }, [currentMapData]);

  const currentMapDataRef = useRef(currentMapData);
  useEffect(() => { currentMapDataRef.current = currentMapData; }, [currentMapData]);

  const playlistDataRef = useRef(playlistData);
  useEffect(() => { playlistDataRef.current = playlistData; }, [playlistData]);

  const viewModeRef = useRef(viewMode);
  useEffect(() => { viewModeRef.current = viewMode; }, [viewMode]);

  const selectedPlaylistKeyRef = useRef(selectedPlaylistKey);
  useEffect(() => { selectedPlaylistKeyRef.current = selectedPlaylistKey; }, [selectedPlaylistKey]);

  const selectedSourceRef = useRef(selectedSource);
  useEffect(() => { selectedSourceRef.current = selectedSource; }, [selectedSource]);

  const currentPageRef = useRef(currentPage);
  useEffect(() => { currentPageRef.current = currentPage; }, [currentPage]);

  const mapSortRef = useRef(mapSort);
  useEffect(() => { mapSortRef.current = mapSort; }, [mapSort]);

  const loadingDataRef = useRef(loadingData);
  useEffect(() => { loadingDataRef.current = loadingData; }, [loadingData]);

  const prevPlayerCountRef = useRef(0);
  const savedPlayerIdsRef = useRef<string[]>([]);

  // 初始化加载：先加载偏好，再加载数据
  useEffect(() => {
    loadUserPrefs().then(prefs => {
      setViewMode(prefs.scoreTable.viewMode);
      setSelectedPlaylistKey(prefs.scoreTable.selectedPlaylistKey);
      setMapSort(prefs.scoreTable.mapSort);
      setDiffSort(prefs.scoreTable.diffSort);
      // 保存玩家ID到ref，供loadPlayers使用
      savedPlayerIdsRef.current = prefs.scoreTable.selectedPlayerIds || [];
      setPrefsLoaded(true);
    });
  }, []);

  // 偏好变化时保存
  useEffect(() => {
    if (!prefsLoaded) return;
    loadUserPrefs().then(prefs => {
      prefs.scoreTable.viewMode = viewMode ?? 'single';
      prefs.scoreTable.selectedPlaylistKey = selectedPlaylistKey;
      prefs.scoreTable.mapSort = mapSort;
      prefs.scoreTable.diffSort = diffSort;
      prefs.scoreTable.selectedPlayerIds = selectedPlayers.map(p => p.id);
      saveUserPrefs(prefs);
    });
  }, [viewMode, selectedPlaylistKey, mapSort, diffSort, selectedPlayers, prefsLoaded]);

  // 初始化加载玩家和歌单（等待偏好加载完成后）
  useEffect(() => {
    if (!prefsLoaded) return;
    loadPlayers();
    loadPlaylists();
  }, [prefsLoaded]);

  async function loadPlayers() {
    setLoadingPlayers(true);
    try {
      const list = await invoke<PlayerInfo[]>('get_player_list');
      if (list.length === 0) return;

      const savedIds = savedPlayerIdsRef.current;
      if (savedIds.length > 0) {
        // 恢复保存的玩家列表
        const restoredPlayers: SelectedPlayer[] = [];
        let order = 0;
        for (const id of savedIds) {
          const player = list.find(p => p.id === id);
          if (player) {
            restoredPlayers.push({ ...player, order });
            order++;
          }
        }
        if (restoredPlayers.length > 0) {
          setSelectedPlayers(restoredPlayers);
          prevPlayerCountRef.current = restoredPlayers.length;
        }
      } else {
        // 没有保存的玩家，使用标星玩家或第一个玩家作为默认
        const prefs = await loadUserPrefs();
        const starredId = prefs.starredPlayerId;
        const defaultPlayer = starredId
          ? list.find(p => p.id === starredId)
          : list[0];

        if (defaultPlayer && selectedPlayers.length === 0) {
          setSelectedPlayers([{ ...defaultPlayer, order: 0 }]);
          prevPlayerCountRef.current = 1;
        }
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

  /** 仅查询成绩（不调用 get_map_by_key/hash，不触发星级补全） */
  async function fetchScores(key: string) {
    const playerIds = selectedPlayersRef.current.map(p => p.id);
    if (playerIds.length === 0) return;
    const result = await invoke<MapScoresResponse>('get_map_scores', { key, playerIds });
    setCurrentMapData(result);
  }

  /** 完整查询：先获取谱面信息（含星级补全），再查成绩 */
  const searchMap = useCallback(async () => {
    const input = mapInputRef.current.trim();
    if (!input) return;
    const players = selectedPlayersRef.current;
    if (players.length === 0) return;

    setLoadingData(true);
    setCurrentMapData(null);
    setDataError('');

    try {
      let key = input;
      if (key.length === 40) {
        const map = await invoke<Map>('get_map_by_hash', { hash: key });
        key = map.key;
      } else {
        await invoke<Map>('get_map_by_key', { key });
      }

      await fetchScores(key);
    } catch (e) {
      console.error('查询谱面失败:', e);
      setDataError(`查询失败: ${e}`);
    } finally {
      setLoadingData(false);
    }
  }, []); // 无依赖，完全稳定

  /** 监听 BeatLeader 星级补全完成 → 仅刷新成绩，不重新获取谱面 */
  useEffect(() => {
    const unlisten = listen<string>('map-stars-ready', (event) => {
      const key = event.payload;
      if (currentMapKeyRef.current === key) {
        fetchScores(key).catch(console.error);
      }
    });
    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  /** 查询歌单 — 无依赖，完全稳定 */
  const searchPlaylist = useCallback(async () => {
    if (!selectedPlaylistKeyRef.current) return;
    const players = selectedPlayersRef.current;
    if (players.length === 0) return;

    setLoadingData(true);
    setPlaylistData(null);
    setDataError('');

    try {
      const [source, ...identifierParts] = selectedPlaylistKeyRef.current.split(':');
      const identifier = identifierParts.join(':');
      const playerIds = players.map(p => p.id);
      const result = await invoke<PlaylistScoresResponse>('get_playlist_scores', {
        source,
        identifier,
        page: currentPageRef.current,
        playerIds,
        reverse: mapSortRef.current === 'desc',
      });

      setPlaylistData(result);
    } catch (e) {
      console.error('查询歌单失败:', e);
      setDataError(`查询失败: ${e}`);
    } finally {
      setLoadingData(false);
    }
  }, []); // 无依赖，完全稳定

  // 切换歌单时重置页码和数据（但保留排序偏好）
  useEffect(() => {
    setCurrentPage(1);
    setPlaylistData(null);
  }, [selectedPlaylistKey]);

  // 歌单类型筛选变化时，清空已选歌单
  useEffect(() => {
    setSelectedPlaylistKey(null);
    setPlaylistData(null);
  }, [selectedSource]);

  // 查询条件变化时查询（歌单模式）
  useEffect(() => {
    if (viewMode === 'playlist' && selectedPlaylistKey) {
      if (currentPage === 1) {
        searchPlaylist();
      } else {
        setCurrentPage(1);
      }
    }
  }, [viewMode, selectedPlaylistKey, mapSort, selectedPlayers]);

  // 页码变化时查询
  useEffect(() => {
    if (viewMode === 'playlist' && selectedPlaylistKey) {
      searchPlaylist();
    }
  }, [currentPage]);

  // 玩家数量变化时自动刷新成绩
  useEffect(() => {
    const currentCount = selectedPlayers.length;
    if (currentCount !== prevPlayerCountRef.current) {
      prevPlayerCountRef.current = currentCount;
      if (viewModeRef.current === 'single' && currentMapDataRef.current && !loadingDataRef.current && mapInputRef.current.trim()) {
        const key = currentMapDataRef.current.map.key;
        fetchScores(key).catch(console.error);
      }
    }
  }, [selectedPlayers]);

  // 添加玩家
  function handleAddPlayer(player: PlayerInfo) {
    const maxOrder = selectedPlayers.length > 0
      ? Math.max(...selectedPlayers.map(p => p.order))
      : -1;
    setSelectedPlayers([...selectedPlayers, { ...player, order: maxOrder + 1 }]);
  }

  // 移除玩家
  function handleRemovePlayer(playerId: string) {
    const newPlayers = selectedPlayers.filter(p => p.id !== playerId);
    setSelectedPlayers(newPlayers);
    prevPlayerCountRef.current = newPlayers.length;
  }

  // 刷新玩家成绩
  async function handleRefreshPlayer(playerId: string) {
    try {
      await invoke<string>('refresh_player_scores', { playerId });
      if (viewModeRef.current === 'single' && currentMapDataRef.current) {
        fetchScores(currentMapDataRef.current.map.key).catch(console.error);
      } else if (viewModeRef.current === 'playlist' && playlistDataRef.current) {
        searchPlaylist();
      }
    } catch (e) {
      console.error('刷新成绩失败:', e);
    }
  }

  // 拖拽排序
  function handleReorderPlayers(draggedId: string, targetId: string) {
    const draggedIndex = selectedPlayers.findIndex(p => p.id === draggedId);
    const targetIndex = selectedPlayers.findIndex(p => p.id === targetId);
    if (draggedIndex === -1 || targetIndex === -1) return;

    const newPlayers = [...selectedPlayers];
    const [removed] = newPlayers.splice(draggedIndex, 1);
    newPlayers.splice(targetIndex, 0, removed);
    setSelectedPlayers(newPlayers.map((p, i) => ({ ...p, order: i })));
  }

  // 获取当前显示的数据
  const displayData = useMemo(() => {
    if (viewMode === null) return [];
    return viewMode === 'single'
      ? (currentMapData ? [currentMapData] : [])
      : (playlistData?.maps || []);
  }, [viewMode, currentMapData, playlistData]);

  // 构建表头数据
  const sortedPlayers = useMemo(() => {
    return [...selectedPlayers].sort((a, b) => a.order - b.order);
  }, [selectedPlayers]);

  return (
    <div className="animate-fade-in h-full flex flex-col">
      {/* 标题和模式切换同一行 */}
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold text-bs-text-primary tracking-tight">打谱查询</h1>
        <div className="flex gap-2">
          {viewMode === null ? (
            // 加载中状态：显示骨架屏或禁用态按钮
            <>
              <div className="px-4 py-2 rounded-lg bg-bs-bg-tertiary text-bs-text-muted animate-pulse">
                单谱面
              </div>
              <div className="px-4 py-2 rounded-lg bg-bs-bg-tertiary text-bs-text-muted animate-pulse">
                歌单
              </div>
            </>
          ) : (
            <>
              <button
                onClick={() => setViewMode('single')}
                className={`px-4 py-2 rounded-lg font-medium transition-all ${
                  viewMode === 'single'
                    ? 'bg-bs-accent-primary text-white shadow-lg hover:bg-bs-accent-secondary hover:shadow-bs-accent-primary/30'
                    : 'bg-bs-bg-tertiary text-bs-text-secondary shadow-sm hover:bg-bs-bg-hover hover:text-bs-text-primary'
                }`}
              >
                单谱面
              </button>
              <button
                onClick={() => setViewMode('playlist')}
                className={`px-4 py-2 rounded-lg font-medium transition-all ${
                  viewMode === 'playlist'
                    ? 'bg-bs-accent-primary text-white shadow-lg hover:bg-bs-accent-secondary hover:shadow-bs-accent-primary/30'
                    : 'bg-bs-bg-tertiary text-bs-text-secondary shadow-sm hover:bg-bs-bg-hover hover:text-bs-text-primary'
                }`}
              >
                歌单
              </button>
            </>
          )}
        </div>
      </div>

      {/* 单谱面输入 - 精简版 */}
      {viewMode === 'single' && (
        <div>
          <div className="flex gap-4 mb-4">
            <input
              type="text"
              value={mapInput}
              onChange={(e) => setMapInput(e.target.value)}
              placeholder="输入谱面 Key 或 Hash..."
              className="flex-1 px-4 py-3 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary placeholder-bs-text-muted focus:border-bs-accent-primary focus:ring-2 focus:ring-bs-accent-primary/20 focus:outline-none transition-all"
              onKeyDown={(e) => e.key === 'Enter' && searchMap()}
            />
            <button
              onClick={searchMap}
              disabled={loadingData || !mapInput.trim()}
              className="px-6 py-3 rounded-lg font-medium text-sm bg-bs-accent-primary text-white hover:bg-bs-accent-secondary transition-all shadow-lg hover:shadow-bs-accent-primary/30 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              {loadingData ? <span className="animate-spin">⟳</span> : <Search size={18} />}
              查询
            </button>
          </div>
        </div>
      )}

      {/* 歌单选择 - 精简版 */}
      {viewMode === 'playlist' && (
        <div className="mb-4 space-y-3">
          <div className="flex gap-2">
            <SourceSelect selected={selectedSource} onSelect={setSelectedSource} />
            <PlaylistSelect
              playlists={playlists.filter(p => selectedSource === 'all' || p.source === selectedSource)}
              selectedKey={selectedPlaylistKey}
              onSelect={setSelectedPlaylistKey}
            />
          </div>

          {/* 分页控制 */}
          {playlistData && playlistData.total_pages > 1 && (
            <div className="flex items-center justify-between">
              <button
                onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                disabled={currentPage <= 1 || loadingData}
                className="px-4 py-2 rounded-lg bg-bs-bg-tertiary text-bs-text-primary hover:bg-bs-border disabled:opacity-50 transition-all flex items-center gap-1"
              >
                <ChevronLeft size={16} />
                上一页
              </button>
              <span className="text-bs-text-secondary">
                第 {currentPage} / {playlistData.total_pages} 页
              </span>
              <button
                onClick={() => setCurrentPage(p => Math.min(playlistData.total_pages, p + 1))}
                disabled={currentPage >= playlistData.total_pages || loadingData}
                className="px-4 py-2 rounded-lg bg-bs-bg-tertiary text-bs-text-primary hover:bg-bs-border disabled:opacity-50 transition-all flex items-center gap-1"
              >
                下一页
                <ChevronRight size={16} />
              </button>
            </div>
          )}
        </div>
      )}

      {/* 错误提示 */}
      {dataError && (
        <div className="mb-4 p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
          {dataError}
          <button
            onClick={() => setDataError('')}
            className="ml-2 text-red-400 hover:text-red-300"
          >
            关闭
          </button>
        </div>
      )}

      {/* 成绩表格 - 占据剩余高度，独立滚动 */}
      <div className="bg-bs-bg-secondary rounded-xl border border-bs-border shadow-lg overflow-hidden flex-1 flex flex-col min-h-0">
        <div className="score-table-scroll overflow-auto flex-1 flex flex-col min-h-0">
          {/* 表头 - 始终显示，提供添加玩家的入口 */}
          <ScoreTableHeader
            players={sortedPlayers}
            onAddPlayer={() => setIsPlayerModalOpen(true)}
            onRemovePlayer={handleRemovePlayer}
            onRefreshPlayer={handleRefreshPlayer}
            onReorderPlayers={handleReorderPlayers}
            mapSort={mapSort}
            onToggleMapSort={() => setMapSort(s => s === 'asc' ? 'desc' : 'asc')}
            showMapSort={viewMode === 'playlist'}
            diffSort={diffSort}
            onToggleDiffSort={() => setDiffSort(s => s === 'asc' ? 'desc' : 'asc')}
          />

          {/* 表格内容 */}
          {loadingData ? (
            <div className="p-8 flex-1">
              <LoadingSpinner />
            </div>
          ) : displayData.length === 0 ? (
            <div className="p-8 flex-1">
              <EmptyState
                message={
                  selectedPlayers.length === 0
                    ? '请先选择玩家'
                    : viewMode === 'single'
                      ? (mapInput ? '该谱面无玩家成绩数据' : '请输入谱面 Key 或 Hash 开始查询')
                      : (selectedPlaylistKey ? '该歌单暂无玩家成绩数据' : '请选择歌单开始查询')
                }
              />
            </div>
          ) : (
            <div className="divide-y divide-bs-border flex-1">
              {displayData.map((mapData, mapIndex) => (
                <MapScoreGroup
                  key={mapData.map.key}
                  map={mapData.map}
                  scores={mapData.scores}
                  players={sortedPlayers}
                  showMapDivider={viewMode === 'playlist' && mapIndex < displayData.length - 1}
                  diffSort={diffSort}
                  onPreviewUrlReady={(url) => {
                    if (viewModeRef.current === 'single' && currentMapDataRef.current?.map.key === mapData.map.key) {
                      setCurrentMapData(prev => prev ? { ...prev, map: { ...prev.map, preview_url: url } } : null);
                    } else if (viewModeRef.current === 'playlist' && playlistDataRef.current) {
                      setPlaylistData(prev => prev ? {
                        ...prev,
                        maps: prev.maps.map(m =>
                          m.map.key === mapData.map.key
                            ? { ...m, map: { ...m.map, preview_url: url } }
                            : m
                        ),
                      } : null);
                    }
                  }}
                />
              ))}
            </div>
          )}
        </div>
      </div>

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
