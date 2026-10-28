import { useEffect, useRef } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

/** Coalesce concurrent background completions into a single visible refresh. */
export function useDataRefresh(events: string[], refresh: () => unknown) {
  const refreshRef = useRef(refresh);
  refreshRef.current = refresh;
  const eventKey = events.join('|');
  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const listeners: UnlistenFn[] = [];
    for (const event of eventKey.split('|')) {
      listen(event, () => {
        if (cancelled) return;
        clearTimeout(timer);
        timer = setTimeout(() => {
          if (!cancelled) Promise.resolve(refreshRef.current()).catch(console.error);
        }, 250);
      }).then(unlisten => {
        if (cancelled) unlisten();
        else listeners.push(unlisten);
      }).catch(console.error);
    }
    return () => {
      cancelled = true;
      clearTimeout(timer);
      listeners.forEach(unlisten => unlisten());
    };
  }, [eventKey]);
}
