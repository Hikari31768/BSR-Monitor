import { useState, useRef, useEffect } from 'react';
import { Playlist } from '../types';

interface PlaylistSelectProps {
  playlists: Playlist[];
  selectedId: number | null;
  onSelect: (id: number | null) => void;
}

export default function PlaylistSelect({ playlists, selectedId, onSelect }: PlaylistSelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const selectedPlaylist = playlists.find(p => p.playlist_id === selectedId);

  // 点击外部关闭
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div ref={containerRef} className="relative">
      {/* 触发按钮 */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-4 py-3 pr-10 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] text-left hover:border-[#3a3a4f] transition-all"
        style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E")`,
          backgroundRepeat: 'no-repeat',
          backgroundPosition: 'right 0.75rem center',
          backgroundSize: '1.25rem',
        }}
      >
        {selectedPlaylist
          ? `${selectedPlaylist.name} (${selectedPlaylist.total_maps || 0} 首)`
          : '请选择歌单'}
      </button>

      {/* 下拉菜单 */}
      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg shadow-lg overflow-hidden z-50">
          <div
            onClick={() => {
              onSelect(null);
              setIsOpen(false);
            }}
            className={`px-4 py-3 cursor-pointer hover:bg-[#2a2a3a] transition-colors ${
              selectedId === null ? 'bg-[#ff6b6b]/20 text-[#ff6b6b]' : 'text-[#f0f0f5]'
            }`}
          >
            请选择歌单
          </div>
          {playlists.map(playlist => (
            <div
              key={playlist.playlist_id}
              onClick={() => {
                onSelect(playlist.playlist_id);
                setIsOpen(false);
              }}
              className={`px-4 py-3 cursor-pointer hover:bg-[#2a2a3a] transition-colors ${
                selectedId === playlist.playlist_id ? 'bg-[#ff6b6b]/20 text-[#ff6b6b]' : 'text-[#f0f0f5]'
              }`}
            >
              {playlist.name} ({playlist.total_maps || 0} 首)
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
