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
}

export default function MapScoreGroup({
  map,
  scores,
  players,
  showMapDivider,
}: MapScoreGroupProps) {
  // 过滤掉所有已选玩家都没有成绩的行
  const filteredScores = scores.filter(scoreRow =>
    players.some(player =>
      scoreRow.scores.some(s => s.player_id === player.id)
    )
  );

  // 派生"无成绩"难度（在当前选中的玩家下没有成绩）
  const noScoreRows = scores.filter(scoreRow =>
    !players.some(player =>
      scoreRow.scores.some(s => s.player_id === player.id)
    )
  );

  // 按 mode 分组无成绩难度
  const noScoreDiffGroups: { mode: string; difficulties: string[] }[] = [];
  noScoreRows.forEach(row => {
    const existingGroup = noScoreDiffGroups.find(g => g.mode === row.mode);
    if (existingGroup) {
      if (!existingGroup.difficulties.includes(row.difficulty)) {
        existingGroup.difficulties.push(row.difficulty);
      }
    } else {
      noScoreDiffGroups.push({
        mode: row.mode,
        difficulties: [row.difficulty],
      });
    }
  });

  // 计算网格列数
  const gridColumns = useMemo(() => {
    return `18rem 6rem ${players.map(() => '7rem').join(' ')} minmax(7rem, 1fr)`;
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
        <div className="group p-4 border-r border-bs-border">
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
          {noScoreDiffGroups.length > 0 && (
            <div className="mt-3 pt-3 border-t border-bs-border/50 space-y-1">
              {noScoreDiffGroups.map(group => (
                <div key={group.mode} className="flex items-center justify-center gap-2">
                  <ModeIcon mode={group.mode} size={14} />
                  <div className="flex gap-1">
                    {group.difficulties.map(diff => (
                      <DiffBadge key={diff} difficulty={diff} size="sm" />
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* 难度列 */}
        <div className="p-4 border-r border-bs-border flex items-center justify-center">
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
      <div className="group p-4 border-r border-bs-border row-span-full">
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
                  {group.difficulties.map(diff => (
                    <DiffBadge key={diff} difficulty={diff} size="sm" />
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
            className={`p-4 border-r border-bs-border flex items-center gap-2 ${
              index > 0 ? 'border-t border-bs-border/50' : ''
            }`}
          >
            <ModeIcon mode={scoreRow.mode} size={18} />
            <DiffBadge difficulty={scoreRow.difficulty} />
            {!scoreRow.is_current_version && (
              <span title="非当前版本难度" className="text-xs text-bs-text-muted">🕒</span>
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
                  <span className="font-mono font-medium text-bs-status-success">
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
