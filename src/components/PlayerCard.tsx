import { PlayerInfo, PlayerUIState } from '../types';
import { RefreshCw, Trash2, Clock, User, Star } from 'lucide-react';

interface PlayerCardProps {
  player: PlayerInfo;
  uiState?: PlayerUIState;
  scoreCount: number;
  onUpdateScores: () => void;
  onDelete: () => void;
  updating?: boolean;
  isStarred?: boolean;
  onToggleStar?: () => void;
}

export default function PlayerCard({ player, uiState, scoreCount, onUpdateScores, onDelete, updating, isStarred, onToggleStar }: PlayerCardProps) {
  const displayName = player.alias || player.name;
  const isLoading = uiState?.loading ?? false;

  return (
    <div className={`bg-bs-bg-secondary rounded-xl p-5 border transition-all duration-300 ${
      isLoading
        ? 'border-bs-status-warning/50 shadow-xl shadow-bs-status-warning/20'
        : 'border-bs-border shadow-lg hover:border-bs-accent-primary/50 hover:shadow-xl hover:shadow-bs-accent-primary/5 group'
    }`}>
      <div className="flex items-start gap-4">
        {/* 头像 */}
        <div className="relative">
          {player.avatar_url ? (
            <img
              src={player.avatar_url}
              alt={displayName}
              className={`w-14 h-14 rounded-xl object-cover cursor-pointer ${
                isStarred
                  ? 'ring-2 ring-yellow-400'
                  : isLoading
                    ? 'ring-2 ring-bs-status-warning/50'
                    : 'ring-2 ring-bs-border group-hover:ring-bs-accent-primary/50'
              } transition-all`}
              onClick={onToggleStar}
            />
          ) : (
            <div
              className={`w-14 h-14 rounded-xl flex items-center justify-center text-2xl cursor-pointer ${
                isStarred
                  ? 'bg-bs-bg-tertiary ring-2 ring-yellow-400'
                  : isLoading
                    ? 'bg-bs-status-warning/10 ring-2 ring-bs-status-warning/50'
                    : 'bg-bs-bg-tertiary ring-2 ring-bs-border group-hover:ring-bs-accent-primary/50'
              } transition-all`}
              onClick={onToggleStar}
            >
              <User size={28} className="text-bs-text-muted" />
            </div>
          )}
          {/* 标星 hover 覆盖层 */}
          {onToggleStar && (
            <div
              className="absolute inset-0 rounded-xl bg-black/40 flex items-center justify-center opacity-0 hover:opacity-100 cursor-pointer transition-opacity"
              onClick={onToggleStar}
            >
              {isStarred ? (
                <Star size={24} className="text-yellow-400 fill-yellow-400" />
              ) : (
                <Star size={24} className="text-white" />
              )}
            </div>
          )}
        </div>

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className={`font-semibold text-lg truncate ${
            isLoading ? 'text-bs-status-warning' : 'text-bs-text-primary'
          }`}>{displayName}</h3>
          {player.alias && !isLoading && (
            <p className="text-bs-text-secondary text-sm truncate">{player.name}</p>
          )}
          <div className="flex items-center gap-2 mt-2">
            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
              isLoading
                ? 'bg-bs-status-warning/15 text-bs-status-warning border border-bs-status-warning/30'
                : 'bg-bs-status-success/15 text-bs-status-success border border-bs-status-success/30'
            }`}>
              {isLoading ? '处理中...' : `${scoreCount} 成绩`}
            </span>
            {uiState?.updatingMessage && !isLoading && (
              <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-bs-status-success/15 text-bs-status-success border border-bs-status-success/30">
                {uiState.updatingMessage}
              </span>
            )}
          </div>

          {/* 进度条和状态消息 */}
          {isLoading && (
            <div className="mt-3">
              <div className="w-full bg-bs-border rounded-full h-2">
                <div
                  className="bg-bs-status-warning h-2 rounded-full transition-all duration-300"
                  style={{ width: `${uiState?.progress ?? 0}%` }}
                ></div>
              </div>
              <p className="text-bs-status-warning text-xs mt-1 truncate">
                {uiState?.statusMessage || '正在添加玩家...'}
              </p>
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex flex-col gap-2">
          {!isLoading ? (
            <>
              <button
                onClick={onUpdateScores}
                disabled={updating}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all disabled:opacity-50 flex items-center gap-1"
              >
                {updating ? <Clock size={14} /> : <RefreshCw size={14} />}
                {updating ? '更新中' : '更新'}
              </button>
              <button
                onClick={onDelete}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20 transition-all flex items-center gap-1"
              >
                <Trash2 size={14} />
                删除
              </button>
            </>
          ) : (
            <button
              className="px-3 py-1.5 rounded-lg text-xs font-medium bg-bs-status-warning/20 text-bs-status-warning border border-bs-status-warning/30 cursor-not-allowed flex items-center gap-1"
              disabled
            >
              <Clock size={14} />
              处理中
            </button>
          )}
        </div>
      </div>
    </div>
  );
}