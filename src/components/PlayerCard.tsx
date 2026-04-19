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
    <div className="bg-bs-dark-800 rounded-lg p-4 border border-bs-dark-600 hover:border-bs-accent-primary/50 transition-colors">
      <div className="flex items-start gap-4">
        {player.avatar_url ? (
          <img
            src={player.avatar_url}
            alt={displayName}
            className="w-12 h-12 rounded-full"
          />
        ) : (
          <div className="w-12 h-12 rounded-full bg-bs-dark-700 flex items-center justify-center text-gray-400">
            👤
          </div>
        )}

        <div className="flex-1 min-w-0">
          <h3 className="text-gray-200 font-medium truncate">{displayName}</h3>
          {player.alias && (
            <p className="text-gray-400 text-sm truncate">{player.name}</p>
          )}
          <p className="text-bs-accent-neon text-sm">{scoreCount} 成绩</p>
        </div>

        <div className="flex gap-2">
          <button
            onClick={onUpdateScores}
            disabled={updating}
            className="px-3 py-1.5 text-sm bg-bs-dark-700 text-gray-300 rounded hover:bg-bs-dark-600 disabled:opacity-50"
          >
            {updating ? '更新中...' : '更新'}
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