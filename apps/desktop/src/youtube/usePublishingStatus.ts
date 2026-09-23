import { useCallback, useEffect, useRef, useState } from 'react';
import type { PublishingApi, YoutubePublishingStatus } from '../daemon/publishing.js';
import { publishingError } from './model.js';

/** Reads saved daemon state; only explicit actions contact Google. */
export function usePublishingStatus(api: PublishingApi) {
  const [status, setStatus] = useState<YoutubePublishingStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshId, setRefreshId] = useState(0);
  const generation = useRef(0);
  const refresh = useCallback(() => setRefreshId((value) => value + 1), []);
  useEffect(() => {
    const mine = ++generation.current;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const read = async () => {
      try {
        const next = await api.fetchYoutubePublishingStatus();
        if (mine !== generation.current) return;
        setStatus(next);
        setError(null);
        if (next.connections.some((connection) => connection.state === 'connecting'))
          timer = setTimeout(() => void read(), 1500);
      } catch (cause) {
        if (mine === generation.current) setError(publishingError(cause));
      }
    };
    void read();
    return () => {
      generation.current += 1;
      if (timer) clearTimeout(timer);
    };
  }, [api, refreshId]);
  return { status, error, refresh };
}
