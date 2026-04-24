import { NavLink } from 'react-router-dom';
import { useAudio } from '../contexts/AudioContext';
import { Target, User, List } from 'lucide-react';

const navItems = [
  { path: '/', label: '打谱查询', icon: Target },
  { path: '/players', label: '玩家管理', icon: User },
  { path: '/playlists', label: '歌单管理', icon: List },
];

export default function Sidebar() {
  const { volume, setVolume } = useAudio();

  return (
    <aside className="w-64 bg-bs-bg-secondary border-r border-bs-border flex flex-col shrink-0">
      {/* Logo 区域 */}
      <div className="p-6 border-b border-bs-border">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-bs-accent-primary to-bs-accent-purple flex items-center justify-center text-white text-lg font-bold shadow-lg">
            BS
          </div>
          <div>
            <h1 className="text-lg font-bold text-bs-text-primary tracking-tight">BSR Monitor</h1>
            <p className="text-xs text-bs-text-muted">BeatSaber 成绩监控</p>
          </div>
        </div>
      </div>

      {/* 导航菜单 */}
      <nav className="flex-1 p-4 space-y-1">
        {navItems.map(item => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) =>
              `flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 ${
                isActive
                  ? 'bg-bs-accent-primary/10 text-bs-accent-primary border border-bs-accent-primary/20'
                  : 'text-bs-text-secondary hover:bg-bs-bg-tertiary hover:text-bs-text-primary'
              }`
            }
          >
            <item.icon size={20} />
            <span className="font-medium">{item.label}</span>
          </NavLink>
        ))}
      </nav>

      {/* 音量控制 */}
      <div className="p-4 border-t border-bs-border space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs text-bs-text-muted">音量</span>
          <span className="text-xs text-bs-text-muted">{Math.round(volume * 100)}%</span>
        </div>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={volume}
          onChange={e => setVolume(parseFloat(e.target.value))}
          className="w-full h-1.5 rounded-full appearance-none cursor-pointer bg-bs-border [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-bs-accent-primary"
          style={{
            background: `linear-gradient(to right, var(--color-accent-primary) ${volume * 100}%, var(--color-border) ${volume * 100}%)`,
          }}
        />
      </div>

      {/* 底部版本 */}
      <div className="px-4 pb-3">
        <p className="text-xs text-bs-text-muted">v0.1.0</p>
      </div>
    </aside>
  );
}