import { Playlist } from '../types';

interface PlaylistCardProps {
  playlist: Playlist;
  onDelete: () => void;
  onViewMaps: () => void;
}

export default function PlaylistCard({ playlist, onDelete, onViewMaps }: PlaylistCardProps) {
  return (
    <div className="bg-[#15151c] rounded-xl p-5 border border-[#2a2a3a] shadow-lg transition-all duration-300 hover:border-[#4ecdc4]/50 hover:shadow-xl hover:shadow-[#4ecdc4]/5 group">
      <div className="flex items-start gap-4">
        {/* 封面 */}
        {playlist.playlist_image ? (
          <img
            src={playlist.playlist_image}
            alt={playlist.name}
            className="w-14 h-14 rounded-xl object-cover ring-2 ring-[#2a2a3a] group-hover:ring-[#4ecdc4]/50 transition-all"
          />
        ) : (
          <div className="w-14 h-14 rounded-xl bg-[#1e1e28] flex items-center justify-center text-2xl ring-2 ring-[#2a2a3a] group-hover:ring-[#4ecdc4]/50 transition-all">
            📋
          </div>
        )}

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className="text-[#f0f0f5] font-semibold text-lg truncate">{playlist.name}</h3>
          {playlist.owner_name && (
            <p className="text-[#a0a0b0] text-sm truncate">by {playlist.owner_name}</p>
          )}
          <div className="flex items-center gap-2 mt-2">
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-[#3b82f6]/15 text-[#60a5fa] border border-[#3b82f6]/30">
              {playlist.total_maps || 0} 谱面
            </span>
          </div>
        </div>

        {/* 操作按钮 */}
        <div className="flex flex-col gap-2">
          <button
            onClick={onViewMaps}
            className="px-3 py-1.5 rounded-lg text-xs font-medium bg-[#1e1e28] text-[#a0a0b0] hover:bg-[#252533] hover:text-[#f0f0f5] transition-all"
          >
            👁️ 查看
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