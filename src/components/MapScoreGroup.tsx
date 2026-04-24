import { useState } from 'react';
import { Map, ScoreRowData, NoScoreDiffGroup, PlayerInfo } from '../types';
import DiffBadge from './DiffBadge';
import ModeIcon from './ModeIcon';
import CoverWithAudio from './CoverWithAudio';

function CopyKeyButton({ mapKey }: { mapKey: string }) {
  const [copied, setCopied] = useState(false);

  function handleClick() {
    navigator.clipboard.writeText(`!bsr ${mapKey}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  }

  return (
    <>
      <button
        onClick={handleClick}
        className="inline-flex items-center justify-center rounded w-5 h-5 text-xs font-semibold leading-none text-[#a78bfa] bg-[#7c3aed]/20 hover:bg-[#7c3aed]/40 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-100"
        style={{ border: '1px solid rgba(124, 58, 237, 0.4)' }}
        title="复制 !bsr 命令"
      >
        !
      </button>
      <span
        className={`inline-flex items-center justify-center rounded h-5 px-2 text-xs font-semibold leading-none text-[#a78bfa] bg-[#7c3aed]/20 transition-opacity duration-100 ${
          copied ? 'opacity-100' : 'opacity-0'
        }`}
        style={{ border: '1px solid rgba(124, 58, 237, 0.4)' }}
      >
        !bsr 已复制
      </span>
    </>
  );
}

interface MapScoreGroupProps {
  map: Map;
  scores: ScoreRowData[];
  noScoreDiffs: NoScoreDiffGroup[];
  players: PlayerInfo[];
  showMapDivider?: boolean;
}

export default function MapScoreGroup({
  map,
  scores,
  noScoreDiffs,
  players,
  showMapDivider,
}: MapScoreGroupProps) {
  // 过滤掉所有玩家都没有成绩的行
  const filteredScores = scores.filter(scoreRow =>
    players.some(player =>
      scoreRow.scores.some(s => s.player_id === player.id)
    )
  );

  // 计算被过滤掉的难度（原本有成绩，但当前筛选玩家都没有）
  const filteredOutRows = scores.filter(scoreRow =>
    !players.some(player =>
      scoreRow.scores.some(s => s.player_id === player.id)
    )
  );

  // 合并到 noScoreDiffs 显示：按 mode 分组
  const extraNoScoreDiffs: NoScoreDiffGroup[] = [];
  filteredOutRows.forEach(row => {
    const existingGroup = extraNoScoreDiffs.find(g => g.mode === row.mode);
    if (existingGroup) {
      if (!existingGroup.difficulties.includes(row.difficulty)) {
        existingGroup.difficulties.push(row.difficulty);
      }
    } else {
      extraNoScoreDiffs.push({
        mode: row.mode,
        difficulties: [row.difficulty],
      });
    }
  });

  // 合并原始 noScoreDiffs 和被过滤掉的难度
  const allNoScoreDiffs: NoScoreDiffGroup[] = [...noScoreDiffs];
  extraNoScoreDiffs.forEach(extra => {
    const existingGroup = allNoScoreDiffs.find(g => g.mode === extra.mode);
    if (existingGroup) {
      extra.difficulties.forEach(diff => {
        if (!existingGroup.difficulties.includes(diff)) {
          existingGroup.difficulties.push(diff);
        }
      });
    } else {
      allNoScoreDiffs.push(extra);
    }
  });

  // 计算网格列数：谱面信息(1) + 难度(1) + 玩家列数 + 空白列(1)
  const gridColumns = `18rem 6rem ${players.map(() => '7rem').join(' ')} 7rem`;

  // 查找每个玩家的成绩
  function getPlayerScore(scoreRow: ScoreRowData, playerId: string) {
    return scoreRow.scores.find(s => s.player_id === playerId);
  }

  // 格式化 accuracy
  function formatAccuracy(acc: number): string {
    return `${(acc * 100).toFixed(2)}%`;
  }

  if (scores.length === 0) {
    // 无成绩的情况
    return (
      <div
        className={`grid border-b border-[#2a2a3a] hover:bg-[#1e1e28]/50 transition-colors ${
          showMapDivider ? 'border-t-2 border-[#ff6b6b]/30' : ''
        }`}
        style={{ gridTemplateColumns: gridColumns }}
      >
        {/* 谱面信息 */}
        <div className="group p-4 border-r border-[#2a2a3a]">
          <div className="flex gap-3">
            <CoverWithAudio map={map} />
            <div className="flex-1 min-w-0">
              <p className="font-medium text-[#f0f0f5] truncate">{map.song_name}</p>
              <p className="text-xs text-[#6b7280] truncate">{map.level_author_name}</p>
              <div className="flex items-center gap-1 mt-1">
                <p className="text-xs text-[#a0a0b0]">{map.key}</p>
                <CopyKeyButton mapKey={map.key} />
              </div>
            </div>
          </div>
          {allNoScoreDiffs.length > 0 && (
            <div className="mt-3 pt-3 border-t border-[#2a2a3a]/50 space-y-1">
              {allNoScoreDiffs.map(group => (
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

        {/* 难度列 - 显示无成绩 */}
        <div className="p-4 border-r border-[#2a2a3a] flex items-center justify-center">
          <span className="text-xs text-[#6b7280]">无成绩</span>
        </div>

        {/* 玩家列 - 空 */}
        {players.map(player => (
          <div key={player.id} className="p-4 border-r border-[#2a2a3a] flex items-center justify-center">
            <span className="text-[#2a2a3a]">-</span>
          </div>
        ))}

        {/* 空白列 */}
        <div className="p-4"></div>
      </div>
    );
  }

  // 有成绩的情况
  return (
    <div
      className={`grid border-b border-[#2a2a3a] hover:bg-[#1e1e28]/50 transition-colors ${
        showMapDivider ? 'border-t-2 border-[#ff6b6b]/30' : ''
      }`}
      style={{
        gridTemplateColumns: gridColumns,
        gridTemplateRows: `repeat(${filteredScores.length}, minmax(3.5rem, auto))`,
      }}
    >
      {/* 谱面信息列 - 跨所有行 */}
      <div className="group p-4 border-r border-[#2a2a3a] row-span-full">
        <div className="flex gap-3">
          <CoverWithAudio map={map} />
          <div className="flex-1 min-w-0">
            <p className="font-medium text-[#f0f0f5] truncate">{map.song_name}</p>
            <p className="text-xs text-[#6b7280] truncate">{map.level_author_name}</p>
            <div className="flex items-center gap-1 mt-1">
              <p className="text-xs text-[#a0a0b0]">{map.key}</p>
              <CopyKeyButton mapKey={map.key} />
            </div>
          </div>
        </div>
        {/* 无成绩难度列表 */}
        {allNoScoreDiffs.length > 0 && (
          <div className="mt-3 pt-3 border-t border-[#2a2a3a]/50 space-y-1">
            {allNoScoreDiffs.map(group => (
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
            className={`p-4 border-r border-[#2a2a3a] flex items-center gap-2 ${
              index > 0 ? 'border-t border-[#2a2a3a]/50' : ''
            }`}
          >
            <ModeIcon mode={scoreRow.mode} size={18} />
            <DiffBadge difficulty={scoreRow.difficulty} />
            {!scoreRow.is_current_version && (
              <span title="非当前版本难度" className="text-xs text-[#6b7280]">🕒</span>
            )}
          </div>

          {/* 玩家成绩 */}
          {players.map(player => {
            const score = getPlayerScore(scoreRow, player.id);
            return (
              <div
                key={`score-${index}-${player.id}`}
                className={`p-4 border-r border-[#2a2a3a] flex items-center justify-center ${
                  index > 0 ? 'border-t border-[#2a2a3a]/50' : ''
                }`}
              >
                {score ? (
                  <span className="font-mono font-medium text-[#4ade80]">
                    {formatAccuracy(score.accuracy)}
                  </span>
                ) : (
                  <span className="text-[#2a2a3a]">-</span>
                )}
              </div>
            );
          })}

          {/* 空白列 */}
          <div
            key={`empty-${index}`}
            className={`p-4 ${index > 0 ? 'border-t border-[#2a2a3a]/50' : ''}`}
          ></div>
        </>
      ))}
    </div>
  );
}
