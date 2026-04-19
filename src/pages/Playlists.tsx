import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Playlist, PlaylistMap } from '../types';
import PlaylistCard from '../components/PlaylistCard';
import MapCard from '../components/MapCard';
import Modal from '../components/Modal';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';

export default function Playlists() {
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
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
    try {
      await invoke('add_playlist', { playlistId: parseInt(addId) });
      setShowAddModal(false);
      setAddId('');
      loadPlaylists();
    } catch (e) {
      alert(`添加失败: ${e}`);
    }
  }

  async function handleDeletePlaylist(id: number) {
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
          className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all shadow-lg hover:shadow-[#ff6b6b]/30"
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