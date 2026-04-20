import { useState } from 'react';
import { PlayerInfo } from '../types';

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

  function handleDragStart(playerId: string) {
    setDraggedId(playerId);
  }

  function handleDragOver(e: React.DragEvent, targetId: string) {
    e.preventDefault();
    if (draggedId && draggedId !== targetId) {
      onReorderPlayers(draggedId, targetId);
    }
  }

  function handleDragEnd() {
    setDraggedId(null);
  }

  // 计算网格列定义，与 MapScoreGroup 保持一致
  const gridColumns = `18rem 6rem ${players.map(() => '7rem').join(' ')} 7rem`;

  return (
    <div className="bg-[#1e1e28] border-b border-[#2a2a3a] flex-shrink-0">
      <div
        className="grid"
        style={{ gridTemplateColumns: gridColumns }}
      >
        {/* 谱面信息列 */}
        <div className="p-4 border-r border-[#2a2a3a] flex flex-col items-center justify-center gap-2">
          <span className="text-sm font-medium text-[#a0a0b0]">谱面信息</span>
          {showMapSort && (
            <button
              onClick={onToggleMapSort}
              className={`inline-flex items-center justify-center rounded h-5 px-2 text-xs font-semibold leading-none cursor-pointer transition-colors ${
                mapSort === 'asc'
                  ? 'text-[#4ade80] bg-[#22c55e]/20 hover:bg-[#22c55e]/40'
                  : 'text-[#fb923c] bg-[#f97316]/20 hover:bg-[#f97316]/40'
              }`}
              style={{
                border: mapSort === 'asc'
                  ? '1px solid rgba(34, 197, 94, 0.4)'
                  : '1px solid rgba(249, 115, 22, 0.4)',
              }}
            >
              {mapSort === 'asc' ? '歌单正序' : '歌单倒序'}
            </button>
          )}
        </div>

        {/* 难度信息列 */}
        <div className="p-4 border-r border-[#2a2a3a] flex items-center justify-center">
          <span className="text-sm font-medium text-[#a0a0b0]">难度</span>
        </div>

        {/* 玩家列 */}
        {players.map((player) => (
          <div
            key={player.id}
            className="p-4 border-r border-[#2a2a3a] group relative"
            draggable
            onDragStart={() => handleDragStart(player.id)}
            onDragOver={(e) => handleDragOver(e, player.id)}
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
                <div className="w-10 h-10 rounded-full bg-[#2a2a3a] flex items-center justify-center text-lg">
                  👤
                </div>
              )}
              <span className="text-xs text-[#f0f0f5] truncate w-full text-center">
                {player.alias || player.name}
              </span>
            </div>

            {/* 删除按钮（悬停显示） */}
            <button
              onClick={() => onRemovePlayer(player.id)}
              className="absolute top-1 right-1 w-5 h-5 rounded bg-red-500 text-white text-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center hover:bg-red-600 z-10"
              title="移除玩家"
            >
              ×
            </button>
          </div>
        ))}

        {/* 添加玩家列 */}
        <div className="p-4 flex items-center justify-center">
          <button
            onClick={onAddPlayer}
            className="w-10 h-10 rounded-full bg-[#2a2a3a] hover:bg-[#ff6b6b] text-[#f0f0f5] hover:text-white transition-all flex items-center justify-center text-xl"
            title="添加玩家"
          >
            +
          </button>
        </div>
      </div>
    </div>
  );
}
