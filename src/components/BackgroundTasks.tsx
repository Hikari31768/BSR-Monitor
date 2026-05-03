import { useState, useEffect, useRef, useCallback } from 'react';
import { useLocation } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { BackgroundTask } from '../types';
import { Check } from 'lucide-react';

const DISPLAY_TASKS = ['fetch_ratings', 'add_player', 'startup_refresh', 'refresh_ratings', 'refresh_player_scores'];

export default function BackgroundTasks() {
  const location = useLocation();
  const isPlayersPage = location.pathname === '/players';
  const [tasks, setTasks] = useState<Record<string, BackgroundTask>>({});
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // 恢复进行中的后台任务
  const restoreActiveTasks = useCallback(async () => {
    try {
      const rawTasks = await invoke<[string, {
        task_type: string;
        target_id: string;
        progress: number;
        message: string;
        display_name: string;
      }][]>('get_active_tasks');

      const backgroundTasks: Record<string, BackgroundTask> = {};
      for (const [taskId, status] of rawTasks) {
        if (DISPLAY_TASKS.includes(status.task_type)) {
          backgroundTasks[taskId] = {
            task_id: taskId,
            task_type: status.task_type,
            target_id: status.target_id,
            progress: status.progress,
            message: status.message,
            display_name: status.display_name,
          };
          // 如果恢复的任务已完成（进度 100），设置定时器移除
          if (status.progress >= 100) {
            setTimeout(() => {
              setTasks(p => {
                const next = { ...p };
                delete next[taskId];
                return next;
              });
            }, 3000);
          }
        }
      }
      setTasks(prev => ({ ...prev, ...backgroundTasks }));
    } catch (e) {
      console.error('恢复后台任务失败:', e);
    }
  }, []);

  useEffect(() => {
    restoreActiveTasks();
  }, [restoreActiveTasks]);

  // 监听进度事件
  useEffect(() => {
    let cancelled = false;
    listen('progress-update', (event) => {
      if (cancelled) return;
      const payload = event.payload as {
        task_type: string;
        task_id: string;
        stage: string;
        progress: number;
        message?: string;
        player_name?: string;
      };

      if (!DISPLAY_TASKS.includes(payload.task_type)) return;

      const isCompleted = payload.stage === 'completed';

      setTasks(prev => {
        const existing = prev[payload.task_id];
        let displayName = existing?.display_name || '';
        if (!displayName && payload.player_name) {
          displayName = payload.task_type === 'add_player'
            ? `更新玩家: ${payload.player_name}`
            : payload.player_name;
        }
        if (!displayName) {
          displayName = payload.task_type === 'add_player' ? '更新玩家...' : '处理中...';
        }
        if (!displayName && payload.task_type === 'refresh_player_scores') {
          // 使用 display_name 或 player_name 作为显示名
          if (payload.player_name) {
            displayName = `刷新: ${payload.player_name}`;
          }
        }

        if (isCompleted) {
          const updated = {
            ...prev,
            [payload.task_id]: {
              task_id: payload.task_id,
              task_type: payload.task_type,
              target_id: payload.task_id,
              progress: 100,
              message: payload.message || '已完成',
              display_name: displayName,
            },
          };
          setTimeout(() => {
            setTasks(p => {
              const next = { ...p };
              delete next[payload.task_id];
              return next;
            });
          }, 3000);
          return updated;
        }

        return {
          ...prev,
          [payload.task_id]: {
            task_id: payload.task_id,
            task_type: payload.task_type,
            target_id: payload.task_id,
            progress: payload.progress,
            message: payload.message || '处理中...',
            display_name: displayName,
          },
        };
      });
    }).then(unlisten => {
      unlistenRef.current = unlisten;
    });

    return () => {
      cancelled = true;
      unlistenRef.current?.();
      unlistenRef.current = null;
    };
  }, []);

  const taskList = Object.values(tasks).filter(task => {
    // 在玩家管理页面时，add_player 进度已在卡片上显示，浮窗不再重复
    if (isPlayersPage && task.task_type === 'add_player') return false;
    return true;
  });
  if (taskList.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 w-72 animate-slide-up">
      {taskList.map(task => {
        const isCompleted = task.progress >= 100;
        return (
          <div
            key={task.task_id}
            className="bg-bs-bg-secondary border border-bs-border rounded-xl shadow-lg p-3 animate-fade-in"
          >
            <div className="flex items-center justify-between mb-1">
              <span className="text-sm font-medium text-bs-text-primary truncate">
                {task.display_name}
              </span>
              {isCompleted && (
                <Check size={14} className="text-bs-accent-primary flex-shrink-0 ml-2" />
              )}
            </div>
            <div className="w-full h-1.5 bg-bs-bg-tertiary rounded-full overflow-hidden">
              <div
                className="h-full bg-bs-accent-primary rounded-full transition-all duration-300"
                style={{ width: `${Math.min(task.progress, 100)}%` }}
              />
            </div>
            {task.message && (
              <p className="text-xs text-bs-text-muted mt-1 truncate">
                {task.message}
              </p>
            )}
          </div>
        );
      })}
    </div>
  );
}
