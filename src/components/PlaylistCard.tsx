import { Playlist, PlaylistUIState } from '../types';
import { Clock, Trash2, List, RefreshCw, Star } from 'lucide-react';

interface PlaylistCardProps {
  playlist: Playlist;
  uiState?: PlaylistUIState;
  onDelete: () => void;
  onViewMaps: () => void;
  onUpdate?: () => void;
  onRefreshRatings?: () => void;
  updating?: boolean;
  refreshingRatings?: boolean;
}

const SOURCE_LABELS: Record<string, string> = {
  beatsaver: 'Saver',
  mapper: '谱师',
  local: '本地',
};

const SOURCE_COLORS: Record<string, string> = {
  beatsaver: 'bg-blue-500/15 text-blue-400 border-blue-500/30',
  mapper: 'bg-purple-500/15 text-purple-400 border-purple-500/30',
  local: 'bg-green-500/15 text-green-400 border-green-500/30',
};

export default function PlaylistCard({ playlist, uiState, onDelete, onViewMaps: _onViewMaps, onUpdate, onRefreshRatings, updating, refreshingRatings }: PlaylistCardProps) {
  const isLoading = uiState?.loading ?? false;
  const sourceKey = playlist.source;
  const sourceLabel = SOURCE_LABELS[sourceKey] ?? sourceKey;
  const sourceColor = SOURCE_COLORS[sourceKey] ?? SOURCE_COLORS.beatsaver;
  // 处理不同格式的 image 字段：
  // - 直接 base64 数据: "iVBORw0KGgo..."
  // - 带 base64, 前缀: "base64,/9j/4AAQSkZJR..."
  const getImageData = () => {
    if (!playlist.playlist_image) return null;
    let data = playlist.playlist_image;
    if (data.startsWith('base64,')) {
      data = data.substring(7);
    }
    return `data:image/png;base64,${data}`;
  };
  const imageSrc = getImageData();

  const formatUpdateTime = (ts?: number) => {
    if (!ts) return null;
    const d = new Date(ts * 1000);
    return d.toLocaleDateString('zh-CN', {
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit',
    });
  };
  const updateTimeText = formatUpdateTime(playlist.updated_at);

  return (
    <div className={`bg-bs-bg-secondary rounded-xl p-5 border transition-all duration-300 ${
      isLoading
        ? 'border-bs-status-warning/50 shadow-xl shadow-bs-status-warning/20'
        : 'border-bs-border shadow-lg hover:border-bs-accent-secondary/50 hover:shadow-xl hover:shadow-bs-accent-secondary/5 group'
    }`}>
      <div className="flex items-start gap-4">
        {/* 封面 + 谱面性质 */}
        <div className="flex flex-col items-center">
          {imageSrc ? (
            <img
              src={imageSrc}
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
          <span className={`mt-1.5 inline-flex items-center justify-center w-14 py-0.5 rounded text-xs font-medium whitespace-nowrap ${sourceColor}`}>
            {sourceLabel}
          </span>
        </div>

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className={`font-semibold text-lg truncate ${
            isLoading ? 'text-bs-status-warning' : 'text-bs-text-primary'
          }`}>{playlist.name}</h3>
          {playlist.owner_name && !isLoading && (
            <p className="text-bs-text-secondary text-sm truncate">by {playlist.owner_name}</p>
          )}
          {updateTimeText && !isLoading && (
            <p className="text-bs-text-muted text-xs flex items-center gap-1 mt-1">
              <Clock size={12} />
              {updateTimeText}
            </p>
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

          {/* 状态消息 */}
          {isLoading && (
            <p className="text-bs-status-warning text-xs mt-2 truncate">
              {uiState?.statusMessage || '正在添加歌单...'}
            </p>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex flex-col gap-2 w-fit">
          {!isLoading ? (
            <>
              {onUpdate && (
                <button
                  onClick={onUpdate}
                  disabled={updating}
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all disabled:opacity-50 flex items-center gap-1 justify-center"
                >
                  {updating ? <Clock size={14} /> : <RefreshCw size={14} />}
                  {updating ? '更新中' : '更新歌单'}
                </button>
              )}
              {onRefreshRatings && (
                <button
                  onClick={onRefreshRatings}
                  disabled={refreshingRatings || updating}
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-bs-bg-tertiary text-bs-text-secondary hover:bg-bs-bg-hover hover:text-bs-text-primary transition-all disabled:opacity-50 flex items-center gap-1 justify-center"
                >
                  {refreshingRatings ? <Clock size={14} /> : <Star size={14} />}
                  {refreshingRatings ? '更新中' : '更新星级'}
                </button>
              )}
              <button
                onClick={onDelete}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20 transition-all flex items-center gap-1 justify-center"
              >
                <Trash2 size={14} />
                删除
              </button>
            </>
          ) : (
            <button
              className="px-3 py-1.5 rounded-lg text-xs font-medium bg-bs-status-warning/20 text-bs-status-warning border border-bs-status-warning/30 cursor-not-allowed flex items-center gap-1 justify-center"
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
