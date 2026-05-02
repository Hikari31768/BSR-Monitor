import { useState, useRef, useEffect, useCallback } from 'react';
import { Playlist } from '../types';

const SOURCE_LABELS: Record<string, string> = {
  beatsaver: 'Saver',
  mapper: '谱师',
  local: '本地',
};

const SOURCE_COLORS: Record<string, string> = {
  beatsaver: 'bg-blue-500/15 text-blue-400 border-blue-500/30',
  mapper: 'bg-purple-500/15 text-purple-400 border-purple-500/30',
  local: 'bg-green-500/15 text-green-400 border-green-500/30',
};

interface PlaylistSelectProps {
  playlists: Playlist[];
  selectedKey: string | null;
  onSelect: (key: string | null) => void;
}

function playlistKey(p: Playlist): string {
  return `${p.source}:${p.identifier}`;
}

export default function PlaylistSelect({ playlists, selectedKey, onSelect }: PlaylistSelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const selectedPlaylist = playlists.find(p => playlistKey(p) === selectedKey);

  // 所有选项（包含"请选择歌单"选项）
  const allOptions = [
    { key: null as string | null, label: '请选择歌单', playlist: null as Playlist | null },
    ...playlists.map(p => ({ key: playlistKey(p), label: `${p.name} (${p.total_maps || 0} 首)`, playlist: p }))
  ];

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

  // 打开时重置 activeIndex
  useEffect(() => {
    if (isOpen) {
      const currentIndex = allOptions.findIndex(o => o.key === selectedKey);
      setActiveIndex(currentIndex >= 0 ? currentIndex : 0);
    }
  }, [isOpen]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!isOpen) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        setIsOpen(true);
      }
      return;
    }

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setActiveIndex(prev => Math.min(prev + 1, allOptions.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setActiveIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (activeIndex >= 0 && activeIndex < allOptions.length) {
          onSelect(allOptions[activeIndex].key);
          setIsOpen(false);
        }
        break;
      case 'Escape':
        e.preventDefault();
        setIsOpen(false);
        break;
    }
  }, [isOpen, activeIndex, allOptions, onSelect]);

  // 滚动活跃项到可视区域
  useEffect(() => {
    if (isOpen && activeIndex >= 0 && listRef.current) {
      const item = listRef.current.children[activeIndex] as HTMLElement;
      if (item) {
        item.scrollIntoView({ block: 'nearest' });
      }
    }
  }, [activeIndex, isOpen]);

  return (
    <div
      ref={containerRef}
      className="relative w-full"
      onKeyDown={handleKeyDown}
    >
      {/* 触发按钮 */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        role="combobox"
        aria-expanded={isOpen}
        aria-haspopup="listbox"
        aria-controls="playlist-listbox"
        aria-label="选择歌单"
        className="w-full px-4 py-3 pr-10 bg-bs-bg-tertiary border border-bs-border rounded-lg text-bs-text-primary text-left hover:border-bs-border-hover transition-all"
        style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E")`,
          backgroundRepeat: 'no-repeat',
          backgroundPosition: 'right 0.75rem center',
          backgroundSize: '1.25rem',
        }}
      >
        {selectedPlaylist ? (
          <div className="flex items-center gap-2">
            <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium ${SOURCE_COLORS[selectedPlaylist.source]}`}>
              {SOURCE_LABELS[selectedPlaylist.source]}
            </span>
            <span className="truncate">{selectedPlaylist.name} ({selectedPlaylist.total_maps || 0} 首)</span>
          </div>
        ) : (
          '请选择歌单'
        )}
      </button>

      {/* 下拉菜单 */}
      {isOpen && (
        <div
          ref={listRef}
          id="playlist-listbox"
          role="listbox"
          aria-label="歌单列表"
          className="absolute top-full left-0 right-0 mt-1 bg-bs-bg-tertiary border border-bs-border rounded-lg shadow-lg overflow-hidden z-50 max-h-60 overflow-y-auto"
        >
          {allOptions.map((option, index) => {
            const sourceKey = option.playlist?.source;
            const sourceLabel = sourceKey ? SOURCE_LABELS[sourceKey] : null;
            const sourceColor = sourceKey ? SOURCE_COLORS[sourceKey] : null;

            return (
              <div
                key={option.key ?? 'none'}
                role="option"
                aria-selected={selectedKey === option.key}
                tabIndex={-1}
                onClick={() => {
                  onSelect(option.key);
                  setIsOpen(false);
                }}
                onMouseEnter={() => setActiveIndex(index)}
                className={`px-4 py-3 cursor-pointer transition-colors flex items-center gap-2 ${
                  selectedKey === option.key
                    ? 'bg-bs-accent-primary/20 text-bs-accent-primary'
                    : index === activeIndex
                      ? 'bg-bs-bg-hover text-bs-text-primary'
                      : 'text-bs-text-secondary hover:text-bs-text-primary'
                }`}
              >
                {sourceLabel && sourceColor && (
                  <span className={`inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium ${sourceColor}`}>
                    {sourceLabel}
                  </span>
                )}
                <span className="truncate">{option.label}</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
