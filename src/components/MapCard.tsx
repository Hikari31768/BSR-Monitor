import { Map, modeNames } from '../types';
import { formatRating } from '../utils/format';
import { getDifficultyClassName } from '../utils/difficulty';

interface MapCardProps {
  map: Map;
  diffs?: { mode: string; difficulty: string }[];
}

export default function MapCard({ map, diffs }: MapCardProps) {
  return (
    <div className="bg-bs-bg-secondary rounded-xl p-5 border border-bs-border shadow-lg">
      <div className="flex items-start gap-5">
        {/* 封面 */}
        {map.cover_url ? (
          <img
            src={map.cover_url}
            alt={map.song_name}
            className="w-28 h-28 rounded-xl object-cover ring-2 ring-bs-border"
          />
        ) : (
          <div className="w-28 h-28 rounded-xl bg-bs-bg-tertiary flex items-center justify-center text-4xl ring-2 ring-bs-border">
            🎵
          </div>
        )}

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className="text-bs-text-primary font-bold text-xl truncate">{map.song_name}</h3>
          <p className="text-bs-text-secondary text-sm truncate mt-1">{map.song_author_name}</p>
          <p className="text-bs-text-muted text-sm truncate">谱师: {map.level_author_name}</p>
          <div className="flex items-center gap-3 mt-3">
            <span className="text-bs-accent-secondary font-medium">
              👍 {formatRating(map.rating)}
            </span>
            <span className="text-bs-text-muted text-xs">
              Key: {map.key}
            </span>
          </div>
        </div>
      </div>

      {/* 难度列表 */}
      {diffs && diffs.length > 0 && (
        <div className="mt-5 pt-5 border-t border-bs-border">
          <h4 className="text-bs-text-muted text-sm font-medium mb-3">难度列表</h4>
          <div className="flex flex-wrap gap-2">
            {diffs.map((diff, idx) => (
              <span
                key={idx}
                className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getDifficultyClassName(diff.difficulty)}`}
              >
                {modeNames[diff.mode] || diff.mode} · {diff.difficulty}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}