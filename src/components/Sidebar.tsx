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
    <aside className="w-64 bg-[#15151c] border-r border-[#2a2a3a] flex flex-col shrink-0">
      {/* Logo 区域 */}
      <div className="p-6 border-b border-[#2a2a3a]">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-[#ff6b6b] to-[#9b59b6] flex items-center justify-center text-white text-lg font-bold shadow-lg">
            BS
          </div>
          <div>
            <h1 className="text-lg font-bold text-[#f0f0f5] tracking-tight">BSR Monitor</h1>
            <p className="text-xs text-[#6b7280]">BeatSaber 成绩监控</p>
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
                  ? 'bg-[#ff6b6b]/10 text-[#ff6b6b] border border-[#ff6b6b]/20'
                  : 'text-[#a0a0b0] hover:bg-[#1e1e28] hover:text-[#f0f0f5]'
              }`
            }
          >
            <span className="text-lg">{item.icon}</span>
            <span className="font-medium">{item.label}</span>
          </NavLink>
        ))}
      </nav>

      {/* 底部信息 */}
      <div className="p-4 border-t border-[#2a2a3a]">
        <p className="text-xs text-[#6b7280]">v0.1.0</p>
      </div>
    </aside>
  );
}