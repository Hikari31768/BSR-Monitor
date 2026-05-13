import { useMemo } from 'react';
import { Map, ScoreRowData, PlayerInfo } from '../types';
import DiffBadge from './DiffBadge';
import ModeIcon from './ModeIcon';
import CoverWithAudio from './CoverWithAudio';
import CopyKeyButton from './CopyKeyButton';
import { formatAccuracy } from '../utils/format';

interface MapScoreGroupProps {
  map: Map;
  scores: ScoreRowData[];
  players: PlayerInfo[];
  showMapDivider?: boolean;
  diffSort?: 'asc' | 'desc';
}

const diffOrderValue = (d: string) => {
  const map: Record<string, number> = { Easy: 1, Normal: 2, Hard: 3, Expert: 4, ExpertPlus: 5 };
  return map[d] ?? 99;
};

const modeOrderValue = (m: string) => {
  const map: Record<string, number> = { Standard: 1, OneSaber: 2, NoArrows: 3, '90Degree': 4, '360Degree': 5, Lightshow: 6, Lawless: 7, Legacy: 8 };
  return map[m] ?? 99;
};

export default function MapScoreGroup({
  map,
  scores,
  players,
  showMapDivider,
  diffSort = 'desc',
  onPreviewUrlReady,
}: MapScoreGroupProps & { onPreviewUrlReady?: (url: string) => void }) {
  // 难度列排序（前端排序）
  const sortedScores = useMemo(() => {
    return [...scores].sort((a, b) => {
      const mo = modeOrderValue(a.mode) - modeOrderValue(b.mode);
      if (mo !== 0) return mo;
      return diffSort === 'desc' ? diffOrderValue(b.difficulty) - diffOrderValue(a.difficulty) : diffOrderValue(a.difficulty) - diffOrderValue(b.difficulty);
    });
  }, [scores, diffSort]);

  // 过滤掉所有已选玩家都没有成绩的行
  const filteredScores = sortedScores.filter(scoreRow =>
    players.some(player =>
      scoreRow.scores.some(s => s.player_id === player.id)
    )
  );

  // 派生"无成绩"难度（在当前选中的玩家下没有成绩）
  const noScoreRows = sortedScores.filter(scoreRow =>
    !players.some(player =>
      scoreRow.scores.some(s => s.player_id === player.id)
    )
  );

  // 按 mode 分组无成绩难度（保留星级信息）
  const noScoreDiffGroups: { mode: string; difficulties: { difficulty: string; stars?: number }[] }[] = [];
  noScoreRows.forEach(row => {
    const existingGroup = noScoreDiffGroups.find(g => g.mode === row.mode);
    if (existingGroup) {
      if (!existingGroup.difficulties.find(d => d.difficulty === row.difficulty)) {
        existingGroup.difficulties.push({ difficulty: row.difficulty, stars: row.stars ?? undefined });
      }
    } else {
      noScoreDiffGroups.push({
        mode: row.mode,
        difficulties: [{ difficulty: row.difficulty, stars: row.stars ?? undefined }],
      });
    }
  });

  // 折叠区难度始终正序
  noScoreDiffGroups.forEach(g => g.difficulties.sort((a, b) => diffOrderValue(a.difficulty) - diffOrderValue(b.difficulty)));

  // 计算网格列数
  const gridColumns = useMemo(() => {
    return `18rem 9rem ${players.map(() => '7rem').join(' ')} minmax(7rem, 1fr)`;
  }, [players.length]);

  // 查找每个玩家的成绩
  function getPlayerScore(scoreRow: ScoreRowData, playerId: string) {
    return scoreRow.scores.find(s => s.player_id === playerId);
  }

  // 无成绩快捷渲染：所有难度对当前玩家都无成绩
  if (filteredScores.length === 0) {
    return (
      <div
        className={`grid border-b border-bs-border hover:bg-bs-bg-tertiary/50 transition-colors min-w-fit ${
          showMapDivider ? 'border-t-2 border-bs-accent-primary/30' : ''
        }`}
        style={{ gridTemplateColumns: gridColumns }}
      >
        {/* 谱面信息 */}
        <div className="sticky-col-1 group p-4 border-r border-bs-border">
          <div className="flex gap-3">
            <CoverWithAudio map={map} onPreviewUrlReady={onPreviewUrlReady} />
            <div className="flex-1 min-w-0">
              <p className="font-medium text-bs-text-primary truncate">{map.song_name}</p>
              <p className="text-xs text-bs-text-muted truncate">{map.level_author_name}</p>
              <div className="flex items-center gap-1 mt-1">
                <p className="text-xs text-bs-text-secondary">{map.key}</p>
                <CopyKeyButton mapKey={map.key} />
              </div>
            </div>
          </div>
          {noScoreDiffGroups.length > 0 && (
            <div className="mt-3 pt-3 border-t border-bs-border/50 space-y-1">
              {noScoreDiffGroups.map(group => (
                <div key={group.mode} className="flex items-center justify-center gap-2">
                  <ModeIcon mode={group.mode} size={14} />
                  <div className="flex gap-1">
                    {group.difficulties.map(d => (
                      <DiffBadge key={d.difficulty} difficulty={d.difficulty} size="sm" stars={d.stars} />
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* 难度列 */}
        <div className="sticky-col-2 p-4 border-r border-bs-border flex items-center justify-center">
          <span className="text-xs text-bs-text-muted">无成绩</span>
        </div>

        {/* 玩家列 */}
        {players.map(player => (
          <div key={player.id} className="p-4 border-r border-bs-border flex items-center justify-center">
            <span className="text-bs-border">-</span>
          </div>
        ))}

        {/* 空白列 */}
        <div className="p-4"></div>
      </div>
    );
  }

  // 有成绩的情况 — 统一渲染路径
  return (
    <div
      className={`grid border-b border-bs-border hover:bg-bs-bg-tertiary/50 transition-colors min-w-fit ${
        showMapDivider ? 'border-t-2 border-bs-accent-primary/30' : ''
      }`}
      style={{
        gridTemplateColumns: gridColumns,
        gridTemplateRows: `repeat(${filteredScores.length}, minmax(3.5rem, auto))`,
      }}
    >
      {/* 谱面信息列 - 跨所有行 */}
      <div className="sticky-col-1 group p-4 border-r border-bs-border row-span-full">
        <div className="flex gap-3">
          <CoverWithAudio map={map} />
          <div className="flex-1 min-w-0">
            <p className="font-medium text-bs-text-primary truncate">{map.song_name}</p>
            <p className="text-xs text-bs-text-muted truncate">{map.level_author_name}</p>
            <div className="flex items-center gap-1 mt-1">
              <p className="text-xs text-bs-text-secondary">{map.key}</p>
              <CopyKeyButton mapKey={map.key} />
            </div>
          </div>
        </div>
        {/* 无成绩难度徽章 */}
        {noScoreDiffGroups.length > 0 && (
          <div className="mt-3 pt-3 border-t border-bs-border/50 space-y-1">
            {noScoreDiffGroups.map(group => (
              <div key={group.mode} className="flex items-center justify-center gap-2">
                <ModeIcon mode={group.mode} size={14} />
                <div className="flex gap-1">
                  {group.difficulties.map(d => (
                    <DiffBadge key={d.difficulty} difficulty={d.difficulty} size="sm" stars={d.stars} />
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 每个难度一行 */}
      {filteredScores.map((scoreRow, index) => (
        <>
          {/* 难度信息 */}
          <div
            key={`diff-${index}`}
            className={`sticky-col-2 p-4 border-r border-bs-border flex items-center gap-2 ${
              index > 0 ? 'border-t border-bs-border/50' : ''
            }`}
          >
            <ModeIcon mode={scoreRow.mode} size={18} />
            <DiffBadge difficulty={scoreRow.difficulty} stars={scoreRow.stars ?? undefined} />
            {!scoreRow.is_current_version && (
              <span title="非当前版本难度" className="text-xs text-bs-text-muted"></span>
            )}
          </div>

          {/* 玩家成绩 */}
          {players.map(player => {
            const score = getPlayerScore(scoreRow, player.id);
            return (
              <div
                key={`score-${index}-${player.id}`}
                className={`p-4 border-r border-bs-border flex items-center justify-center ${
                  index > 0 ? 'border-t border-bs-border/50' : ''
                }`}
              >
                {score ? (
                  <span className={`font-mono font-medium ${
                    score.is_expired
                      ? 'text-bs-text-muted'
                      : 'text-bs-status-success'
                  }`}>
                    {score.is_expired && (
                      <svg className="inline-block w-3 h-3 mr-1 opacity-60" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="12,6 12,12 16,14"/>
                      </svg>
                    )}
                    {formatAccuracy(score.accuracy)}
                  </span>
                ) : (
                  <span className="text-bs-border">-</span>
                )}
              </div>
            );
          })}

          {/* 空白列 */}
          <div
            key={`empty-${index}`}
            className={`p-4 ${index > 0 ? 'border-t border-bs-border/50' : ''}`}
          ></div>
        </>
      ))}
    </div>
  );
}
