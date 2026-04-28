import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Playlist, PlaylistMap, PlaylistUIState } from '../types';
import PlaylistCard from '../components/PlaylistCard';
import MapCard from '../components/MapCard';
import Modal from '../components/Modal';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';
import { Plus } from 'lucide-react';

export default function Playlists() {
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [playlistUIStates, setPlaylistUIStates] = useState<Record<number, PlaylistUIState>>({});
  const [playlistTaskIds, setPlaylistTaskIds] = useState<Record<number, string>>({});
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState<number | null>(null);
  const [showMapsModal, setShowMapsModal] = useState<number | null>(null);
  const [playlistMaps, setPlaylistMaps] = useState<PlaylistMap[]>([]);
  const [addId, setAddId] = useState('');
  const [addFormError, setAddFormError] = useState('');
  const [inlineError, setInlineError] = useState<string>('');
  const [updatingId, setUpdatingId] = useState<number | null>(null);
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

  // 恢复进行中的任务状态
  const restoreActiveTasks = useCallback(async () => {
    try {
      const tasks = await invoke<[string, { task_type: string; target_id: string; progress: number; message: string }][]>('get_active_tasks');
      const playlistTasks = tasks.filter(([, s]) => s.task_type === 'add_playlist');
      if (playlistTasks.length === 0) return;

      setPlaylistUIStates(prev => {
        const updated = { ...prev };
        for (const [, status] of playlistTasks) {
          const targetId = parseInt(status.target_id);
          updated[targetId] = {
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
  const playlistTaskIdsRef = useRef<Record<number, string>>({});
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
        // 查找对应此 task_id 的 playlistId
        const playlistId = Object.keys(playlistTaskIdsRef.current).find(
          key => playlistTaskIdsRef.current[parseInt(key)] === progressData.task_id
        );
        if (!playlistId) return;

        const playlistIdNum = parseInt(playlistId);

        const isCompleted = progressData.stage === 'completed';

        // 更新 UI 状态
        setPlaylistUIStates(prev => {
          if (isCompleted) {
            const updated = { ...prev };
            delete updated[playlistIdNum];
            return updated;
          }
          return {
            ...prev,
            [playlistIdNum]: {
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
    if (!addId.trim()) return;

    // 验证歌单 ID 格式（纯数字）
    if (!/^\d+$/.test(addId.trim())) {
      setAddFormError('歌单 ID 必须为纯数字');
      return;
    }

    setAddFormError('');

    // 检查是否已存在
    const playlistId = parseInt(addId.trim());
    const existingIdx = playlists.findIndex(p => p.playlist_id === playlistId);
    const isExisting = existingIdx !== -1;
    const uiState = playlistUIStates[playlistId];
    const isProcessing = isExisting && uiState?.loading;

    if (isProcessing) {
      setAddFormError('该歌单正在处理中，请稍候...');
      return;
    }

    // 前端生成 task_id，确保进度事件能正确匹配
    const taskId = crypto.randomUUID();

    try {
      // 更新 taskId 映射
      setPlaylistTaskIds(prev => ({ ...prev, [playlistId]: taskId }));

      // 设置 UI 状态
      setPlaylistUIStates(prev => ({
        ...prev,
        [playlistId]: {
          loading: true,
          progress: 0,
          statusMessage: isExisting ? '正在更新...' : '准备添加...'
        }
      }));

      // 如果已存在，更新现有条目状态；否则添加临时条目
      if (!isExisting) {
        const tempPlaylist: Playlist = {
          playlist_id: playlistId,
          name: '加载中...',
          description: undefined,
          playlist_image: undefined,
          owner_id: undefined,
          owner_name: undefined,
          total_maps: undefined,
          created_at: undefined,
          songs_changed_at: undefined,
          download_url: undefined,
        };
        setPlaylists(prev => [...prev, tempPlaylist]);
      } else {
        setPlaylists(prev => prev.map((p, i) => i === existingIdx ? { ...p, name: '加载中...' } : p));
      }

      setShowAddModal(false);
      setAddId('');
      setAddFormError('');

      // 执行长时间操作，将前端生成的 task_id 传给后端
      await invoke<{ task_id: string; playlist: Playlist }>('add_playlist', {
        playlistId,
        taskId  // 传递前端生成的 task_id
      });

      // 操作成功，重新加载列表
      await loadPlaylists();
      await restoreActiveTasks();
    } catch (e) {
      // 操作失败，恢复 UI 状态
      setPlaylistUIStates(prev => {
        const updated = { ...prev };
        delete updated[playlistId];
        return updated;
      });
      setPlaylistTaskIds(prev => {
        const updated = { ...prev };
        delete updated[playlistId];
        return updated;
      });

      if (!isExisting) {
        setPlaylists(prev => prev.filter(p => p.playlist_id !== playlistId));
      } else {
        // 恢复原来的名称
        setPlaylists(prev => prev.map((p, i) => i === existingIdx ? { ...p, name: playlists[existingIdx].name } : p));
      }
      setAddFormError(`${isExisting ? '更新' : '添加'}失败: ${e}`);
    }
  }

  async function handleDeletePlaylist(id: number) {
    try {
      await invoke('delete_playlist', { playlistId: id });
      setShowDeleteModal(null);
      loadPlaylists();
    } catch (e) {
      setInlineError(`删除失败: ${e}`);
    }
  }

  async function handleViewMaps(playlistId: number) {
    try {
      // 切换歌单时先清空旧数据
      setPlaylistMaps([]);
      const maps = await invoke<PlaylistMap[]>('get_playlist_maps', { playlistId });
      setPlaylistMaps(maps);
      setShowMapsModal(playlistId);
    } catch (e) {
      setInlineError(`获取谱面列表失败: ${e}`);
    }
  }

  async function handleUpdatePlaylist(playlistId: number) {
    setUpdatingId(playlistId);
    setInlineError('');

    const taskId = crypto.randomUUID();
    setPlaylistTaskIds(prev => ({ ...prev, [playlistId]: taskId }));
    setPlaylistUIStates(prev => ({
      ...prev,
      [playlistId]: {
        loading: true,
        progress: 0,
        statusMessage: '准备更新...'
      }
    }));
    setPlaylists(prev => prev.map(p => p.playlist_id === playlistId ? { ...p, name: '加载中...' } : p));

    try {
      await invoke('add_playlist', { playlistId, taskId });
      await loadPlaylists();
      await restoreActiveTasks();
    } catch (e) {
      setPlaylistUIStates(prev => {
        const updated = { ...prev };
        delete updated[playlistId];
        return updated;
      });
      setPlaylistTaskIds(prev => {
        const updated = { ...prev };
        delete updated[playlistId];
        return updated;
      });
      setInlineError(`更新失败: ${e}`);
    } finally {
      setUpdatingId(null);
    }
  }

  if (loading) return <LoadingSpinner />;


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
        <div className="grid gap-4" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(360px, 1fr))' }}>
          {playlists.map(playlist => (
            <PlaylistCard
              key={playlist.playlist_id}
              playlist={playlist}
              uiState={playlistUIStates[playlist.playlist_id]}
              onDelete={() => setShowDeleteModal(playlist.playlist_id)}
              onViewMaps={() => handleViewMaps(playlist.playlist_id)}
              onUpdate={() => handleUpdatePlaylist(playlist.playlist_id)}
              updating={updatingId === playlist.playlist_id}
            />
          ))}
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
        onClose={() => { setShowAddModal(false); setAddFormError(''); }}
        title="添加歌单"
      >
        <div className="space-y-4">
          {addFormError && (
            <div className="p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
              {addFormError}
            </div>
          )}
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
          <div className="flex gap-3 justify-end pt-4">
            <button
              onClick={() => { setShowAddModal(false); setAddFormError(''); }}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all"
            >
              取消
            </button>
            <button
              onClick={handleAddPlaylist}
              disabled={!addId.trim()}
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
                <div key={pm.map_key} className="p-4 rounded-lg bg-bs-bg-tertiary border border-bs-border text-bs-text-muted text-sm">
                  谱面信息缺失 (key: {pm.map_key})
                </div>
              )
            ))
          )}
        </div>
      </Modal>
    </div>
  );
}
