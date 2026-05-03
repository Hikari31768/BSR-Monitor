import { useState, useEffect, useRef } from 'react';
import { Map } from '../types';
import { useAudio } from '../contexts/AudioContext';
import { invoke } from '@tauri-apps/api/core';

const RADIUS = 26;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

interface CoverWithAudioProps {
  map: Map;
  onPreviewUrlReady?: (url: string) => void;
}

export default function CoverWithAudio({ map, onPreviewUrlReady }: CoverWithAudioProps) {
  const [progress, setProgress] = useState(0);
  const [loading, setLoading] = useState(false);
  const { playPreview, stopPreview, audioRef, currentPlayingId } = useAudio();
  const isOwnRef = useRef(false);

  // 判断当前组件是否正在播放（基于全局 currentPlayingId）
  const isPlaying = currentPlayingId === map.key;

  useEffect(() => {
    return () => {
      // 当组件卸载或切换谱面（map.key 变化）时停止播放
      // 注意：这里不监听 map.preview_url，避免在补充 preview_url 时误停止播放
      if (isOwnRef.current) {
        stopPreview();
        setProgress(0);
        isOwnRef.current = false;
      }
    };
  }, [map.key, stopPreview]);

  // Track progress on the global audio element
  useEffect(() => {
    if (!isPlaying) {
      setProgress(0);
      return;
    }

    const audio = audioRef.current;
    if (!audio) return;

    const handleTimeUpdate = () => {
      if (audio.duration) {
        setProgress(audio.currentTime / audio.duration);
      }
    };
    const handleEnded = () => {
      setProgress(0);
      isOwnRef.current = false;
    };

    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('ended', handleEnded);

    return () => {
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('ended', handleEnded);
    };
  }, [isPlaying, audioRef]);

  async function handleClick() {
    if (isPlaying) {
      stopPreview();
      isOwnRef.current = false;
      return;
    }

    // 如果已有 previewURL，直接播放
    if (map.preview_url) {
      await playPreview(map.preview_url, map.key);
      isOwnRef.current = true;
      return;
    }

    // 没有 previewURL，按需从 BeatSaver 获取
    if (loading) return;
    setLoading(true);

    try {
      const url = await invoke<string>('fetch_and_store_preview_url', { key: map.key });
      // 先开始播放，再通知父组件更新
      await playPreview(url, map.key);
      isOwnRef.current = true;
      // 播放开始后再更新父组件的 map.preview_url
      onPreviewUrlReady?.(url);
    } catch {
      // 获取失败或播放失败，静默处理
    } finally {
      setLoading(false);
    }
  }

  if (!map.cover_url) return null;

  const dashOffset = CIRCUMFERENCE * (1 - progress);

  return (
    <button
      onClick={handleClick}
      className="relative w-16 h-16 rounded-lg overflow-hidden flex-shrink-0 cursor-pointer group/cover"
      title={map.preview_url ? '点击播放预览' : '点击获取预览音频'}
    >
      <img
        src={map.cover_url}
        alt={map.song_name}
        className={`w-full h-full object-cover transition-opacity ${isPlaying ? 'opacity-70' : 'group-hover/cover:opacity-80'} ${loading ? 'opacity-40' : ''}`}
      />
      {loading && (
        <svg
          className="absolute inset-0 w-full h-full animate-spin"
          viewBox="0 0 64 64"
        >
          {/* 轨道 */}
          <circle
            cx="32" cy="32" r={RADIUS}
            fill="none"
            stroke="rgba(255,255,255,0.15)"
            strokeWidth="3"
          />
          {/* Loading 指示器 - 一小段高亮弧线 */}
          <circle
            cx="32" cy="32" r={RADIUS}
            fill="none"
            stroke="currentColor"
            className="text-bs-accent-purple"
            strokeWidth="3"
            strokeDasharray={`${CIRCUMFERENCE * 0.25} ${CIRCUMFERENCE * 0.75}`}
            strokeLinecap="round"
            transform="rotate(-90 32 32)"
          />
        </svg>
      )}
      {isPlaying && (
        <svg
          className="absolute inset-0 w-full h-full"
          viewBox="0 0 64 64"
        >
          {/* 轨道 */}
          <circle
            cx="32" cy="32" r={RADIUS}
            fill="none"
            stroke="rgba(255,255,255,0.15)"
            strokeWidth="3"
          />
          {/* 进度 */}
          <circle
            cx="32" cy="32" r={RADIUS}
            fill="none"
            stroke="currentColor"
            className="text-bs-accent-purple"
            strokeWidth="3"
            strokeDasharray={CIRCUMFERENCE}
            strokeDashoffset={dashOffset}
            strokeLinecap="round"
            transform="rotate(-90 32 32)"
          />
        </svg>
      )}
    </button>
  );
}
