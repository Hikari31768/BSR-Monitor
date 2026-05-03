import { createContext, useContext, useState, useEffect, useRef, useCallback, ReactNode } from 'react';

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

const VOLUME_KEY = 'bsr-monitor-audio-volume';

export function AudioProvider({ children }: { children: ReactNode }) {
  const [volume, setVolumeState] = useState(() => {
    const saved = localStorage.getItem(VOLUME_KEY);
    return saved ? parseFloat(saved) : 1;
  });
  const [currentPlayingId, setCurrentPlayingId] = useState<string | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  // Create global Audio singleton on mount
  useEffect(() => {
    const audio = new Audio();
    audio.volume = volume;
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

  const setVolume = useCallback((v: number) => {
    setVolumeState(v);
    localStorage.setItem(VOLUME_KEY, v.toString());
    if (audioRef.current) {
      audioRef.current.volume = v;
    }
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

  return (
    <AudioContext.Provider value={{ volume, setVolume, playPreview, stopPreview, audioRef, currentPlayingId }}>
      {children}
    </AudioContext.Provider>
  );
}

export function useAudio() {
  return useContext(AudioContext);
}
