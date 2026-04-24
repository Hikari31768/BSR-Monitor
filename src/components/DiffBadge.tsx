import { difficultyColors, getDifficultyAbbreviation } from '../utils/difficulty';

interface DiffBadgeProps {
  difficulty: string;
  showFull?: boolean;
  size?: 'normal' | 'sm';
}

export default function DiffBadge({ difficulty, showFull = false, size = 'normal' }: DiffBadgeProps) {
  const sizeClasses = size === 'sm'
    ? 'px-1 py-0 text-[10px]'
    : 'px-2 py-0.5 text-xs';

  return (
    <span
      title={difficulty}
      aria-label={difficulty}
      className={`inline-flex items-center justify-center rounded font-semibold border ${sizeClasses} ${difficultyColors[difficulty] || difficultyColors['Unknown']}`}
    >
      {showFull ? difficulty : getDifficultyAbbreviation(difficulty)}
    </span>
  );
}
