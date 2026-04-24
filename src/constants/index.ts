// 难度颜色映射（基于 tailwind.config.js 的 difficulty 配置）
export const difficultyColors: Record<string, string> = {
  'Easy': 'text-difficulty-easy-text bg-difficulty-easy-bg border-difficulty-easy-border',
  'Normal': 'text-difficulty-normal-text bg-difficulty-normal-bg border-difficulty-normal-border',
  'Hard': 'text-difficulty-hard-text bg-difficulty-hard-bg border-difficulty-hard-border',
  'Expert': 'text-difficulty-expert-text bg-difficulty-expert-bg border-difficulty-expert-border',
  'ExpertPlus': 'text-difficulty-expertplus-text bg-difficulty-expertplus-bg border-difficulty-expertplus-border',
};

// 模式名称映射
export const modeNames: Record<string, string> = {
  'Standard': '标准',
  'OneSaber': '单剑',
  'NoArrows': '无箭头',
  '90Degree': '90度',
  '360Degree': '360度',
  'Lightshow': '光秀',
  'Lawless': '无限制',
  'Legacy': '旧版',
};
