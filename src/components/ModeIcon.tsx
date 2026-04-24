import { modeNames } from '../constants';

interface ModeIconProps {
  mode: keyof typeof modeIcons;
  size?: number;
}

const modeIcons: Record<string, string> = {
  Standard: '/icons/standard.svg',
  OneSaber: '/icons/onesaber.svg',
  NoArrows: '/icons/noarrows.svg',
  '90Degree': '/icons/90degree.svg',
  '360Degree': '/icons/360degree.svg',
  Lightshow: '/icons/lightshow.svg',
  Lawless: '/icons/lawless.svg',
  Legacy: '/icons/legacy.svg',
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
            e.currentTarget.style.display = 'none';
            const fallback = e.currentTarget.nextElementSibling;
            if (fallback) (fallback as HTMLElement).style.display = '';
          }}
        />
      ) : null}
      <span className="text-xs text-bs-text-secondary" style={{ display: iconPath ? 'none' : '' }}>
        {displayName}
      </span>
    </div>
  );
}
