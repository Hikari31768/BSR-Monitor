import { Map, NoScoreDiffGroup, PlayerInfo } from '../types';
import ModeIcon from './ModeIcon';
import DiffBadge from './DiffBadge';

interface NoScoreMapRowProps {
  map: Map;
  noScoreDiffs: NoScoreDiffGroup[];
  players: PlayerInfo[];
  showMapDivider?: boolean;
}

export default function NoScoreMapRow({
  map,
  noScoreDiffs,
  players,
  showMapDivider,
}: NoScoreMapRowProps) {
  return (
    <div
      className={`flex hover:bg-[#1e1e28]/50 transition-colors ${
        showMapDivider ? 'border-t-2 border-[#ff6b6b]/30' : ''
      }`}
    >
      {/* 谱面信息列 */}
      <div className="w-72 flex-shrink-0 p-4 border-r border-[#2a2a3a]">
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
        {/* 所有难度（无成绩） */}
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

      {/* 难度信息列（空） */}
      <div className="w-24 flex-shrink-0 p-4 border-r border-[#2a2a3a] flex items-center justify-center">
        <span className="text-xs text-[#6b7280]">无成绩</span>
      </div>

      {/* 玩家成绩列（空） */}
      {players.map((player) => (
        <div
          key={player.id}
          className="w-28 flex-shrink-0 p-4 border-r border-[#2a2a3a] flex items-center justify-center"
        >
          <span className="text-[#2a2a3a]">-</span>
        </div>
      ))}

      {/* 空白列 */}
      <div className="w-28 flex-shrink-0 p-4"></div>
    </div>
  );
}