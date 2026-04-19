import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PlayerInfo, Map, PlayerScore } from '../types';
import ScoreRow from '../components/ScoreRow';
import MapCard from '../components/MapCard';
import LoadingSpinner from '../components/LoadingSpinner';
import EmptyState from '../components/EmptyState';

export default function ScoreLookup() {
  const [players, setPlayers] = useState<PlayerInfo[]>([]);
  const [selectedPlayer, setSelectedPlayer] = useState<PlayerInfo | null>(null);
  const [mapQuery, setMapQuery] = useState('');
  const [map, setMap] = useState<Map | null>(null);
  const [scores, setScores] = useState<PlayerScore[]>([]);
  const [loadingPlayers, setLoadingPlayers] = useState(true);
  const [loadingMap, setLoadingMap] = useState(false);
  const [loadingScores, setLoadingScores] = useState(false);

  useEffect(() => {
    async function loadPlayers() {
      try {
        const list = await invoke<PlayerInfo[]>('get_player_list');
        setPlayers(list);
      } catch (e) {
        console.error('加载玩家失败:', e);
      } finally {
        setLoadingPlayers(false);
      }
    }
    loadPlayers();
  }, []);

  async function handleSearchMap() {
    if (!mapQuery) return;
    setLoadingMap(true);
    setMap(null);
    setScores([]);

    try {
      const result = mapQuery.length <= 16
        ? await invoke<Map>('get_map_by_key', { key: mapQuery })
        : await invoke<Map>('get_map_by_hash', { hash: mapQuery });
      setMap(result);

      if (selectedPlayer) {
        await loadScoresForPlayer(selectedPlayer.id, result.key);
      }
    } catch (e) {
      alert(`查询失败: ${e}`);
    } finally {
      setLoadingMap(false);
    }
  }

  async function loadScoresForPlayer(playerId: string, mapKey: string) {
    setLoadingScores(true);
    try {
      const allScores = await invoke<PlayerScore[]>('get_player_scores', { playerId });
      const mapScores = allScores.filter(s => s.map_key === mapKey);
      setScores(mapScores);
    } catch (e) {
      console.error('获取成绩失败:', e);
    } finally {
      setLoadingScores(false);
    }
  }

  async function handlePlayerChange(playerId: string) {
    const player = players.find(p => p.id === playerId);
    setSelectedPlayer(player || null);

    if (player && map) {
      await loadScoresForPlayer(player.id, map.key);
    }
  }

  if (loadingPlayers) return <LoadingSpinner />;

  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-200 mb-6">成绩查询</h1>

      {/* 玩家选择 */}
      <div className="mb-4">
        <label className="block text-sm text-gray-400 mb-1">选择玩家</label>
        <select
          value={selectedPlayer?.id || ''}
          onChange={(e) => handlePlayerChange(e.target.value)}
          className="w-full px-4 py-2 bg-bs-dark-700 border border-bs-dark-600 rounded-lg text-gray-200 focus:border-bs-accent-primary focus:outline-none"
        >
          <option value="">请选择玩家</option>
          {players.map(player => (
            <option key={player.id} value={player.id}>
              {player.alias || player.name}
            </option>
          ))}
        </select>
      </div>

      {/* 谱面查询 */}
      <div className="flex gap-4 mb-6">
        <input
          type="text"
          value={mapQuery}
          onChange={(e) => setMapQuery(e.target.value)}
          placeholder="输入谱面 Key 或 Hash"
          className="flex-1 px-4 py-2 bg-bs-dark-700 border border-bs-dark-600 rounded-lg text-gray-200 focus:border-bs-accent-primary focus:outline-none"
        />
        <button
          onClick={handleSearchMap}
          disabled={loadingMap}
          className="px-6 py-2 bg-bs-accent-primary text-white rounded-lg hover:bg-bs-accent-primary/80 disabled:opacity-50"
        >
          {loadingMap ? '查询中...' : '查询'}
        </button>
      </div>

      {/* 谱面信息 */}
      {map && <MapCard map={map} />}

      {/* 成绩列表 */}
      {selectedPlayer && map && (
        <div className="mt-6">
          <h2 className="text-lg font-semibold text-gray-200 mb-4">成绩列表</h2>
          {loadingScores ? (
            <LoadingSpinner />
          ) : scores.length === 0 ? (
            <EmptyState message="该玩家尚未游玩此谱面" icon="🚫" />
          ) : (
            <div className="space-y-2">
              {scores.map(score => (
                <ScoreRow key={score.score_id} score={score} />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}