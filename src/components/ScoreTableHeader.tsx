import { useState } from 'react';
import { PlayerInfo } from '../types';
import { User, Plus, X } from 'lucide-react';

interface ScoreTableHeaderProps {
  players: PlayerInfo[];
  onAddPlayer: () => void;
  onRemovePlayer: (playerId: string) => void;
  onReorderPlayers: (draggedId: string, targetId: string) => void;
  mapSort: 'asc' | 'desc';
  onToggleMapSort: () => void;
  showMapSort: boolean;
}

export default function ScoreTableHeader({
  players,
  onAddPlayer,
  onRemovePlayer,
  onReorderPlayers,
  mapSort,
  onToggleMapSort,
  showMapSort,
}: ScoreTableHeaderProps) {
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);

  function handleDragStart(playerId: string) {
    setDraggedId(playerId);
  }

  function handleDragOver(e: React.DragEvent, targetId: string) {
    e.preventDefault();
    setDropTargetId(targetId);
    if (draggedId && draggedId !== targetId) {
      onReorderPlayers(draggedId, targetId);
    }
  }

  function handleDragEnd() {
    setDraggedId(null);
    setDropTargetId(null);
  }

  // 计算网格列定义，与 MapScoreGroup 保持一致
  const gridColumns = `18rem 6rem ${players.map(() => '7rem').join(' ')} minmax(7rem, 1fr)`;

  return (
    <div className="rounded-xl flex-shrink-0">
      <div
        className="grid bg-bs-bg-tertiary border-b border-bs-border sticky top-0 z-10 min-w-fit min-h-[6rem]"
        style={{ gridTemplateColumns: gridColumns }}
      >
        {/* 谱面信息列 */}
        <div className="p-4 border-r border-bs-border flex flex-col items-center justify-center gap-2">
          <span className="text-sm font-medium text-bs-text-secondary">谱面信息</span>
          {showMapSort && (
            <button
              onClick={onToggleMapSort}
              className={`inline-flex items-center justify-center rounded h-5 px-2 text-xs font-semibold leading-none cursor-pointer transition-colors border ${
                mapSort === 'asc'
                  ? 'text-bs-status-success bg-bs-status-success/20 hover:bg-bs-status-success/40 border-bs-status-success/40'
                  : 'text-bs-accent-secondary bg-bs-accent-secondary/20 hover:bg-bs-accent-secondary/40 border-bs-accent-secondary/40'
              }`}
            >
              {mapSort === 'asc' ? '歌单正序' : '歌单倒序'}
            </button>
          )}
        </div>

        {/* 难度信息列 */}
        <div className="p-4 border-r border-bs-border flex items-center justify-center">
          <span className="text-sm font-medium text-bs-text-secondary">难度</span>
        </div>

        {/* 玩家列 */}
        {players.map((player) => (
          <div
            key={player.id}
            className={`p-4 border-r border-bs-border group relative transition-colors ${
              dropTargetId === player.id && draggedId !== player.id
                ? 'bg-bs-accent-primary/10'
                : ''
            }`}
            draggable
            onDragStart={() => handleDragStart(player.id)}
            onDragOver={(e) => handleDragOver(e, player.id)}
            onDragLeave={() => setDropTargetId(null)}
            onDragEnd={handleDragEnd}
          >
            <div className="flex flex-col items-center gap-2 cursor-move">
              {player.avatar_url ? (
                <img
                  src={player.avatar_url}
                  alt={player.name}
                  className="w-10 h-10 rounded-full object-cover"
                />
              ) : (
                <div className="w-10 h-10 rounded-full bg-bs-bg-hover flex items-center justify-center">
                  <User size={20} className="text-bs-text-muted" />
                </div>
              )}
              <span className="text-xs text-bs-text-primary truncate w-full text-center">
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
          </div>
        ))}

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
    </div>
  );
}
