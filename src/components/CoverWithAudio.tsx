import { useState, useEffect, useRef } from 'react';
import { Map } from '../types';
import { useAudio } from '../contexts/AudioContext';

const RADIUS = 26;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

interface CoverWithAudioProps {
  map: Map;
}

export default function CoverWithAudio({ map }: CoverWithAudioProps) {
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(0);
  const { playPreview, stopPreview, audioRef } = useAudio();
  const isOwnRef = useRef(false);

  useEffect(() => {
    return () => {
      // Stop playback when this component unmounts or preview_url changes
      if (isOwnRef.current) {
        stopPreview();
        setPlaying(false);
        setProgress(0);
        isOwnRef.current = false;
      }
    };
  }, [map.preview_url, stopPreview]);

  // Track progress on the global audio element
  useEffect(() => {
    if (!playing) return;

    const audio = audioRef.current;
    if (!audio) return;

    const handleTimeUpdate = () => {
      if (audio.duration) {
        setProgress(audio.currentTime / audio.duration);
      }
    };
    const handleEnded = () => {
      setPlaying(false);
      setProgress(0);
      isOwnRef.current = false;
    };

    audio.addEventListener('timeupdate', handleTimeUpdate);
    audio.addEventListener('ended', handleEnded);

    return () => {
      audio.removeEventListener('timeupdate', handleTimeUpdate);
      audio.removeEventListener('ended', handleEnded);
    };
  }, [playing, audioRef]);

  function handleClick() {
    if (!map.preview_url) return;
    if (playing) {
      stopPreview();
      setPlaying(false);
      setProgress(0);
      isOwnRef.current = false;
    } else {
      // Stop any other component's playback first
      stopPreview();
      playPreview(map.preview_url);
      setPlaying(true);
      isOwnRef.current = true;
    }
  }

  if (!map.cover_url) return null;

  const dashOffset = CIRCUMFERENCE * (1 - progress);

  return (
    <button
      onClick={handleClick}
      className="relative w-16 h-16 rounded-lg overflow-hidden flex-shrink-0 cursor-pointer group/cover"
      title={map.preview_url ? '点击播放预览' : '暂无预览音频'}
    >
      <img
        src={map.cover_url}
        alt={map.song_name}
        className={`w-full h-full object-cover transition-opacity ${playing ? 'opacity-70' : 'group-hover/cover:opacity-80'}`}
      />
      {playing && (
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
