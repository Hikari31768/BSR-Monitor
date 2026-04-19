import { NavLink } from 'react-router-dom';

const navItems = [
  { path: '/players', label: '玩家管理', icon: '👤' },
  { path: '/playlists', label: '歌单管理', icon: '📋' },
  { path: '/maps', label: '谱面查询', icon: '🎵' },
  { path: '/scores', label: '成绩查询', icon: '📊' },
  { path: '/test', label: '测试页面', icon: '🔧' },
];

export default function Sidebar() {
  return (
    <aside className="w-60 bg-bs-dark-700 border-r border-bs-dark-600 flex flex-col">
      <div className="p-4 border-b border-bs-dark-600">
        <h1 className="text-xl font-bold text-bs-accent-primary">BSR Monitor</h1>
        <p className="text-xs text-gray-400">BeatSaber 成绩监控</p>
      </div>

      <nav className="flex-1 p-2 space-y-1">
        {navItems.map(item => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) =>
              `flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                isActive
                  ? 'bg-bs-accent-primary/20 text-bs-accent-primary border border-bs-accent-primary/30'
                  : 'text-gray-300 hover:bg-bs-dark-600'
              }`
            }
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>

      <div className="p-4 border-t border-bs-dark-600 text-xs text-gray-500">
        v0.1.0
      </div>
    </aside>
  );
}