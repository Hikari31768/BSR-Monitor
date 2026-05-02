import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Playlist, PlaylistUIState, PlaylistSource } from '../types';
import PlaylistCard from '../components/PlaylistCard';
import MapCard from '../components/MapCard';
import Modal from '../components/Modal';
import EmptyState from '../components/EmptyState';
import { Plus, Star } from 'lucide-react';

type AddSource = 'beatsaver' | 'mapper' | 'local';

interface MapCardData {
  key: string;
  map?: { key: string; song_name: string; song_author_name: string; level_author_name: string; rating?: number; cover_url?: string; preview_url?: string };
}

function playlistKey(p: Playlist): string {
  return `${p.source}:${p.identifier}`;
}

export default function Playlists() {
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [playlistUIStates, setPlaylistUIStates] = useState<Record<string, PlaylistUIState>>({});
  const [playlistTaskIds, setPlaylistTaskIds] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState<string | null>(null);
  const [showMapsModal, setShowMapsModal] = useState<string | null>(null);
  const [showRefreshRatingsModal, setShowRefreshRatingsModal] = useState<string | null>(null);
  const [playlistMaps, setPlaylistMaps] = useState<MapCardData[]>([]);
  const [addSource, setAddSource] = useState<AddSource>('beatsaver');
  const [addId, setAddId] = useState('');
  const [addFilePath, setAddFilePath] = useState('');
  const [addFormError, setAddFormError] = useState('');
  const [inlineError, setInlineError] = useState<string>('');
  const [updatingKey, setUpdatingKey] = useState<string | null>(null);
  const [refreshingRatingsKey, setRefreshingRatingsKey] = useState<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  async function loadPlaylists() {
    setLoading(true);
    try {
      const list = await invoke<Playlist[]>('get_playlist_list');
      setPlaylists(list);
    } catch (e) {
      console.error('加载歌单失败:', e);
    } finally {
      setLoading(false);
    }
  }

  // 静默刷新数据，不触发 loading 状态
  async function refreshPlaylists() {
    try {
      const list = await invoke<Playlist[]>('get_playlist_list');
      setPlaylists(list);
    } catch (e) {
      console.error('刷新歌单失败:', e);
    }
  }

  // 恢复进行中的任务状态
  const restoreActiveTasks = useCallback(async () => {
    try {
      const tasks = await invoke<[string, { task_type: string; target_id: string; progress: number; message: string }][]>('get_active_tasks');
      const playlistTasks = tasks.filter(([, s]) => s.task_type === 'add_playlist' || s.task_type === 'refresh_playlist');
      if (playlistTasks.length === 0) return;

      setPlaylistUIStates(prev => {
        const updated = { ...prev };
        for (const [, status] of playlistTasks) {
          const targetKey = status.target_id;
          updated[targetKey] = {
            loading: true,
            progress: status.progress,
            statusMessage: status.message || '处理中...'
          };
        }
        return updated;
      });
    } catch (e) {
      console.error('恢复任务状态失败:', e);
    }
  }, []);

  useEffect(() => {
    loadPlaylists().then(() => restoreActiveTasks());
  }, [restoreActiveTasks]);

  // 使用 ref 存储 taskId 映射，避免频繁重新订阅事件
  const playlistTaskIdsRef = useRef<Record<string, string>>({});
  useEffect(() => {
    playlistTaskIdsRef.current = playlistTaskIds;
  }, [playlistTaskIds]);

  // 进度事件监听器 — 在 useEffect 中管理，确保卸载时清理
  useEffect(() => {
    let cancelled = false;
    listen('progress-update', (event) => {
      if (cancelled) return;
      const progressData = event.payload as {
        task_type: string;
        task_id: string;
        stage: string;
        progress: number;
        message?: string;
      };

      // 只处理 add_playlist 类型的进度
      if (progressData.task_type === 'add_playlist') {
        // 查找对应此 task_id 的 playlist key
        const pKey = Object.keys(playlistTaskIdsRef.current).find(
          key => playlistTaskIdsRef.current[key] === progressData.task_id
        );
        if (!pKey) return;

        const isCompleted = progressData.stage === 'completed';

        // 更新 UI 状态
        setPlaylistUIStates(prev => {
          if (isCompleted) {
            const updated = { ...prev };
            delete updated[pKey];
            return updated;
          }
          return {
            ...prev,
            [pKey]: {
              loading: true,
              progress: progressData.progress,
              statusMessage: progressData.message || `阶段: ${progressData.stage}`
            }
          };
        });
      }
    }).then(unlisten => {
      unlistenRef.current = unlisten;
    });

    return () => {
      cancelled = true;
      unlistenRef.current?.();
      unlistenRef.current = null;
    };
  }, []);

  async function handleAddPlaylist() {
    const source = addSource;
    const identifier = source === 'local' ? addFilePath : addId.trim();

    if (!identifier) return;

    if (source !== 'local' && !/^\d+$/.test(identifier)) {
      setAddFormError('ID 必须为纯数字');
      return;
    }

    setAddFormError('');

    const key = `${source}:${identifier}`;
    const existingIdx = playlists.findIndex(p => playlistKey(p) === key);
    const isExisting = existingIdx !== -1;
    const uiState = playlistUIStates[key];
    const isProcessing = isExisting && uiState?.loading;

    if (isProcessing) {
      setAddFormError('该歌单正在处理中，请稍候...');
      return;
    }

    const taskId = crypto.randomUUID();

    try {
      setPlaylistTaskIds(prev => ({ ...prev, [key]: taskId }));

      setPlaylistUIStates(prev => ({
        ...prev,
        [key]: {
          loading: true,
          progress: 0,
          statusMessage: '正在添加...'
        }
      }));

      if (!isExisting) {
        const tempPlaylist: Playlist = {
          source: source as PlaylistSource,
          identifier,
          name: '加载中...',
        };
        setPlaylists(prev => [...prev, tempPlaylist]);
      } else {
        setPlaylists(prev => prev.map((p, i) => i === existingIdx ? { ...p, name: '加载中...' } : p));
      }

      setShowAddModal(false);
      setAddId('');
      setAddFilePath('');
      setAddFormError('');

      await invoke<{ task_id: string; playlist: Playlist }>('add_playlist', {
        source,
        identifier,
        taskId,
      });

      await refreshPlaylists();
      await restoreActiveTasks();
    } catch (e) {
      setPlaylistUIStates(prev => {
        const updated = { ...prev };
        delete updated[key];
        return updated;
      });
      setPlaylistTaskIds(prev => {
        const updated = { ...prev };
        delete updated[key];
        return updated;
      });

      if (!isExisting) {
        setPlaylists(prev => prev.filter(p => playlistKey(p) !== key));
      } else {
        setPlaylists(prev => prev.map((p, i) => i === existingIdx ? { ...p, name: playlists[existingIdx].name } : p));
      }
      setAddFormError(`添加失败: ${e}`);
    }
  }

  async function handleDeletePlaylist(pKey: string) {
    const [source, ...identifierParts] = pKey.split(':');
    const identifier = identifierParts.join(':');
    try {
      await invoke('delete_playlist', { source, identifier });
      setShowDeleteModal(null);
      loadPlaylists();
    } catch (e) {
      setInlineError(`删除失败: ${e}`);
    }
  }

  async function handleViewMaps(pKey: string) {
    try {
      const [source, ...identifierParts] = pKey.split(':');
      const identifier = identifierParts.join(':');

      const songs = await invoke<any[]>('get_playlist_maps', { source, identifier });

      const maps: MapCardData[] = songs.map((song: any) => ({
        key: song.hash,
        map: song.key ? {
          key: song.key,
          song_name: song.songName || 'Unknown',
          song_author_name: '',
          level_author_name: song.levelAuthorName || 'Unknown',
        } : undefined,
      }));

      setPlaylistMaps(maps);
      setShowMapsModal(pKey);
    } catch (e) {
      setInlineError(`获取谱面列表失败: ${e}`);
    }
  }

  async function handleUpdatePlaylist(pKey: string) {
    setUpdatingKey(pKey);
    setInlineError('');

    const [source, ...identifierParts] = pKey.split(':');
    const identifier = identifierParts.join(':');

    const taskId = crypto.randomUUID();
    setPlaylistTaskIds(prev => ({ ...prev, [pKey]: taskId }));
    setPlaylistUIStates(prev => ({
      ...prev,
      [pKey]: {
        loading: true,
        progress: 0,
        statusMessage: '准备更新...'
      }
    }));
    setPlaylists(prev => prev.map(p => playlistKey(p) === pKey ? { ...p, name: '加载中...' } : p));

    try {
      await invoke('refresh_playlist', { source, identifier, taskId });
      await refreshPlaylists();
      await restoreActiveTasks();
    } catch (e) {
      setPlaylistUIStates(prev => {
        const updated = { ...prev };
        delete updated[pKey];
        return updated;
      });
      setPlaylistTaskIds(prev => {
        const updated = { ...prev };
        delete updated[pKey];
        return updated;
      });
      setInlineError(`更新失败: ${e}`);
    } finally {
      setUpdatingKey(null);
    }
  }

  async function handleRefreshPlaylistRatings(pKey: string) {
    setShowRefreshRatingsModal(null);
    setRefreshingRatingsKey(pKey);
    setInlineError('');

    const [source, ...identifierParts] = pKey.split(':');
    const identifier = identifierParts.join(':');

    try {
      await invoke<string>('refresh_playlist_ratings', { source, identifier });
    } catch (e) {
      setInlineError(`更新星级失败: ${e}`);
    } finally {
      setRefreshingRatingsKey(null);
    }
  }

  return (
    <div className="animate-fade-in">
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold text-bs-text-primary tracking-tight">歌单管理</h1>
        <button
          onClick={() => setShowAddModal(true)}
          className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-accent-primary text-white hover:bg-bs-accent-secondary transition-all shadow-lg hover:shadow-bs-accent-primary/30 disabled:opacity-50 flex items-center gap-2"
          disabled={loading}
        >
          <Plus size={18} />
          添加歌单
        </button>
      </div>

      {playlists.length === 0 ? (
        <EmptyState message="暂无歌单，点击上方按钮添加" />
      ) : (
        <div className="grid gap-4" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(420px, 1fr))' }}>
          {playlists.map(playlist => {
            const pKey = playlistKey(playlist);
            return (
              <PlaylistCard
                key={pKey}
                playlist={playlist}
                uiState={playlistUIStates[pKey]}
                onDelete={() => setShowDeleteModal(pKey)}
                onViewMaps={() => handleViewMaps(pKey)}
                onUpdate={() => handleUpdatePlaylist(pKey)}
                onRefreshRatings={() => setShowRefreshRatingsModal(pKey)}
                updating={updatingKey === pKey}
                refreshingRatings={refreshingRatingsKey === pKey}
              />
            );
          })}
        </div>
      )}

      {/* 内联错误提示 */}
      {inlineError && (
        <div className="mt-4 p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
          {inlineError}
          <button
            onClick={() => setInlineError('')}
            className="ml-2 text-red-400 hover:text-red-300"
          >
            关闭
          </button>
        </div>
      )}

      <Modal
        isOpen={showAddModal}
        onClose={() => { setShowAddModal(false); setAddFormError(''); setAddId(''); setAddFilePath(''); }}
        title="添加歌单"
      >
        <div className="space-y-4">
          {addFormError && (
            <div className="p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
              {addFormError}
            </div>
          )}

          {/* 来源选择 Tab */}
          <div className="flex gap-2">
            {([['beatsaver', 'BeatSaver'], ['mapper', '谱师'], ['local', '本地']] as [AddSource, string][]).map(([s, label]) => (
              <button
                key={s}
                onClick={() => { setAddSource(s); setAddId(''); setAddFilePath(''); setAddFormError(''); }}
                className={`flex-1 px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                  addSource === s
                    ? 'bg-bs-accent-primary text-white'
                    : 'bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary'
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          {/* BeatSaver 输入 */}
          {addSource === 'beatsaver' && (
            <div>
              <label className="block text-sm text-bs-text-secondary mb-2">
                BeatSaver 歌单 ID <span className="text-red-400">*</span>
              </label>
              <input
                type="text"
                inputMode="numeric"
                pattern="\d*"
                value={addId}
                onChange={(e) => {
                  setAddId(e.target.value.replace(/[^\d]/g, ''));
                  setAddFormError('');
                }}
                placeholder="如: 1"
                className="w-full px-4 py-3 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary placeholder-bs-text-muted focus:border-bs-accent-primary focus:ring-2 focus:ring-bs-accent-primary/20 focus:outline-none transition-all"
              />
              <p className="text-xs text-bs-text-muted mt-1">请输入纯数字歌单 ID</p>
            </div>
          )}

          {/* 谱师输入 */}
          {addSource === 'mapper' && (
            <div>
              <label className="block text-sm text-bs-text-secondary mb-2">
                谱师用户 ID <span className="text-red-400">*</span>
              </label>
              <input
                type="text"
                inputMode="numeric"
                pattern="\d*"
                value={addId}
                onChange={(e) => {
                  setAddId(e.target.value.replace(/[^\d]/g, ''));
                  setAddFormError('');
                }}
                placeholder="如: 12345"
                className="w-full px-4 py-3 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary placeholder-bs-text-muted focus:border-bs-accent-primary focus:ring-2 focus:ring-bs-accent-primary/20 focus:outline-none transition-all"
              />
              <p className="text-xs text-bs-text-muted mt-1">请输入 BeatSaver 谱师用户 ID</p>
            </div>
          )}

          {/* 本地文件选择 */}
          {addSource === 'local' && (
            <div>
              <label className="block text-sm text-bs-text-secondary mb-2">
                .bplist 文件 <span className="text-red-400">*</span>
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={addFilePath}
                  onChange={(e) => {
                    setAddFilePath(e.target.value);
                    setAddFormError('');
                  }}
                  placeholder="请输入或选择 .bplist 文件路径..."
                  className="flex-1 px-4 py-3 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary placeholder-bs-text-muted focus:border-bs-accent-primary focus:ring-2 focus:ring-bs-accent-primary/20 focus:outline-none transition-all"
                />
                <button
                  onClick={async () => {
                    const selected = await open({
                      multiple: false,
                      filters: [{ name: 'Playlist', extensions: ['bplist'] }],
                    });
                    if (selected && typeof selected === 'string') {
                      setAddFilePath(selected);
                      setAddFormError('');
                    }
                  }}
                  className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all flex items-center gap-1 whitespace-nowrap"
                >
                  浏览
                </button>
              </div>
              <p className="text-xs text-bs-text-muted mt-1">选择本地 .bplist 文件</p>
            </div>
          )}

          <div className="flex gap-3 justify-end pt-4">
            <button
              onClick={() => { setShowAddModal(false); setAddFormError(''); setAddId(''); setAddFilePath(''); }}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all"
            >
              取消
            </button>
            <button
              onClick={handleAddPlaylist}
              disabled={addSource === 'local' ? !addFilePath : !addId.trim()}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-accent-primary text-white hover:bg-bs-accent-secondary transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              添加
            </button>
          </div>
        </div>
      </Modal>

      <Modal
        isOpen={showDeleteModal !== null}
        onClose={() => { setShowDeleteModal(null); setInlineError(''); }}
        title="确认删除"
      >
        {inlineError && (
          <div className="p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
            {inlineError}
          </div>
        )}
        <p className="text-bs-text-secondary mb-6">确定要删除该歌单吗？删除歌单不会移除已下载的谱面文件。</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={() => { setShowDeleteModal(null); setInlineError(''); }}
            className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all"
          >
            取消
          </button>
          <button
            onClick={() => handleDeletePlaylist(showDeleteModal!)}
            className="px-4 py-2.5 rounded-lg font-medium text-sm bg-red-500 text-white hover:bg-red-600 transition-all"
          >
            删除
          </button>
        </div>
      </Modal>

      <Modal
        isOpen={showMapsModal !== null}
        onClose={() => { setShowMapsModal(null); setPlaylistMaps([]); }}
        title="歌单谱面列表"
      >
        <div className="max-h-[60vh] overflow-auto space-y-3 pr-2">
          {playlistMaps.length === 0 ? (
            <EmptyState message="暂无谱面" />
          ) : (
            playlistMaps.map((pm) => (
              pm.map ? <MapCard key={pm.map.key} map={pm.map} /> : (
                <div key={pm.key} className="p-4 rounded-lg bg-bs-bg-tertiary border border-bs-border text-bs-text-muted text-sm">
                  谱面信息缺失 (hash: {pm.key})
                </div>
              )
            ))
          )}
        </div>
      </Modal>

      <Modal
        isOpen={showRefreshRatingsModal !== null}
        onClose={() => setShowRefreshRatingsModal(null)}
        title="确认更新星级"
      >
        <div className="space-y-4">
          <div className="flex items-start gap-3 p-3 rounded-lg bg-amber-500/10 border border-amber-500/30">
            <Star size={20} className="text-amber-400 flex-shrink-0 mt-0.5" />
            <div>
              <p className="text-amber-300 font-medium text-sm">此操作将全量重新拉取该歌单所有谱面的星级数据</p>
              <p className="text-amber-400/80 text-xs mt-1">
                即使已有星级信息也会被覆盖更新。由于需要逐个调用 API，对于谱面较多的歌单会比较耗时，请耐心等待。
              </p>
            </div>
          </div>
          <div className="flex gap-3 justify-end">
            <button
              onClick={() => setShowRefreshRatingsModal(null)}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all"
            >
              取消
            </button>
            <button
              onClick={() => handleRefreshPlaylistRatings(showRefreshRatingsModal!)}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-accent-primary text-white hover:bg-bs-accent-secondary transition-all"
            >
              确认更新
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
