import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { PlayerInfo, PlayerWithScoreCount, PlayerUIState } from '../types';
import PlayerCard from '../components/PlayerCard';
import Modal from '../components/Modal';
import EmptyState from '../components/EmptyState';
import { Plus } from 'lucide-react';

export default function Players() {
  const [players, setPlayers] = useState<PlayerWithScoreCount[]>([]);
  const [playerUIStates, setPlayerUIStates] = useState<Record<string, PlayerUIState>>({});
  const [playerTaskIds, setPlayerTaskIds] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState<string | null>(null);
  const [addForm, setAddForm] = useState({ id: '', liveUrl: '', alias: '' });
  const [addFormError, setAddFormError] = useState<string>('');
  const [updatingId, setUpdatingId] = useState<string | null>(null);
  const [updateError, setUpdateError] = useState<string>('');
  const unlistenRef = useRef<UnlistenFn | null>(null);

  async function loadPlayers(): Promise<PlayerWithScoreCount[]> {
    setLoading(true);
    try {
      const playersWithCounts = await invoke<PlayerWithScoreCount[]>('get_players_with_score_counts');
      setPlayers(playersWithCounts);
      return playersWithCounts;
    } catch (e) {
      console.error('加载玩家失败:', e);
      return [];
    } finally {
      setLoading(false);
    }
  }

  // 静默刷新数据，不触发 loading 状态
  async function refreshPlayers(): Promise<PlayerWithScoreCount[]> {
    try {
      const playersWithCounts = await invoke<PlayerWithScoreCount[]>('get_players_with_score_counts');
      setPlayers(playersWithCounts);
      return playersWithCounts;
    } catch (e) {
      console.error('刷新玩家失败:', e);
      return [];
    }
  }

  // 恢复进行中的任务状态
  const restoreActiveTasks = useCallback(async () => {
    try {
      const tasks = await invoke<[string, { task_type: string; target_id: string; progress: number; message: string }][]>('get_active_tasks');
      const playerTasks = tasks.filter(([, s]) => s.task_type === 'add_player');
      if (playerTasks.length === 0) return;

      setPlayerUIStates(prev => {
        const updated = { ...prev };
        for (const [, status] of playerTasks) {
          updated[status.target_id] = {
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
    loadPlayers().then(() => restoreActiveTasks());
  }, [restoreActiveTasks]);

  // 使用 ref 存储 taskId 映射，避免频繁重新订阅事件
  const playerTaskIdsRef = useRef<Record<string, string>>({});
  useEffect(() => {
    playerTaskIdsRef.current = playerTaskIds;
  }, [playerTaskIds]);

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
        // 查找对应此 task_id 的 playerId
        const playerId = Object.keys(playerTaskIdsRef.current).find(
          id => playerTaskIdsRef.current[id] === progressData.task_id
        );
        if (!playerId) return;

        // 更新玩家基本信息（名称、头像）
        setPlayers(prev => prev.map(p => {
          if (p.id === playerId) {
            return {
              ...p,
              name: progressData.player_name || p.name,
              avatar_url: progressData.player_avatar_url || p.avatar_url,
            };
          }
          return p;
        }));

        const isCompleted = progressData.stage === 'completed';

        // 更新 UI 状态
        setPlayerUIStates(prev => {
          if (isCompleted) {
            const updated = { ...prev };
            delete updated[playerId];
            return updated;
          }
          return {
            ...prev,
            [playerId]: {
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

  async function handleAddPlayer() {
    if (!addForm.id.trim()) return;

    // 验证玩家 ID 格式（纯数字）
    if (!/^\d+$/.test(addForm.id.trim())) {
      setAddFormError('玩家 ID 必须为纯数字');
      return;
    }

    setAddFormError('');

    // 检查是否已存在
    const existingIdx = players.findIndex(p => p.id === addForm.id.trim());
    const isExisting = existingIdx !== -1;
    const uiState = playerUIStates[addForm.id.trim()];
    const isProcessing = isExisting && uiState?.loading;

    if (isProcessing) {
      setAddFormError('该玩家正在处理中，请稍候...');
      return;
    }

    // 前端生成 task_id，确保进度事件能正确匹配
    const taskId = crypto.randomUUID();

    try {
      // 更新 taskId 映射
      setPlayerTaskIds(prev => ({ ...prev, [addForm.id.trim()]: taskId }));

      // 设置 UI 状态
      setPlayerUIStates(prev => ({
        ...prev,
        [addForm.id.trim()]: {
          loading: true,
          progress: 0,
          statusMessage: isExisting ? '正在更新...' : '准备添加...'
        }
      }));

      // 如果已存在，更新现有条目状态；否则添加临时条目
      if (!isExisting) {
        const tempPlayer: PlayerWithScoreCount = {
          id: addForm.id.trim(),
          name: '加载中...',
          avatar_url: undefined,
          live_url: addForm.liveUrl || undefined,
          alias: addForm.alias || undefined,
          score_count: 0,
        };
        setPlayers(prev => [...prev, tempPlayer]);
      } else {
        // 更新现有玩家名称显示
        setPlayers(prev => prev.map((p, i) => i === existingIdx ? { ...p, name: '加载中...' } : p));
      }

      setShowAddModal(false);
      setAddForm({ id: '', liveUrl: '', alias: '' });
      setAddFormError('');

      // 执行长时间操作，将前端生成的 task_id 传给后端
      await invoke<{ task_id: string; player: PlayerInfo }>('add_player', {
        id: addForm.id.trim(),
        liveUrl: addForm.liveUrl || null,
        alias: addForm.alias || null,
        taskId  // 传递前端生成的 task_id
      });

      // 操作成功，静默刷新列表
      await refreshPlayers();
      await restoreActiveTasks();
    } catch (e) {
      // 操作失败，恢复 UI 状态
      setPlayerUIStates(prev => {
        const updated = { ...prev };
        delete updated[addForm.id.trim()];
        return updated;
      });
      setPlayerTaskIds(prev => {
        const updated = { ...prev };
        delete updated[addForm.id.trim()];
        return updated;
      });

      if (!isExisting) {
        setPlayers(prev => prev.filter(p => p.id !== addForm.id.trim()));
      }
      setAddFormError(`${isExisting ? '更新' : '添加'}失败: ${e}`);
    }
  }

  async function handleDeletePlayer(id: string) {
    try {
      await invoke('delete_player', { id });
      setShowDeleteModal(null);
      loadPlayers();
    } catch (e) {
      setUpdateError(`删除失败: ${e}`);
    }
  }

  async function handleUpdateScores(playerId: string) {
    setUpdatingId(playerId);
    setUpdateError('');

    const oldScoreCount = players.find(p => p.id === playerId)?.score_count ?? 0;

    setPlayerUIStates(prev => ({
      ...prev,
      [playerId]: {
        loading: false,
        progress: 0,
        statusMessage: '',
        updatingMessage: '更新中...'
      }
    }));

    try {
      await invoke('add_player', { id: playerId, liveUrl: null, alias: null });
      const newList = await refreshPlayers();
      await restoreActiveTasks();

      const newScoreCount = newList.find(p => p.id === playerId)?.score_count ?? oldScoreCount;
      const delta = Math.max(0, newScoreCount - oldScoreCount);

      setPlayerUIStates(prev => ({
        ...prev,
        [playerId]: {
          loading: false,
          progress: 0,
          statusMessage: '',
          updatingMessage: `更新成绩 ${delta} 条`
        }
      }));

      setTimeout(() => {
        setPlayerUIStates(prev => {
          const updated = { ...prev };
          delete updated[playerId];
          return updated;
        });
      }, 2000);
    } catch (e) {
      setPlayerUIStates(prev => {
        const updated = { ...prev };
        delete updated[playerId];
        return updated;
      });
      setUpdateError(`更新失败: ${e}`);
    } finally {
      setUpdatingId(null);
    }
  }

  return (
    <div className="animate-fade-in">
      {/* 页面标题 */}
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold text-bs-text-primary tracking-tight">玩家管理</h1>
        <button
          onClick={() => setShowAddModal(true)}
          className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-accent-primary text-white hover:bg-bs-accent-secondary transition-all shadow-lg hover:shadow-bs-accent-primary/30 disabled:opacity-50 flex items-center gap-2"
          disabled={loading}
        >
          <Plus size={18} />
          添加玩家
        </button>
      </div>

      {/* 玩家列表 */}
      {players.length === 0 ? (
        <EmptyState message="暂无玩家，点击上方按钮添加" />
      ) : (
        <div className="grid gap-4" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(360px, 1fr))' }}>
          {players.map(player => (
            <PlayerCard
              key={player.id}
              player={player}
              uiState={playerUIStates[player.id]}
              scoreCount={player.score_count}
              onUpdateScores={() => { setUpdateError(''); handleUpdateScores(player.id); }}
              onDelete={() => setShowDeleteModal(player.id)}
              updating={updatingId === player.id}
            />
          ))}
        </div>
      )}

      {/* 更新错误提示 */}
      {updateError && (
        <div className="mt-4 p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
          {updateError}
          <button
            onClick={() => setUpdateError('')}
            className="ml-2 text-red-400 hover:text-red-300"
          >
            关闭
          </button>
        </div>
      )}

      {/* 添加玩家模态框 */}
      <Modal
        isOpen={showAddModal}
        onClose={() => { setShowAddModal(false); setAddFormError(''); }}
        title="添加玩家"
      >
        <div className="space-y-4">
          {addFormError && (
            <div className="p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
              {addFormError}
            </div>
          )}
          <div>
            <label className="block text-sm text-bs-text-secondary mb-2">
              玩家 ID <span className="text-red-400">*</span>
            </label>
            <input
              type="text"
              inputMode="numeric"
              pattern="\d*"
              value={addForm.id}
              onChange={(e) => {
                setAddForm({ ...addForm, id: e.target.value.replace(/[^\d]/g, '') });
                setAddFormError('');
              }}
              placeholder="如: 76561198035353849"
              className="w-full px-4 py-3 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary placeholder-bs-text-muted focus:border-bs-accent-primary focus:ring-2 focus:ring-bs-accent-primary/20 focus:outline-none transition-all"
            />
            <p className="text-xs text-bs-text-muted mt-1">请输入纯数字 Steam ID</p>
          </div>
          <div>
            <label className="block text-sm text-bs-text-secondary mb-2">直播间地址 (可选)</label>
            <input
              type="text"
              value={addForm.liveUrl}
              onChange={(e) => setAddForm({ ...addForm, liveUrl: e.target.value })}
              placeholder="https://..."
              className="w-full px-4 py-3 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary placeholder-bs-text-muted focus:border-bs-accent-primary focus:ring-2 focus:ring-bs-accent-primary/20 focus:outline-none transition-all"
            />
          </div>
          <div>
            <label className="block text-sm text-bs-text-secondary mb-2">备注名 (可选)</label>
            <input
              type="text"
              value={addForm.alias}
              onChange={(e) => setAddForm({ ...addForm, alias: e.target.value })}
              placeholder="自定义名称"
              className="w-full px-4 py-3 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary placeholder-bs-text-muted focus:border-bs-accent-primary focus:ring-2 focus:ring-bs-accent-primary/20 focus:outline-none transition-all"
            />
          </div>
          <div className="flex gap-3 justify-end pt-4">
            <button
              onClick={() => { setShowAddModal(false); setAddFormError(''); }}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all"
            >
              取消
            </button>
            <button
              onClick={handleAddPlayer}
              disabled={!addForm.id.trim()}
              className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-accent-primary text-white hover:bg-bs-accent-secondary transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              添加
            </button>
          </div>
        </div>
      </Modal>

      {/* 删除确认模态框 */}
      <Modal
        isOpen={showDeleteModal !== null}
        onClose={() => { setShowDeleteModal(null); setUpdateError(''); }}
        title="确认删除"
      >
        {updateError && (
          <div className="p-3 rounded-lg bg-red-500/15 text-red-400 border border-red-500/30 text-sm">
            {updateError}
          </div>
        )}
        <p className="text-bs-text-secondary mb-6">确定要删除该玩家吗？所有成绩记录将一并删除。</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={() => { setShowDeleteModal(null); setUpdateError(''); }}
            className="px-4 py-2.5 rounded-lg font-medium text-sm bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all"
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
