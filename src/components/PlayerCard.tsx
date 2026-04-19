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
    <div className={`bg-[#15151c] rounded-xl p-5 border transition-all duration-300 ${
      player.loading
        ? 'border-[#f59e0b]/50 shadow-xl shadow-[#f59e0b]/20'
        : 'border-[#2a2a3a] shadow-lg hover:border-[#ff6b6b]/50 hover:shadow-xl hover:shadow-[#ff6b6b]/5 group'
    }`}>
      <div className="flex items-start gap-4">
        {/* 头像 */}
        {player.avatar_url ? (
          <img
            src={player.avatar_url}
            alt={displayName}
            className={`w-14 h-14 rounded-xl object-cover ${
              player.loading
                ? 'ring-2 ring-[#f59e0b]/50'
                : 'ring-2 ring-[#2a2a3a] group-hover:ring-[#ff6b6b]/50'
            } transition-all`}
          />
        ) : (
          <div className={`w-14 h-14 rounded-xl flex items-center justify-center text-2xl ${
            player.loading
              ? 'bg-[#f59e0b]/10 ring-2 ring-[#f59e0b]/50'
              : 'bg-[#1e1e28] ring-2 ring-[#2a2a3a] group-hover:ring-[#ff6b6b]/50'
          } transition-all`}>
            👤
          </div>
        )}

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className={`font-semibold text-lg truncate ${
            player.loading ? 'text-[#fbbf24]' : 'text-[#f0f0f5]'
          }`}>{displayName}</h3>
          {player.alias && !player.loading && (
            <p className="text-[#a0a0b0] text-sm truncate">{player.name}</p>
          )}
          <div className="flex items-center gap-2 mt-2">
            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
              player.loading
                ? 'bg-[#f59e0b]/15 text-[#fbbf24] border border-[#f59e0b]/30'
                : 'bg-[#22c55e]/15 text-[#4ade80] border border-[#22c55e]/30'
            }`}>
              {player.loading ? '处理中...' : `${scoreCount} 成绩`}
            </span>
          </div>

          {/* 进度条和状态消息 */}
          {player.loading && (
            <div className="mt-3">
              <div className="w-full bg-[#2a2a3a] rounded-full h-2">
                <div
                  className="bg-[#f59e0b] h-2 rounded-full transition-all duration-300"
                  style={{ width: `${player.progress || 0}%` }}
                ></div>
              </div>
              <p className="text-[#fbbf24] text-xs mt-1 truncate">
                {player.status_message || '正在添加玩家...'}
              </p>
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex flex-col gap-2">
          {!player.loading ? (
            <>
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
            </>
          ) : (
            <button
              className="px-3 py-1.5 rounded-lg text-xs font-medium bg-[#f59e0b]/20 text-[#fbbf24] border border-[#f59e0b]/30 cursor-not-allowed"
              disabled
            >
              ⏳ 处理中
            </button>
          )}
        </div>
      </div>
    </div>
  );
}