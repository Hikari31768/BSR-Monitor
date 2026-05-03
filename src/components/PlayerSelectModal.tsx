import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PlayerInfo } from '../types';
import Modal from './Modal';
import { User } from 'lucide-react';

interface PlayerSelectModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (player: PlayerInfo) => void;
  excludeIds: string[];
}

export default function PlayerSelectModal({ isOpen, onClose, onSelect, excludeIds }: PlayerSelectModalProps) {
  const [players, setPlayers] = useState<PlayerInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (isOpen) {
      loadPlayers();
    }
  }, [isOpen]);

  async function loadPlayers() {
    setLoading(true);
    try {
      const list = await invoke<PlayerInfo[]>('get_player_list');
      setPlayers(list);
    } catch (e) {
      console.error('加载玩家失败:', e);
    } finally {
      setLoading(false);
    }
  }

  const availablePlayers = players.filter(p => !excludeIds.includes(p.id));

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="选择玩家">
      <div className="w-96 max-h-96 overflow-y-auto score-table-scroll">
        {loading ? (
          <div className="flex items-center justify-center py-8">
            <div className="w-6 h-6 border-2 border-bs-accent-primary border-t-transparent rounded-full animate-spin"></div>
          </div>
        ) : availablePlayers.length === 0 ? (
          <div className="text-center py-8 text-bs-text-muted">
            <div className="mb-2"><User size={24} /></div>
            <p>没有更多可选玩家</p>
          </div>
        ) : (
          <div className="space-y-2">
            {availablePlayers.map(player => (
              <button
                key={player.id}
                onClick={() => {
                  onSelect(player);
                  onClose();
                }}
                className="w-full flex items-center gap-3 p-3 rounded-lg bg-bs-bg-tertiary hover:bg-bs-border transition-colors text-left"
              >
                {player.avatar_url ? (
                  <img
                    src={player.avatar_url}
                    alt={player.name}
                    className="w-10 h-10 rounded-full object-cover"
                  />
                ) : (
                  <div className="w-10 h-10 rounded-full bg-bs-border flex items-center justify-center">
                    <User size={20} className="text-bs-text-muted" />
                  </div>
                )}
                <div>
                  <p className="font-medium text-bs-text-primary">{player.alias || player.name}</p>
                  <p className="text-xs text-bs-text-muted">{player.name}</p>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    </Modal>
  );
}
