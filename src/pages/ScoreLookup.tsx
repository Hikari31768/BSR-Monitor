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
    <div className="animate-fade-in">
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-bold text-[#f0f0f5] tracking-tight">📊 成绩查询</h1>
      </div>

      {/* 玩家选择 */}
      <div className="bg-[#15151c] rounded-xl p-6 border border-[#2a2a3a] shadow-lg mb-6">
        <label className="block text-sm text-[#a0a0b0] mb-2">选择玩家</label>
        <select
          value={selectedPlayer?.id || ''}
          onChange={(e) => handlePlayerChange(e.target.value)}
          className="w-full px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
        >
          <option value="" className="bg-[#1e1e28]">请选择玩家</option>
          {players.map(player => (
            <option key={player.id} value={player.id} className="bg-[#1e1e28]">
              {player.alias || player.name}
            </option>
          ))}
        </select>
      </div>

      {/* 谱面查询 */}
      <div className="bg-[#15151c] rounded-xl p-6 border border-[#2a2a3a] shadow-lg mb-6">
        <label className="block text-sm text-[#a0a0b0] mb-2">谱面查询</label>
        <div className="flex gap-4">
          <input
            type="text"
            value={mapQuery}
            onChange={(e) => setMapQuery(e.target.value)}
            placeholder="输入谱面 Key 或 Hash..."
            className="flex-1 px-4 py-3 bg-[#1e1e28] border border-[#2a2a3a] rounded-lg text-[#f0f0f5] placeholder-[#6b7280] focus:border-[#ff6b6b] focus:ring-2 focus:ring-[#ff6b6b]/20 focus:outline-none transition-all"
            onKeyDown={(e) => e.key === 'Enter' && handleSearchMap()}
          />
          <button
            onClick={handleSearchMap}
            disabled={loadingMap}
            className="px-6 py-3 rounded-lg font-medium text-sm bg-[#ff6b6b] text-white hover:bg-[#ff5252] transition-all shadow-lg hover:shadow-[#ff6b6b]/30 disabled:opacity-50"
          >
            {loadingMap ? '⏳' : '🔍 查询'}
          </button>
        </div>
      </div>

      {/* 谱面信息 */}
      {map && <MapCard map={map} />}

      {/* 成绩列表 */}
      {selectedPlayer && map && (
        <div className="mt-6 animate-fade-in">
          <h2 className="text-xl font-bold text-[#f0f0f5] mb-4 flex items-center gap-2">
            📈 成绩列表
            {scores.length > 0 && (
              <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-[#22c55e]/15 text-[#4ade80] border border-[#22c55e]/30">
                {scores.length} 条
              </span>
            )}
          </h2>
          {loadingScores ? (
            <LoadingSpinner />
          ) : scores.length === 0 ? (
            <EmptyState message="该玩家尚未游玩此谱面" icon="🚫" />
          ) : (
            <div className="space-y-3">
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