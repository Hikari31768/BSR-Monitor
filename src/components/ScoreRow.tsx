import { PlayerScore } from '../types';
import { formatEpochTime, formatAccuracy } from '../utils/format';
import { difficultyColors, modeNames } from '../types';

interface ScoreRowProps {
  score: PlayerScore;
}

export default function ScoreRow({ score }: ScoreRowProps) {
  return (
    <div className="bg-bs-dark-800 rounded-lg p-3 border border-bs-dark-600 flex items-center justify-between">
      <div className="flex items-center gap-3">
        <span
          className={`px-2 py-1 rounded text-xs ${
            difficultyColors[score.difficulty] || 'bg-bs-dark-700 text-gray-300'
          }`}
        >
          {score.difficulty}
        </span>
        <span className="text-gray-400 text-xs">
          {modeNames[score.mode] || score.mode}
        </span>
      </div>

      <div className="flex items-center gap-4">
        <span className="text-bs-accent-neon font-medium">
          {formatAccuracy(score.accuracy)}
        </span>
        <span className="text-gray-500 text-sm">
          {formatEpochTime(score.epoch_time)}
        </span>
      </div>
    </div>
  );
}