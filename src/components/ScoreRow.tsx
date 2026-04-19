import { modeNames } from '../types';
import { formatEpochTime, formatAccuracy } from '../utils/format';

interface ScoreRowProps {
  score: {
    difficulty: string;
    mode: string;
    accuracy: number;
    epoch_time: number;
  };
}

export default function ScoreRow({ score }: ScoreRowProps) {
  const difficultyClass = score.difficulty === 'Easy'
    ? 'bg-[#22c55e]/15 text-[#4ade80] border-[#22c55e]/30'
    : score.difficulty === 'Normal'
    ? 'bg-[#3b82f6]/15 text-[#60a5fa] border-[#3b82f6]/30'
    : score.difficulty === 'Hard'
    ? 'bg-[#f59e0b]/15 text-[#fbbf24] border-[#f59e0b]/30'
    : score.difficulty === 'Expert'
    ? 'bg-[#ef4444]/15 text-[#f87171] border-[#ef4444]/30'
    : 'bg-[#9b59b6]/15 text-[#c084fc] border-[#9b59b6]/30';

  return (
    <div className="bg-[#15151c] rounded-xl p-4 border border-[#2a2a3a] flex items-center justify-between animate-fade-in">
      <div className="flex items-center gap-3">
        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${difficultyClass}`}>
          {score.difficulty}
        </span>
        <span className="text-[#6b7280] text-sm">
          {modeNames[score.mode] || score.mode}
        </span>
      </div>

      <div className="flex items-center gap-6">
        <span className="text-[#4ecdc4] font-semibold text-lg">
          {formatAccuracy(score.accuracy)}
        </span>
        <span className="text-[#6b7280] text-sm">
          {formatEpochTime(score.epoch_time)}
        </span>
      </div>
    </div>
  );
}