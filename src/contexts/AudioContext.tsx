import { createContext, useContext, useState, ReactNode } from 'react';

interface AudioContextType {
  volume: number;
  setVolume: (v: number) => void;
}

const AudioContext = createContext<AudioContextType>({ volume: 1, setVolume: () => {} });

export function AudioProvider({ children }: { children: ReactNode }) {
  const [volume, setVolume] = useState(1);
  return (
    <AudioContext.Provider value={{ volume, setVolume }}>
      {children}
    </AudioContext.Provider>
  );
}

export function useAudio() {
  return useContext(AudioContext);
}
