import { Map, modeNames } from '../types';
import { formatRating } from '../utils/format';

interface MapCardProps {
  map: Map;
  diffs?: { mode: string; difficulty: string }[];
}

export default function MapCard({ map, diffs }: MapCardProps) {
  return (
    <div className="bg-[#15151c] rounded-xl p-5 border border-[#2a2a3a] shadow-lg">
      <div className="flex items-start gap-5">
        {/* 封面 */}
        {map.cover_url ? (
          <img
            src={map.cover_url}
            alt={map.song_name}
            className="w-28 h-28 rounded-xl object-cover ring-2 ring-[#2a2a3a]"
          />
        ) : (
          <div className="w-28 h-28 rounded-xl bg-[#1e1e28] flex items-center justify-center text-4xl ring-2 ring-[#2a2a3a]">
            🎵
          </div>
        )}

        {/* 信息 */}
        <div className="flex-1 min-w-0">
          <h3 className="text-[#f0f0f5] font-bold text-xl truncate">{map.song_name}</h3>
          <p className="text-[#a0a0b0] text-sm truncate mt-1">{map.song_author_name}</p>
          <p className="text-[#6b7280] text-sm truncate">谱师: {map.level_author_name}</p>
          <div className="flex items-center gap-3 mt-3">
            <span className="text-[#4ecdc4] font-medium">
              ⭐ {formatRating(map.rating)}
            </span>
            <span className="text-[#6b7280] text-xs">
              Key: {map.key}
            </span>
          </div>
        </div>
      </div>

      {/* 难度列表 */}
      {diffs && diffs.length > 0 && (
        <div className="mt-5 pt-5 border-t border-[#2a2a3a]">
          <h4 className="text-[#6b7280] text-sm font-medium mb-3">难度列表</h4>
          <div className="flex flex-wrap gap-2">
            {diffs.map((diff, idx) => {
              const difficultyClass = diff.difficulty === 'Easy'
                ? 'bg-[#22c55e]/15 text-[#4ade80] border-[#22c55e]/30'
                : diff.difficulty === 'Normal'
                ? 'bg-[#3b82f6]/15 text-[#60a5fa] border-[#3b82f6]/30'
                : diff.difficulty === 'Hard'
                ? 'bg-[#f59e0b]/15 text-[#fbbf24] border-[#f59e0b]/30'
                : diff.difficulty === 'Expert'
                ? 'bg-[#ef4444]/15 text-[#f87171] border-[#ef4444]/30'
                : 'bg-[#9b59b6]/15 text-[#c084fc] border-[#9b59b6]/30';

              return (
                <span
                  key={idx}
                  className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${difficultyClass}`}
                >
                  {modeNames[diff.mode] || diff.mode} · {diff.difficulty}
                </span>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}