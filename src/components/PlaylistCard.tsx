import { Playlist } from '../types';

interface PlaylistCardProps {
  playlist: Playlist;
  onDelete: () => void;
  onViewMaps: () => void;
}

export default function PlaylistCard({ playlist, onDelete, onViewMaps }: PlaylistCardProps) {
  return (
    <div className="bg-bs-dark-800 rounded-lg p-4 border border-bs-dark-600 hover:border-bs-accent-primary/50 transition-colors">
      <div className="flex items-start gap-4">
        {playlist.playlist_image ? (
          <img
            src={playlist.playlist_image}
            alt={playlist.name}
            className="w-12 h-12 rounded object-cover"
          />
        ) : (
          <div className="w-12 h-12 rounded bg-bs-dark-700 flex items-center justify-center text-gray-400">
            📋
          </div>
        )}

        <div className="flex-1 min-w-0">
          <h3 className="text-gray-200 font-medium truncate">{playlist.name}</h3>
          {playlist.owner_name && (
            <p className="text-gray-400 text-sm truncate">by {playlist.owner_name}</p>
          )}
          <p className="text-bs-accent-neon text-sm">{playlist.total_maps || 0} 谱面</p>
        </div>

        <div className="flex gap-2">
          <button
            onClick={onViewMaps}
            className="px-3 py-1.5 text-sm bg-bs-dark-700 text-gray-300 rounded hover:bg-bs-dark-600"
          >
            查看
          </button>
          <button
            onClick={onDelete}
            className="px-3 py-1.5 text-sm bg-red-500/20 text-red-400 rounded hover:bg-red-500/30"
          >
            删除
          </button>
        </div>
      </div>
    </div>
  );
}