/**
 * 难度颜色映射表
 * 与 tailwind.config.js 中的 difficulty 配置保持一致
 */
export const difficultyColors: Record<string, string> = {
  'Easy': 'text-difficulty-easy-text bg-difficulty-easy-bg border-difficulty-easy-border',
  'Normal': 'text-difficulty-normal-text bg-difficulty-normal-bg border-difficulty-normal-border',
  'Hard': 'text-difficulty-hard-text bg-difficulty-hard-bg border-difficulty-hard-border',
  'Expert': 'text-difficulty-expert-text bg-difficulty-expert-bg border-difficulty-expert-border',
  'ExpertPlus': 'text-difficulty-expertplus-text bg-difficulty-expertplus-bg border-difficulty-expertplus-border',
  'Unknown': 'bg-gray-500/15 text-gray-400 border-gray-500/30',
};

/**
 * 获取难度对应的 Tailwind 类名
 * 与 tailwind.config.js 中的 difficulty 配置保持一致
 */
export function getDifficultyClassName(difficulty: string): string {
  const normalized = difficulty.toLowerCase();

  switch (normalized) {
    case 'easy':
      return 'bg-difficulty-easy-bg text-difficulty-easy-text border-difficulty-easy-border';
    case 'normal':
      return 'bg-difficulty-normal-bg text-difficulty-normal-text border-difficulty-normal-border';
    case 'hard':
      return 'bg-difficulty-hard-bg text-difficulty-hard-text border-difficulty-hard-border';
    case 'expert':
      return 'bg-difficulty-expert-bg text-difficulty-expert-text border-difficulty-expert-border';
    case 'expertplus':
    case 'expert+':
      return 'bg-difficulty-expertplus-bg text-difficulty-expertplus-text border-difficulty-expertplus-border';
    default:
      return 'bg-gray-500/15 text-gray-400 border-gray-500/30';
  }
}

/**
 * 获取难度的缩写
 */
export function getDifficultyAbbreviation(difficulty: string): string {
  const normalized = difficulty.toLowerCase();

  switch (normalized) {
    case 'easy':
      return 'E';
    case 'normal':
      return 'N';
    case 'hard':
      return 'H';
    case 'expert':
      return 'Ex';
    case 'expertplus':
    case 'expert+':
      return 'E+';
    default:
      return '?';
  }
}
