import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Map } from '../types';
import MapCard from '../components/MapCard';
import LoadingSpinner from '../components/LoadingSpinner';

export default function MapSearch() {
  const [query, setQuery] = useState('');
  const [map, setMap] = useState<Map | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSearch() {
    if (!query) return;
    setLoading(true);
    setError(null);
    setMap(null);

    try {
      const result = query.length <= 16
        ? await invoke<Map>('get_map_by_key', { key: query })
        : await invoke<Map>('get_map_by_hash', { hash: query });
      setMap(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="animate-fade-in">
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold text-[#f0f0f5] tracking-tight">🎵 谱面查询</h1>
      </div>

      <div className="bg-[#15151c] rounded-xl p-6 border border-[#2a2a3a] shadow-lg mb-6">
        <div className="flex gap-4">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="输入谱面 Key 或 Hash..."
            className="flex-1 px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] placeholder-[#6b7280] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
          />
          <button
            onClick={handleSearch}
            disabled={loading}
            className="px-6 py-3 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all shadow-lg hover:shadow-[#ff6b6b]/30 disabled:opacity-50"
          >
            {loading ? '⏳ 查询中...' : '🔍 查询'}
          </button>
        </div>
      </div>

      {loading && <LoadingSpinner />}

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-4">
          <p className="text-red-400 flex items-center gap-2">
            <span>⚠️</span>
            {error}
          </p>
        </div>
      )}

      {map && <MapCard map={map} />}
    </div>
  );
}