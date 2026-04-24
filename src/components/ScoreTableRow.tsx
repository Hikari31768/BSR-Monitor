import { Map, ScoreRowData, NoScoreDiffGroup, PlayerInfo } from '../types';
import DiffBadge from './DiffBadge';
import ModeIcon from './ModeIcon';

interface ScoreTableRowProps {
  map: Map;
  scoreRow: ScoreRowData;
  players: PlayerInfo[];
  rowIndex: number;
  totalRows: number;
  noScoreDiffs: NoScoreDiffGroup[];
  showMapDivider?: boolean;
}

export default function ScoreTableRow({
  map,
  scoreRow,
  players,
  rowIndex,
  totalRows,
  noScoreDiffs,
  showMapDivider,
}: ScoreTableRowProps) {
  const isFirstRow = rowIndex === 0;

  // 查找每个玩家的成绩
  function getPlayerScore(playerId: string) {
    return scoreRow.scores.find(s => s.player_id === playerId);
  }

  // 格式化 accuracy
  function formatAccuracy(acc: number): string {
    return `${(acc * 100).toFixed(2)}%`;
  }

  return (
    <div
      className={`contents ${
        showMapDivider && isFirstRow ? 'border-t-2 border-bs-accent-primary/30' : ''
      }`}
    >
      {/* 谱面信息列 - 只在第一行显示，使用 grid-row 跨多行 */}
      {isFirstRow ? (
        <div
          className="w-72 p-4 border-r border-bs-border bg-bs-bg-secondary"
          style={{ gridRow: `span ${totalRows}` }}
        >
          <div className="flex gap-3">
            {map.cover_url && (
              <img
                src={map.cover_url}
                alt={map.song_name}
                className="w-16 h-16 rounded-lg object-cover flex-shrink-0"
              />
            )}
            <div className="flex-1 min-w-0">
              <p className="font-medium text-bs-text-primary truncate" title={map.song_name}>
                {map.song_name}
              </p>
              <p className="text-xs text-bs-text-muted truncate" title={map.level_author_name}>
                {map.level_author_name}
              </p>
              <p className="text-xs text-bs-text-secondary mt-1">{map.key}</p>
            </div>
          </div>
          {/* 无成绩难度列表 */}
          {noScoreDiffs.length > 0 && (
            <div className="mt-3 pt-3 border-t border-bs-border/50 space-y-1">
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
      ) : null}

      {/* 难度信息列 */}
      <div className="w-24 p-4 border-r border-bs-border flex items-center gap-2">
        <ModeIcon mode={scoreRow.mode} size={18} />
        <DiffBadge difficulty={scoreRow.difficulty} />
        {!scoreRow.is_current_version && (
          <span title="非当前版本难度" className="text-xs text-bs-text-muted">🕒</span>
        )}
      </div>

      {/* 玩家成绩列 */}
      {players.map((player) => {
        const score = getPlayerScore(player.id);
        return (
          <div
            key={player.id}
            className="w-28 p-4 border-r border-bs-border flex items-center justify-center"
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
      <div className="w-28 p-4"></div>
    </div>
  );
}
