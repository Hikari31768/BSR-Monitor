import { useState, useRef, useCallback, useEffect } from 'react';
import { PlayerInfo, PlayerFilterState } from '../types';
import { User, Plus, X, RefreshCw, ArrowRight, ArrowLeft, Check, Minus } from 'lucide-react';

interface ScoreTableHeaderProps {
  players: PlayerInfo[];
  onAddPlayer: () => void;
  onRemovePlayer: (playerId: string) => void;
  onRefreshPlayer: (playerId: string) => void;
  onReorderPlayers: (draggedId: string, targetId: string) => void;
  mapSort: 'asc' | 'desc';
  onToggleMapSort: () => void;
  showMapSort: boolean;
  diffSort: 'asc' | 'desc';
  onToggleDiffSort: () => void;
  viewMode: 'single' | 'playlist' | null;
  playerFilters: Record<string, PlayerFilterState>;
  onTogglePlayerFilter: (playerId: string, type: 'has_scores' | 'no_scores') => void;
}

export default function ScoreTableHeader({
  players,
  onAddPlayer,
  onRemovePlayer,
  onRefreshPlayer,
  onReorderPlayers,
  mapSort,
  onToggleMapSort,
  showMapSort,
  diffSort,
  onToggleDiffSort,
  viewMode,
  playerFilters,
  onTogglePlayerFilter,
}: ScoreTableHeaderProps) {
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);
  const [cursorPos, setCursorPos] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);
  const cellRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const dragStateRef = useRef<{ src: string | null; dst: string | null }>({ src: null, dst: null });

  function findPlayerIdAtPoint(x: number, y: number): string | null {
    const el = document.elementFromPoint(x, y);
    if (!el) return null;
    for (const [id, ref] of cellRefs.current) {
      if (ref === el || ref.contains(el)) return id;
    }
    return null;
  }

  const handleMouseMove = useCallback((e: MouseEvent) => {
    const src = dragStateRef.current.src;
    if (!src) return;
    setCursorPos({ x: e.clientX, y: e.clientY });
    const targetId = findPlayerIdAtPoint(e.clientX, e.clientY);
    dragStateRef.current.dst = targetId || null;
    setDropTargetId(targetId || null);
  }, []);

  const handleMouseUp = useCallback(() => {
    const src = dragStateRef.current.src;
    const dst = dragStateRef.current.dst;
    if (src && dst && src !== dst) {
      onReorderPlayers(src, dst);
    }
    setDraggedId(null);
    setDropTargetId(null);
    dragStateRef.current = { src: null, dst: null };
  }, [onReorderPlayers]);

  useEffect(() => {
    if (!draggedId) return;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [draggedId, handleMouseMove, handleMouseUp]);

  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>, playerId: string) => {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest('button')) return;

    dragStateRef.current = { src: playerId, dst: null };
    setCursorPos({ x: e.clientX, y: e.clientY });
    setDraggedId(playerId);
  }, []);

  const draggedPlayer = draggedId ? players.find(p => p.id === draggedId) : null;

  // 计算网格列定义，与 MapScoreGroup 保持一致
  const gridColumns = `18rem 9rem ${players.map(() => '7rem').join(' ')} minmax(7rem, 1fr)`;

  return (
    <div className="sticky top-0 z-10 flex-shrink-0">
      <div
        ref={containerRef}
        className="grid bg-bs-bg-tertiary border-b border-bs-border min-w-fit min-h-[6rem]"
        style={{ gridTemplateColumns: gridColumns }}
      >
        {/* 谱面信息列 */}
        <div
          className="sticky-col-1 p-4 border-r border-bs-border flex items-center justify-center gap-1"
          style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
        >
          <span className="text-sm font-medium text-bs-text-secondary">谱面信息</span>
          {showMapSort && (
            <button
              onClick={onToggleMapSort}
              className="inline-flex items-center text-bs-text-secondary hover:text-bs-text-primary transition-colors cursor-pointer"
              title="切换歌单排序"
            >
              {mapSort === 'asc' ? (
                <ArrowRight size={14} className="text-bs-status-success" />
              ) : (
                <ArrowLeft size={14} className="text-bs-status-warning" />
              )}
            </button>
          )}
        </div>

        {/* 难度信息列 */}
        <div
          className="sticky-col-2 p-4 border-r border-bs-border flex items-center justify-center gap-1"
          style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
        >
          <span className="text-sm font-medium text-bs-text-secondary">难度</span>
          <button
            onClick={onToggleDiffSort}
            className="inline-flex items-center text-bs-text-secondary hover:text-bs-text-primary transition-colors cursor-pointer"
            title="切换难度排序"
          >
            {diffSort === 'desc' ? (
              <ArrowLeft size={14} className="text-bs-status-warning" />
            ) : (
              <ArrowRight size={14} className="text-bs-status-success" />
            )}
          </button>
        </div>

        {/* 玩家列 */}
        {players.map((player) => {
          const isDragged = draggedId === player.id;
          const isDropTarget = dropTargetId === player.id && !isDragged;
          const filter = playerFilters[player.id] ?? 'none';
          const isFilterActive = filter !== 'none';
          const showFilterButtons = viewMode === 'playlist';

          return (
            <div
              key={player.id}
              ref={(el) => {
                if (el) cellRefs.current.set(player.id, el);
                else cellRefs.current.delete(player.id);
              }}
              className={`p-4 border-r border-bs-border group relative transition-all ${
                isDropTarget
                  ? 'bg-bs-accent-primary/15 ring-2 ring-bs-accent-primary/50 ring-inset scale-105'
                  : ''
              } ${
                isDragged
                  ? 'opacity-30 border-2 border-dashed border-bs-accent-primary'
                  : ''
              }`}
              onMouseDown={(e) => handleMouseDown(e, player.id)}
            >
              {/* 有成绩筛选按钮 */}
              {showFilterButtons && (
                <button
                  onClick={() => onTogglePlayerFilter(player.id, 'has_scores')}
                  className="absolute top-1 left-1 w-5 h-5 rounded bg-bs-status-success text-white opacity-0 group-hover:opacity-100 touch-manipulation:opacity-100 transition-opacity flex items-center justify-center hover:bg-green-600 z-10"
                  title="筛选此人有过成绩的谱面（任何难度）"
                  aria-label={`筛选 ${player.alias || player.name} 有过成绩的谱面`}
                >
                  <Check size={12} />
                </button>
              )}

              {/* 无成绩筛选按钮 */}
              {showFilterButtons && (
                <button
                  onClick={() => onTogglePlayerFilter(player.id, 'no_scores')}
                  className="absolute top-7 left-1 w-5 h-5 rounded bg-gray-500 text-white opacity-0 group-hover:opacity-100 touch-manipulation:opacity-100 transition-opacity flex items-center justify-center hover:bg-gray-600 z-10"
                  title="筛选此人没有成绩的谱面（所有难度）"
                  aria-label={`筛选 ${player.alias || player.name} 没有成绩的谱面`}
                >
                  <Minus size={12} />
                </button>
              )}

              <div className="flex flex-col items-center gap-2 cursor-grab active:cursor-grabbing select-none">
                <div className="relative">
                  {player.avatar_url ? (
                    <img
                      src={player.avatar_url}
                      alt={player.name}
                      className="w-10 h-10 rounded-full object-cover pointer-events-none"
                    />
                  ) : (
                    <div className="w-10 h-10 rounded-full bg-bs-bg-hover flex items-center justify-center pointer-events-none">
                      <User size={20} className="text-bs-text-muted pointer-events-none" />
                    </div>
                  )}
                  {isFilterActive && (
                    <span
                      className={`absolute -top-0.5 -right-0.5 w-3.5 h-3.5 rounded-full border-2 ${
                        filter === 'has_scores' ? 'bg-bs-status-success' : 'bg-gray-500'
                      }`}
                      style={{ borderColor: 'var(--color-bg-tertiary)' }}
                    />
                  )}
                </div>
                <span className="text-xs text-bs-text-primary truncate w-full text-center pointer-events-none">
                  {player.alias || player.name}
                </span>
              </div>

              {/* 删除按钮 */}
              <button
                onClick={() => onRemovePlayer(player.id)}
                className="absolute top-1 right-1 w-5 h-5 rounded bg-red-500 text-white opacity-0 group-hover:opacity-100 touch-manipulation:opacity-100 transition-opacity flex items-center justify-center hover:bg-red-600 z-10"
                title="移除玩家"
                aria-label={`移除玩家 ${player.alias || player.name}`}
              >
                <X size={12} />
              </button>

              {/* 刷新成绩按钮 */}
              <button
                onClick={() => onRefreshPlayer(player.id)}
                className="absolute top-7 right-1 w-5 h-5 rounded bg-bs-accent-primary text-white opacity-0 group-hover:opacity-100 touch-manipulation:opacity-100 transition-opacity flex items-center justify-center hover:bg-bs-accent-secondary z-10"
                title="刷新成绩"
                aria-label={`刷新 ${player.alias || player.name} 的成绩`}
              >
                <RefreshCw size={12} />
              </button>
            </div>
          );
        })}

        {/* 添加玩家列 */}
        <div className="p-4 flex items-center justify-center">
          <button
            onClick={onAddPlayer}
            className="w-12 h-12 rounded-full bg-bs-bg-hover hover:bg-bs-accent-primary text-bs-text-primary hover:text-white transition-all flex items-center justify-center"
            title="添加玩家"
          >
            <Plus size={20} />
          </button>
        </div>
      </div>

      {/* 浮动拖拽预览 */}
      {draggedId && draggedPlayer && (
        <div
          className="fixed z-50 pointer-events-none animate-fade-in"
          style={{
            left: cursorPos.x + 12,
            top: cursorPos.y - 16,
            transform: 'translate(-50%, -50%)',
          }}
        >
          <div className="bg-bs-bg-secondary border border-bs-accent-primary/50 rounded-xl shadow-lg shadow-bs-accent-primary/30 px-3 py-2 flex items-center gap-2">
            {draggedPlayer.avatar_url ? (
              <img
                src={draggedPlayer.avatar_url}
                alt={draggedPlayer.name}
                className="w-8 h-8 rounded-full object-cover"
              />
            ) : (
              <div className="w-8 h-8 rounded-full bg-bs-bg-hover flex items-center justify-center">
                <User size={16} className="text-bs-text-muted" />
              </div>
            )}
            <span className="text-xs font-medium text-bs-text-primary whitespace-nowrap">
              {draggedPlayer.alias || draggedPlayer.name}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
