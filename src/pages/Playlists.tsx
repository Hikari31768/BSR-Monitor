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
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-200">歌单管理</h1>
        <button
          onClick={() => setShowAddModal(true)}
          className="px-4 py-2 bg-bs-accent-primary text-white rounded-lg hover:bg-bs-accent-primary/80"
        >
          添加歌单
        </button>
      </div>

      {playlists.length === 0 ? (
        <EmptyState message="暂无歌单，点击上方按钮添加" icon="📋" />
      ) : (
        <div className="grid gap-4">
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

      {/* 添加歌单模态框 */}
      <Modal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        title="添加歌单"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">BeatSaver 歌单 ID</label>
            <input
              type="text"
              value={addId}
              onChange={(e) => setAddId(e.target.value)}
              placeholder="如: 1"
              className="w-full px-4 py-2 bg-bs-dark-700 border border-bs-dark-600 rounded-lg text-gray-200 focus:border-bs-accent-primary focus:outline-none"
            />
          </div>
          <div className="flex gap-3 justify-end">
            <button
              onClick={() => setShowAddModal(false)}
              className="px-4 py-2 bg-bs-dark-700 text-gray-300 rounded-lg hover:bg-bs-dark-600"
            >
              取消
            </button>
            <button
              onClick={handleAddPlaylist}
              className="px-4 py-2 bg-bs-accent-primary text-white rounded-lg hover:bg-bs-accent-primary/80"
            >
              添加
            </button>
          </div>
        </div>
      </Modal>

      {/* 删除确认模态框 */}
      <Modal
        isOpen={showDeleteModal !== null}
        onClose={() => setShowDeleteModal(null)}
        title="确认删除"
      >
        <p className="text-gray-300 mb-4">确定要删除该歌单吗？</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={() => setShowDeleteModal(null)}
            className="px-4 py-2 bg-bs-dark-700 text-gray-300 rounded-lg hover:bg-bs-dark-600"
          >
            取消
          </button>
          <button
            onClick={() => handleDeletePlaylist(showDeleteModal!)}
            className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-500/80"
          >
            删除
          </button>
        </div>
      </Modal>

      {/* 查看谱面列表模态框 */}
      <Modal
        isOpen={showMapsModal !== null}
        onClose={() => setShowMapsModal(null)}
        title="歌单谱面列表"
      >
        <div className="max-h-[400px] overflow-auto space-y-3">
          {playlistMaps.length === 0 ? (
            <p className="text-gray-400">暂无谱面</p>
          ) : (
            playlistMaps.map((pm) => (
              pm.map && (
                <MapCard key={pm.map_key} map={pm.map} />
              )
            ))
          )}
        </div>
      </Modal>
    </div>
  );
}