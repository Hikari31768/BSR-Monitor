import { Playlist, PlaylistUIState } from '../types';
import { Clock, Eye, Trash2, List, RefreshCw } from 'lucide-react';

interface PlaylistCardProps {
  playlist: Playlist;
  uiState?: PlaylistUIState;
  onDelete: () => void;
  onViewMaps: () => void;
  onUpdate?: () => void;
  updating?: boolean;
}

export default function PlaylistCard({ playlist, uiState, onDelete, onViewMaps, onUpdate, updating }: PlaylistCardProps) {
  const isLoading = uiState?.loading ?? false;

  return (
    <div className={`bg-bs-bg-secondary rounded-xl p-5 border transition-all duration-300 ${
      isLoading
        ? 'border-bs-status-warning/50 shadow-xl shadow-bs-status-warning/20'
        : 'border-bs-border shadow-lg hover:border-bs-accent-secondary/50 hover:shadow-xl hover:shadow-bs-accent-secondary/5 group'
    }`}>
      <div className="flex items-start gap-4">
        {/* 封面 */}
        {playlist.playlist_image ? (
          <img
            src={playlist.playlist_image}
            alt={playlist.name}
            className={`w-14 h-14 rounded-xl object-cover ${
              isLoading
                ? 'ring-2 ring-bs-status-warning/50'
                : 'ring-2 ring-bs-border group-hover:ring-bs-accent-secondary/50'
            } transition-all`}
          />
        ) : (
          <div className={`w-14 h-14 rounded-xl flex items-center justify-center text-2xl ${
            isLoading
              ? 'bg-bs-status-warning/10 ring-2 ring-bs-status-warning/50'
              : 'bg-bs-bg-tertiary ring-2 ring-bs-border group-hover:ring-bs-accent-secondary/50'
          } transition-all`}>
            <List size={28} className="text-bs-text-muted" />
          </div>
        )}

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className={`font-semibold text-lg truncate ${
            isLoading ? 'text-bs-status-warning' : 'text-bs-text-primary'
          }`}>{playlist.name}</h3>
          {playlist.owner_name && !isLoading && (
            <p className="text-bs-text-secondary text-sm truncate">by {playlist.owner_name}</p>
          )}
          <div className="flex items-center gap-2 mt-2">
            <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
              isLoading
                ? 'bg-bs-status-warning/15 text-bs-status-warning border border-bs-status-warning/30'
                : 'bg-bs-status-info/15 text-bs-status-info border border-bs-status-info/30'
            }`}>
              {isLoading ? '处理中...' : `${playlist.total_maps || 0} 谱面`}
            </span>
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
                {uiState?.statusMessage || '正在添加歌单...'}
              </p>
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex flex-col gap-2">
          {!isLoading ? (
            <>
              {onUpdate && (
                <button
                  onClick={onUpdate}
                  disabled={updating}
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all disabled:opacity-50 flex items-center gap-1"
                >
                  {updating ? <Clock size={14} /> : <RefreshCw size={14} />}
                  {updating ? '更新中' : '更新'}
                </button>
              )}
              <button
                onClick={onViewMaps}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all flex items-center gap-1"
              >
                <Eye size={14} />
                查看
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