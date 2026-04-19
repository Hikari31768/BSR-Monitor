import { Map, ScoreRow, NoScoreDiffGroup } from '../types';
import DiffBadge from './DiffBadge';
import ModeIcon from './ModeIcon';
import { PlayerInfo } from '../types';

interface ScoreTableRowProps {
  map: Map;
  scoreRow: ScoreRow;
  players: PlayerInfo[];
  isFirstRow: boolean;
  totalRows: number; // 该谱面总共有多少难度行
  noScoreDiffs: NoScoreDiffGroup[]; // 无成绩的难度分组
  showMapDivider?: boolean;
}

export default function ScoreTableRow({
  map,
  scoreRow,
  players,
  isFirstRow,
  totalRows,
  noScoreDiffs,
  showMapDivider,
}: ScoreTableRowProps) {
  // 查找每个玩家的成绩
  function getPlayerScore(playerId: string) {
    return scoreRow.scores.find(s => s.player_id === playerId);
  }

  // 格式化 accuracy
  function formatAccuracy(acc: number): string {
    return `${(acc * 100).toFixed(2)}%`;
  }

  // 计算谱面信息单元格高度（合并单元格）
  const mapCellHeight = totalRows > 1 ? totalRows * 56 : 56; // 每行约56px高度

  return (
    <div
      className={`flex hover:bg-[#1e1e28]/50 transition-colors ${
        showMapDivider && isFirstRow ? 'border-t-2 border-[#ff6b6b]/30' : ''
      }`}
    >
      {/* 谱面信息列（合并单元格） */}
      <div
        className={`w-72 flex-shrink-0 p-4 border-r border-[#2a2a3a] ${
          isFirstRow ? '' : 'border-t-0'
        }`}
        style={isFirstRow && totalRows > 1 ? { height: `${mapCellHeight}px` } : {}}
      >
        {isFirstRow && (
          <div>
            <div className="flex gap-3">
              {map.cover_url && (
                <img
                  src={map.cover_url}
                  alt={map.song_name}
                  className="w-16 h-16 rounded-lg object-cover flex-shrink-0"
                />
              )}
              <div className="flex-1 min-w-0">
                <p className="font-medium text-[#f0f0f5] truncate" title={map.song_name}>
                  {map.song_name}
                </p>
                <p className="text-xs text-[#6b7280] truncate" title={map.level_author_name}>
                  {map.level_author_name}
                </p>
                <p className="text-xs text-[#a0a0b0] mt-1">{map.key}</p>
              </div>
            </div>
            {/* 无成绩难度列表 */}
            {noScoreDiffs.length > 0 && (
              <div className="mt-3 pt-3 border-t border-[#2a2a3a]/50 space-y-1">
                {noScoreDiffs.map(group => (
                  <div key={group.mode} className="flex items-center gap-2">
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
        )}
      </div>

      {/* 难度信息列 */}
      <div className="w-24 flex-shrink-0 p-4 border-r border-[#2a2a3a] flex items-center gap-2">
        <ModeIcon mode={scoreRow.mode} size={18} />
        <DiffBadge difficulty={scoreRow.difficulty} />
        {!scoreRow.is_current_version && (
          <span title="非当前版本难度" className="text-xs text-[#6b7280]">🕒</span>
        )}
      </div>

      {/* 玩家成绩列 */}
      {players.map((player) => {
        const score = getPlayerScore(player.id);
        return (
          <div
            key={player.id}
            className={`w-28 flex-shrink-0 p-4 border-r border-[#2a2a3a] flex items-center justify-center ${
              isFirstRow ? '' : 'border-t-0'
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

      {/* 空白列（与添加按钮对齐） */}
      <div className={`w-28 flex-shrink-0 p-4 ${isFirstRow ? '' : 'border-t-0'}`}></div>
    </div>
  );
}