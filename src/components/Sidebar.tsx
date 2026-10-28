import { useState, useEffect } from 'react';
import { NavLink } from 'react-router-dom';
import { useAudio } from '../contexts/AudioContext';
import { invoke } from '@tauri-apps/api/core';
import { Target, User, List, FolderOpen, Settings } from 'lucide-react';

const navItems = [
  { path: '/', label: '打谱查询', icon: Target },
  { path: '/players', label: '玩家管理', icon: User },
  { path: '/playlists', label: '歌单管理', icon: List },
];

export default function Sidebar() {
  const { volume, setVolume } = useAudio();
  const [version, setVersion] = useState('0.0.0');

  useEffect(() => {
    invoke<string>('get_version')
      .then(setVersion)
      .catch(() => {});
  }, []);

  return (
    <aside className="w-52 bg-bs-bg-secondary border-r border-bs-border flex flex-col shrink-0 overscroll-contain">
      {/* Logo 区域 */}
      <div className="p-6 border-b border-bs-border">
        <div className="text-xl font-bold text-bs-text-primary tracking-tight text-center">!bsr 👀</div>
      </div>

      {/* 导航菜单 */}
      <nav className="flex-1 p-4 space-y-1">
        {navItems.map(item => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) =>
              `flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 justify-center ${
                isActive
                  ? 'bg-bs-accent-primary/10 text-bs-accent-primary border border-bs-accent-primary/20'
                  : 'text-bs-text-secondary hover:bg-bs-bg-tertiary hover:text-bs-text-primary'
              }`
            }
          >
            <item.icon size={20} aria-hidden="true" />
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
          aria-label="音量控制"
          className="w-full h-1.5 rounded-full appearance-none cursor-pointer bg-bs-border [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-bs-accent-primary"
          style={{
            background: `linear-gradient(to right, var(--color-accent-primary) ${volume * 100}%, var(--color-border) ${volume * 100}%)`,
          }}
        />
      </div>

      {/* 底部版本 + 操作按钮 */}
      <div className="px-4 pb-3 flex items-center gap-2">
        <button
          onClick={() => invoke('open_url', { url: 'https://github.com/YoLeax/BSR-Monitor' })}
          className="text-xs text-bs-text-muted shrink-0 hover:text-bs-text-primary transition-colors cursor-pointer"
          title="在 GitHub 上查看项目"
        >
          v{version}
        </button>
        <div className="flex-1" />
        <button
          onClick={() => invoke('open_data_dir')}
          className="p-1.5 rounded-lg text-bs-text-muted hover:text-bs-text-primary hover:bg-bs-bg-tertiary transition-colors"
          title="打开数据目录"
        >
          <FolderOpen size={16} />
        </button>
        <NavLink
          to="/settings"
          className={({ isActive }) => `p-1.5 rounded-lg transition-colors ${isActive ? 'text-bs-accent-primary bg-bs-accent-primary/10' : 'text-bs-text-muted hover:text-bs-text-primary hover:bg-bs-bg-tertiary'}`}
          title="设置"
          aria-label="设置"
        >
          <Settings size={16} />
        </NavLink>
      </div>
    </aside>
  );
}
