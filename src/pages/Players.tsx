import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { PlayerInfo, PlayerWithScoreCount } from '../types';
import PlayerCard from '../components/PlayerCard';
import Modal from '../components/Modal';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';

export default function Players() {
  const [players, setPlayers] = useState<(PlayerWithScoreCount & { loading?: boolean; progress?: number; status_message?: string })[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState<string | null>(null);
  const [addForm, setAddForm] = useState({ id: '', liveUrl: '', alias: '' });
  const [updatingId, setUpdatingId] = useState<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

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

  // 恢复进行中的任务状态
  const restoreActiveTasks = useCallback(async () => {
    try {
      const tasks = await invoke<[string, { task_type: string; target_id: string; progress: number; message: string }][]>('get_active_tasks');
      const playerTasks = tasks.filter(([, s]) => s.task_type === 'add_player');
      if (playerTasks.length === 0) return;

      setPlayers(prev => {
        const updated = [...prev];
        for (const [, status] of playerTasks) {
          const idx = updated.findIndex(p => p.id === status.target_id);
          if (idx !== -1) {
            updated[idx] = {
              ...updated[idx],
              loading: true,
              progress: status.progress,
              status_message: status.message || '处理中...'
            };
          }
        }
        return updated;
      });
    } catch (e) {
      console.error('恢复任务状态失败:', e);
    }
  }, []);

  useEffect(() => {
    loadPlayers().then(() => restoreActiveTasks());
  }, [restoreActiveTasks]);

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
        player_name?: string;
        player_avatar_url?: string;
      };

      // 只处理 add_player 类型的进度
      if (progressData.task_type === 'add_player') {
        setPlayers(prev => prev.map(p => {
          // 通过后端返回的 task_id 和前端存储的 currentTaskId 匹配
          if ((p as any).currentTaskId && (p as any).currentTaskId === progressData.task_id) {
            return {
              ...p,
              name: progressData.player_name || p.name,
              avatar_url: progressData.player_avatar_url || p.avatar_url,
              progress: progressData.progress,
              status_message: progressData.message || `阶段: ${progressData.stage}`
            };
          }
          return p;
        }));
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

  async function handleAddPlayer() {
    if (!addForm.id) return;

    // 检查是否已存在
    const existingIdx = players.findIndex(p => p.id === addForm.id);
    const isExisting = existingIdx !== -1;
    const isProcessing = isExisting
      ? players[existingIdx].loading
      : false;

    if (isProcessing) {
      alert('该玩家正在处理中，请稍候...');
      return;
    }

    // 前端生成 task_id，确保进度事件能正确匹配
    const taskId = crypto.randomUUID();

    try {
      // 如果已存在，更新现有条目状态；否则添加临时条目
      const tempPlayer: PlayerWithScoreCount & { loading?: boolean; progress?: number; status_message?: string; currentTaskId?: string } = {
        ...(isExisting ? players[existingIdx] : {
          id: addForm.id,
          name: '加载中...',
          avatar_url: undefined,
          live_url: addForm.liveUrl || undefined,
          alias: addForm.alias || undefined,
          score_count: 0,
        }),
        loading: true,
        progress: 0,
        status_message: isExisting ? '正在更新...' : '准备添加...',
        currentTaskId: taskId  // 设置 task_id 以便进度监听器匹配
      };

      if (isExisting) {
        setPlayers(prev => prev.map((p, i) => i === existingIdx ? { ...p, ...tempPlayer } : p));
      } else {
        setPlayers(prev => [...prev, tempPlayer]);
      }
      setShowAddModal(false);
      setAddForm({ id: '', liveUrl: '', alias: '' });

      // 执行长时间操作，将前端生成的 task_id 传给后端
      await invoke<{ task_id: string; player: PlayerInfo }>('add_player', {
        id: addForm.id,
        liveUrl: addForm.liveUrl || null,
        alias: addForm.alias || null,
        taskId  // 传递前端生成的 task_id
      });

      // 操作成功，重新加载列表
      await loadPlayers();
      await restoreActiveTasks();
    } catch (e) {
      // 操作失败，恢复原有状态或移除临时条目
      if (isExisting) {
        setPlayers(prev => prev.map((p, i) => i === existingIdx ? { ...p, loading: false, progress: undefined, status_message: undefined, currentTaskId: undefined } : p));
      } else {
        setPlayers(prev => prev.filter(p => p.id !== addForm.id));
      }
      alert(`${isExisting ? '更新' : '添加'}失败: ${e}`);
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
    <div className="animate-fade-in">
      {/* 页面标题 */}
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold text-[#f0f0f5] tracking-tight">👤 玩家管理</h1>
        <button
          onClick={() => setShowAddModal(true)}
          className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all shadow-lg hover:shadow-[#ff6b6b]/30 disabled:opacity-50"
          disabled={loading}
        >
          ➕ 添加玩家
        </button>
      </div>

      {/* 玩家列表 */}
      {players.length === 0 ? (
        <EmptyState message="暂无玩家，点击上方按钮添加" icon="👤" />
      ) : (
        <div className="grid gap-4" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(360px, 1fr))' }}>
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
            <label className="block text-sm text-[#a0a0b0] mb-2">玩家 ID</label>
            <input
              type="text"
              value={addForm.id}
              onChange={(e) => setAddForm({ ...addForm, id: e.target.value })}
              placeholder="如: 76561198035353849"
              className="w-full px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] placeholder-[#6b7280] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
            />
          </div>
          <div>
            <label className="block text-sm text-[#a0a0b0] mb-2">直播间地址 (可选)</label>
            <input
              type="text"
              value={addForm.liveUrl}
              onChange={(e) => setAddForm({ ...addForm, liveUrl: e.target.value })}
              placeholder="https://..."
              className="w-full px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] placeholder-[#6b7280] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
            />
          </div>
          <div>
            <label className="block text-sm text-[#a0a0b0] mb-2">备注名 (可选)</label>
            <input
              type="text"
              value={addForm.alias}
              onChange={(e) => setAddForm({ ...addForm, alias: e.target.value })}
              placeholder="自定义名称"
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
              onClick={handleAddPlayer}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all"
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
        <p className="text-[#a0a0b0] mb-6">确定要删除该玩家吗？所有成绩记录将一并删除。</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={() => setShowDeleteModal(null)}
            className="px-4 py-2.5 rounded-lg font-medium text-sm bg-[#1e1e28] text-[#a0a0b0] hover:bg-[#252533] hover:text-[#f0f0f5] transition-all"
          >
            取消
          </button>
          <button
            onClick={() => handleDeletePlayer(showDeleteModal!)}
            className="px-4 py-2.5 rounded-lg font-medium text-sm bg-red-500 text-white hover:bg-red-600 transition-all"
          >
            删除
          </button>
        </div>
      </Modal>
    </div>
  );
}
