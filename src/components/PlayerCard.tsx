import { PlayerInfo } from '../types';

interface PlayerCardProps {
  player: PlayerInfo;
  scoreCount: number;
  onUpdateScores: () => void;
  onDelete: () => void;
  updating?: boolean;
}

export default function PlayerCard({ player, scoreCount, onUpdateScores, onDelete, updating }: PlayerCardProps) {
  const displayName = player.alias || player.name;

  return (
    <div className="bg-[#15151c] rounded-xl p-5 border border-[#2a2a3a] shadow-lg transition-all duration-300 hover:border-[#ff6b6b]/50 hover:shadow-xl hover:shadow-[#ff6b6b]/5 group">
      <div className="flex items-start gap-4">
        {/* 头像 */}
        {player.avatar_url ? (
          <img
            src={player.avatar_url}
            alt={displayName}
            className="w-14 h-14 rounded-xl object-cover ring-2 ring-[#2a2a3a] group-hover:ring-[#ff6b6b]/50 transition-all"
          />
        ) : (
          <div className="w-14 h-14 rounded-xl bg-[#1e1e28] flex items-center justify-center text-2xl ring-2 ring-[#2a2a3a] group-hover:ring-[#ff6b6b]/50 transition-all">
            👤
          </div>
        )}

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className="text-[#f0f0f5] font-semibold text-lg truncate">{displayName}</h3>
          {player.alias && (
            <p className="text-[#a0a0b0] text-sm truncate">{player.name}</p>
          )}
          <div className="flex items-center gap-2 mt-2">
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-[#22c55e]/15 text-[#4ade80] border border-[#22c55e]/30">
              {scoreCount} 成绩
            </span>
          </div>
        </div>

        {/* 操作按钮 */}
        <div className="flex flex-col gap-2">
          <button
            onClick={onUpdateScores}
            disabled={updating}
            className="px-3 py-1.5 rounded-lg text-xs font-medium bg-[#1e1e28] text-[#a0a0b0] hover:bg-[#252533] hover:text-[#f0f0f5] transition-all disabled:opacity-50"
          >
            {updating ? '⏳' : '🔄 更新'}
          </button>
          <button
            onClick={onDelete}
            className="px-3 py-1.5 rounded-lg text-xs font-medium bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20 transition-all"
          >
            🗑️ 删除
          </button>
        </div>
      </div>
    </div>
  );
}