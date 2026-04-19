interface ModeIconProps {
  mode: string;
  size?: number;
}

const modeIcons: Record<string, string> = {
  Standard: 'icons/standard.svg',
  OneSaber: 'icons/onesaber.svg',
  NoArrows: 'icons/noarrows.svg',
  '90Degree': 'icons/90degree.svg',
  '360Degree': 'icons/360degree.svg',
  Lightshow: 'icons/lightshow.svg',
  Lawless: 'icons/lawless.svg',
  Legacy: 'icons/legacy.svg',
};

const modeNames: Record<string, string> = {
  Standard: '标准',
  OneSaber: '单剑',
  NoArrows: '无箭头',
  '90Degree': '90度',
  '360Degree': '360度',
  Lightshow: '光秀',
  Lawless: '无限制',
  Legacy: '旧版',
};

export default function ModeIcon({ mode, size = 20 }: ModeIconProps) {
  const iconPath = modeIcons[mode];
  const displayName = modeNames[mode] || mode;

  return (
    <div className="flex items-center gap-1.5" title={displayName}>
      {iconPath ? (
        <img
          src={iconPath}
          alt={displayName}
          width={size}
          height={size}
          className="object-contain"
          onError={(e) => {
            // 如果图标加载失败，显示文字缩写
            e.currentTarget.style.display = 'none';
          }}
        />
      ) : (
        <span className="text-xs text-[#a0a0b0]">{mode.slice(0, 2)}</span>
      )}
    </div>
  );
}
