import { createContext, useContext, useState, useEffect, useRef, useCallback, ReactNode } from 'react';
import { loadUserPrefs, saveUserPrefs } from '../utils/prefs';

interface AudioContextType {
  volume: number;
  setVolume: (v: number) => void;
  playPreview: (url: string, id?: string) => Promise<void>;
  stopPreview: () => void;
  audioRef: React.MutableRefObject<HTMLAudioElement | null>;
  currentPlayingId: string | null;
}

const AudioContext = createContext<AudioContextType>({
  volume: 1,
  setVolume: () => {},
  playPreview: async () => {},
  stopPreview: () => {},
  audioRef: { current: null },
  currentPlayingId: null,
});

export function AudioProvider({ children }: { children: ReactNode }) {
  const [volume, setVolumeState] = useState(1);
  const [isLoaded, setIsLoaded] = useState(false);
  const [currentPlayingId, setCurrentPlayingId] = useState<string | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  // 初始化：加载保存的音量
  useEffect(() => {
    loadUserPrefs().then(prefs => {
      setVolumeState(prefs.volume);
      setIsLoaded(true);
    });
  }, []);

  // Create global Audio singleton on mount
  useEffect(() => {
    const audio = new Audio();
    audioRef.current = audio;

    // 监听播放结束事件，自动清除 currentPlayingId
    const handleEnded = () => {
      setCurrentPlayingId(null);
    };
    audio.addEventListener('ended', handleEnded);

    return () => {
      audio.removeEventListener('ended', handleEnded);
      audio.pause();
      audio.src = '';
      audioRef.current = null;
    };
  }, []);

  // 音量变化时更新 audio 元素并保存
  const setVolume = useCallback(async (v: number) => {
    setVolumeState(v);
    if (audioRef.current) {
      audioRef.current.volume = v;
    }
    // 保存到 JSON 文件
    const prefs = await loadUserPrefs();
    prefs.volume = v;
    await saveUserPrefs(prefs);
  }, []);

  const playPreview = useCallback(async (url: string, id?: string) => {
    const audio = audioRef.current;
    if (!audio) return;
    // If same URL and already playing, toggle pause
    if (audio.src === url && !audio.paused) {
      audio.pause();
      setCurrentPlayingId(null);
      return;
    }
    audio.src = url;
    audio.volume = volume;
    setCurrentPlayingId(id ?? url);
    await audio.play().catch(() => {
      setCurrentPlayingId(null);
    });
  }, [volume]);

  const stopPreview = useCallback(() => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.pause();
    audio.currentTime = 0;
    setCurrentPlayingId(null);
  }, []);

  // 等待加载完成再渲染子组件，确保音量值正确
  if (!isLoaded) {
    return null;
  }

  return (
    <AudioContext.Provider value={{ volume, setVolume, playPreview, stopPreview, audioRef, currentPlayingId }}>
      {children}
    </AudioContext.Provider>
  );
}

export function useAudio() {
  return useContext(AudioContext);
}
