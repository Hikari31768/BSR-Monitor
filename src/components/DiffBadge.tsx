interface DiffBadgeProps {
  difficulty: string;
  showFull?: boolean;
}

const difficultyConfig: Record<string, { abbr: string; color: string; bgColor: string }> = {
  Easy: { abbr: 'E', color: '#347e55', bgColor: 'rgba(52, 126, 85, 0.2)' },
  Normal: { abbr: 'N', color: '#2f66a0', bgColor: 'rgba(47, 102, 160, 0.2)' },
  Hard: { abbr: 'H', color: '#b15c11', bgColor: 'rgba(177, 92, 17, 0.2)' },
  Expert: { abbr: 'Ex', color: '#a83922', bgColor: 'rgba(168, 57, 34, 0.2)' },
  ExpertPlus: { abbr: 'E+', color: '#7049ae', bgColor: 'rgba(112, 73, 174, 0.2)' },
};

export default function DiffBadge({ difficulty, showFull = false }: DiffBadgeProps) {
  const config = difficultyConfig[difficulty] || { abbr: '?', color: '#888', bgColor: 'rgba(136, 136, 136, 0.2)' };

  return (
    <span
      className="inline-flex items-center justify-center px-2 py-0.5 rounded text-xs font-semibold"
      style={{
        color: config.color,
        backgroundColor: config.bgColor,
        border: `1px solid ${config.color}40`,
      }}
    >
      {showFull ? difficulty : config.abbr}
    </span>
  );
}
