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
    <div>
      <h1 className="text-2xl font-bold text-gray-200 mb-6">谱面查询</h1>

      <div className="flex gap-4 mb-6">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="输入谱面 Key 或 Hash"
          className="flex-1 px-4 py-2 bg-bs-dark-700 border border-bs-dark-600 rounded-lg text-gray-200 focus:border-bs-accent-primary focus:outline-none"
        />
        <button
          onClick={handleSearch}
          disabled={loading}
          className="px-6 py-2 bg-bs-accent-primary text-white rounded-lg hover:bg-bs-accent-primary/80 disabled:opacity-50"
        >
          {loading ? '查询中...' : '查询'}
        </button>
      </div>

      {loading && <LoadingSpinner />}

      {error && (
        <div className="bg-red-500/20 border border-red-500/30 rounded-lg p-4 text-red-400">
          {error}
        </div>
      )}

      {map && <MapCard map={map} />}
    </div>
  );
}