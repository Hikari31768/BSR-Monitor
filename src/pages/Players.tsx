import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PlayerInfo, PlayerWithScoreCount } from '../types';
import PlayerCard from '../components/PlayerCard';
import Modal from '../components/Modal';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';

export default function Players() {
  const [players, setPlayers] = useState<PlayerWithScoreCount[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState<string | null>(null);
  const [addForm, setAddForm] = useState({ id: '', liveUrl: '', alias: '' });
  const [updatingId, setUpdatingId] = useState<string | null>(null);

  async function loadPlayers() {
    setLoading(true);
    try {
      const playerList = await invoke<PlayerInfo[]>('get_player_list');
      const playersWithCounts = await Promise.all(
        playerList.map(async (player) => {
          const count = await invoke<number>('get_player_score_count', { playerId: player.id });
          return { ...player, score_count: count };
        })
      );
      setPlayers(playersWithCounts);
    } catch (e) {
      console.error('加载玩家失败:', e);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadPlayers();
  }, []);

  async function handleAddPlayer() {
    if (!addForm.id) return;
    try {
      await invoke('add_player', {
        id: addForm.id,
        liveUrl: addForm.liveUrl || null,
        alias: addForm.alias || null
      });
      setShowAddModal(false);
      setAddForm({ id: '', liveUrl: '', alias: '' });
      loadPlayers();
    } catch (e) {
      alert(`添加失败: ${e}`);
    }
  }

  async function handleDeletePlayer(id: string) {
    try {
      await invoke('delete_player', { id });
      setShowDeleteModal(null);
      loadPlayers();
    } catch (e) {
      alert(`删除失败: ${e}`);
    }
  }

  async function handleUpdateScores(playerId: string) {
    setUpdatingId(playerId);
    try {
      const count = await invoke<number>('update_player_scores', { playerId });
      alert(`更新了 ${count} 条成绩`);
      loadPlayers();
    } catch (e) {
      alert(`更新失败: ${e}`);
    } finally {
      setUpdatingId(null);
    }
  }

  if (loading) return <LoadingSpinner />;

  return (
    <div>
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-200">玩家管理</h1>
        <button
          onClick={() => setShowAddModal(true)}
          className="px-4 py-2 bg-bs-accent-primary text-white rounded-lg hover:bg-bs-accent-primary/80"
        >
          添加玩家
        </button>
      </div>

      {players.length === 0 ? (
        <EmptyState message="暂无玩家，点击上方按钮添加" icon="👤" />
      ) : (
        <div className="grid gap-4">
          {players.map(player => (
            <PlayerCard
              key={player.id}
              player={player}
              scoreCount={player.score_count}
              onUpdateScores={() => handleUpdateScores(player.id)}
              onDelete={() => setShowDeleteModal(player.id)}
              updating={updatingId === player.id}
            />
          ))}
        </div>
      )}

      {/* 添加玩家模态框 */}
      <Modal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        title="添加玩家"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">玩家 ID</label>
            <input
              type="text"
              value={addForm.id}
              onChange={(e) => setAddForm({ ...addForm, id: e.target.value })}
              placeholder="如: 76561198035353849"
              className="w-full px-4 py-2 bg-bs-dark-700 border border-bs-dark-600 rounded-lg text-gray-200 focus:border-bs-accent-primary focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">直播间地址 (可选)</label>
            <input
              type="text"
              value={addForm.liveUrl}
              onChange={(e) => setAddForm({ ...addForm, liveUrl: e.target.value })}
              className="w-full px-4 py-2 bg-bs-dark-700 border border-bs-dark-600 rounded-lg text-gray-200 focus:border-bs-accent-primary focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">备注名 (可选)</label>
            <input
              type="text"
              value={addForm.alias}
              onChange={(e) => setAddForm({ ...addForm, alias: e.target.value })}
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
              onClick={handleAddPlayer}
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
        <p className="text-gray-300 mb-4">确定要删除该玩家吗？所有成绩记录将一并删除。</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={() => setShowDeleteModal(null)}
            className="px-4 py-2 bg-bs-dark-700 text-gray-300 rounded-lg hover:bg-bs-dark-600"
          >
            取消
          </button>
          <button
            onClick={() => handleDeletePlayer(showDeleteModal!)}
            className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-500/80"
          >
            删除
          </button>
        </div>
      </Modal>
    </div>
  );
}