import { difficultyColors, difficultyBorderColors, getDifficultyAbbreviation } from '../utils/difficulty';

interface DiffBadgeProps {
  difficulty: string;
  showFull?: boolean;
  size?: 'normal' | 'sm';
  stars?: number;
}

export default function DiffBadge({ difficulty, showFull = false, size = 'normal', stars }: DiffBadgeProps) {
  const sizeClasses = size === 'sm'
    ? 'px-1 py-0 text-[10px]'
    : 'px-2 py-0.5 text-xs';

  const borderColor = difficultyBorderColors[difficulty];

  return (
    <span
      title={difficulty}
      aria-label={difficulty}
      className={`inline-flex items-center justify-center rounded font-semibold border ${sizeClasses} ${difficultyColors[difficulty] || difficultyColors['Unknown']}`}
      style={borderColor ? { borderColor } : undefined}
    >
      {showFull ? difficulty : getDifficultyAbbreviation(difficulty)}
      {stars !== undefined && stars !== null && (
        <span className="ml-1 font-bold">{stars.toFixed(1)}</span>
      )}
    </span>
  );
}
