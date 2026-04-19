import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Playlist, PlaylistMap } from '../types';
import PlaylistCard from '../components/PlaylistCard';
import MapCard from '../components/MapCard';
import Modal from '../components/Modal';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';

export default function Playlists() {
  const [playlists, setPlaylists] = useState<(Playlist & { loading?: boolean; progress?: number; status_message?: string })[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState<number | null>(null);
  const [showMapsModal, setShowMapsModal] = useState<number | null>(null);
  const [playlistMaps, setPlaylistMaps] = useState<PlaylistMap[]>([]);
  const [addId, setAddId] = useState('');

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

  useEffect(() => {
    loadPlaylists();
  }, []);

  async function handleAddPlaylist() {
    if (!addId) return;

    // 检查是否已存在
    const playlistId = parseInt(addId);
    const existingIdx = playlists.findIndex(p => p.playlist_id === playlistId);
    const isExisting = existingIdx !== -1;
    const isProcessing = isExisting
      ? playlists[existingIdx].loading
      : false;

    if (isProcessing) {
      alert('该歌单正在处理中，请稍候...');
      return;
    }

    try {
      // 如果已存在，更新现有条目状态；否则添加临时条目
      const tempPlaylist: Playlist & { loading?: boolean; progress?: number; status_message?: string } = {
        ...(isExisting ? playlists[existingIdx] : {
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
        }),
        loading: true,
        progress: 0,
        status_message: isExisting ? '正在更新...' : '准备添加...'
      };

      if (isExisting) {
        setPlaylists(prev => prev.map((p, i) => i === existingIdx ? tempPlaylist : p));
      } else {
        setPlaylists(prev => [...prev, tempPlaylist]);
      }
      setShowAddModal(false);
      setAddId('');

      // 监听进度事件
      const unlisten = await listen('progress-update', (event) => {
        const progressData = event.payload as {
          task_type: string;
          task_id: string;
          stage: string;
          progress: number;
          total?: number;
          current?: number;
          message?: string;
        };

        // 只处理当前任务的进度
        if (progressData.task_type === 'add_playlist') {
          setPlaylists(prev => prev.map(p =>
            p.playlist_id === playlistId
              ? {
                  ...p,
                  progress: progressData.progress,
                  status_message: progressData.message || `阶段: ${progressData.stage}`
                }
              : p
          ));
        }
      });

      // 执行长时间操作
      try {
        await invoke('add_playlist', { playlistId });

        // 操作成功，重新加载列表
        loadPlaylists();
      } catch (e) {
        // 操作失败，恢复原有状态或移除临时条目
        if (isExisting) {
          setPlaylists(prev => prev.map((p, i) => i === existingIdx ? { ...p, loading: false, progress: undefined, status_message: undefined } : p));
        } else {
          setPlaylists(prev => prev.filter(p => p.playlist_id !== playlistId));
        }
        alert(`${isExisting ? '更新' : '添加'}失败: ${e}`);
      } finally {
        // 清理事件监听器
        unlisten();
      }
    } catch (e) {
      alert(`操作失败: ${e}`);
    }
  }

  async function handleDeletePlaylist(id: number) {
    // 检查是否正在添加中，不允许删除
    const playlist = playlists.find(p => p.playlist_id === id);
    if (playlist && playlist.loading) {
      alert('无法删除正在添加的歌单');
      return;
    }

    try {
      await invoke('delete_playlist', { playlistId: id });
      setShowDeleteModal(null);
      loadPlaylists();
    } catch (e) {
      alert(`删除失败: ${e}`);
    }
  }

  async function handleViewMaps(playlistId: number) {
    try {
      const maps = await invoke<PlaylistMap[]>('get_playlist_maps', { playlistId });
      setPlaylistMaps(maps);
      setShowMapsModal(playlistId);
    } catch (e) {
      alert(`获取谱面列表失败: ${e}`);
    }
  }

  if (loading) return <LoadingSpinner />;

  return (
    <div className="animate-fade-in">
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold text-[#f0f0f5] tracking-tight">📋 歌单管理</h1>
        <button
          onClick={() => setShowAddModal(true)}
          className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all shadow-lg hover:shadow-[#ff6b6b]/30 disabled:opacity-50"
          disabled={loading}
        >
          ➕ 添加歌单
        </button>
      </div>

      {playlists.length === 0 ? (
        <EmptyState message="暂无歌单，点击上方按钮添加" icon="📋" />
      ) : (
        <div className="grid gap-4" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(360px, 1fr))' }}>
          {playlists.map(playlist => (
            <PlaylistCard
              key={playlist.playlist_id}
              playlist={playlist}
              onDelete={() => setShowDeleteModal(playlist.playlist_id)}
              onViewMaps={() => handleViewMaps(playlist.playlist_id)}
            />
          ))}
        </div>
      )}

      <Modal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        title="添加歌单"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-[#a0a0b0] mb-2">BeatSaver 歌单 ID</label>
            <input
              type="text"
              value={addId}
              onChange={(e) => setAddId(e.target.value)}
              placeholder="如: 1"
              className="w-full px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] placeholder-[#6b7280] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
            />
          </div>
          <div className="flex gap-3 justify-end pt-4">
            <button
              onClick={() => setShowAddModal(false)}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#1e1e28] text-[#a0a0b0] hover:bg-[#252533] hover:text-[#f0f0f5] transition-all"
            >
              取消
            </button>
            <button
              onClick={handleAddPlaylist}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all"
            >
              添加
            </button>
          </div>
        </div>
      </Modal>

      <Modal
        isOpen={showDeleteModal !== null}
        onClose={() => setShowDeleteModal(null)}
        title="确认删除"
      >
        <p className="text-[#a0a0b0] mb-6">确定要删除该歌单吗？</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={() => setShowDeleteModal(null)}
            className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#1e1e28] text-[#a0a0b0] hover:bg-[#252533] hover:text-[#f0f0f5] transition-all"
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
        onClose={() => setShowMapsModal(null)}
        title="歌单谱面列表"
      >
        <div className="max-h-[60vh] overflow-auto space-y-3 pr-2">
          {playlistMaps.length === 0 ? (
            <EmptyState message="暂无谱面" icon="🎵" />
          ) : (
            playlistMaps.map((pm) => (
              pm.map && <MapCard key={pm.map_key} map={pm.map} />
            ))
          )}
        </div>
      </Modal>
    </div>
  );
}