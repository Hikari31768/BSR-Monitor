import { invoke } from '@tauri-apps/api/core';
import { urlPrefixes } from '../constants';

interface OpenBeatSaverLinkButtonProps {
  mapKey: string;
  onButtonClick: Function
}

export default function OpenBeatSaverLinkButton({ mapKey, onButtonClick }: OpenBeatSaverLinkButtonProps) {
  async function handleClick() {
    try {
      await invoke('open_url', { url: urlPrefixes.BeatSaverMap + mapKey });
      onButtonClick();
    } catch (e) {
      console.error('打开链接失败:', e);
    }
  }

  return (
    <button
      onClick={handleClick}
      className="inline-flex p-1 items-center justify-center rounded w-5 h-5 text-xs font-semibold leading-none text-bs-accent-purple bg-bs-accent-purple/20 hover:bg-bs-accent-purple/40 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity duration-100 border border-bs-accent-purple/40"
      title="前往 BeatSaver 页面"
    >
      <img
        src="/icons/link.svg"
        alt="BeatSaver Map Link"
      />
    </button>
  );
}
