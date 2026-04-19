import { Map, difficultyColors, modeNames } from '../types';
import { formatRating } from '../utils/format';

interface MapCardProps {
  map: Map;
  diffs?: { mode: string; difficulty: string }[];
}

export default function MapCard({ map, diffs }: MapCardProps) {
  return (
    <div className="bg-bs-dark-800 rounded-lg p-4 border border-bs-dark-600">
      <div className="flex items-start gap-4">
        {map.cover_url ? (
          <img
            src={map.cover_url}
            alt={map.song_name}
            className="w-24 h-24 rounded object-cover"
          />
        ) : (
          <div className="w-24 h-24 rounded bg-bs-dark-700 flex items-center justify-center text-gray-400 text-3xl">
            🎵
          </div>
        )}

        <div className="flex-1 min-w-0">
          <h3 className="text-gray-200 font-medium truncate text-lg">{map.song_name}</h3>
          <p className="text-gray-400 text-sm truncate">{map.song_author_name}</p>
          <p className="text-gray-500 text-sm truncate">谱师: {map.level_author_name}</p>
          <p className="text-bs-accent-neon text-sm mt-1">
            评分: {formatRating(map.rating)}
          </p>
        </div>

        <div className="text-xs text-gray-500">Key: {map.key}</div>
      </div>

      {diffs && diffs.length > 0 && (
        <div className="mt-4 pt-4 border-t border-bs-dark-600">
          <h4 className="text-sm text-gray-400 mb-2">难度列表</h4>
          <div className="flex flex-wrap gap-2">
            {diffs.map((diff, idx) => (
              <span
                key={idx}
                className={`px-2 py-1 rounded text-xs ${
                  difficultyColors[diff.difficulty] || 'bg-bs-dark-700 text-gray-300'
                }`}
              >
                {modeNames[diff.mode] || diff.mode} - {diff.difficulty}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}