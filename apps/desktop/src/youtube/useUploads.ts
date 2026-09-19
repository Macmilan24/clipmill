import { useCallback, useEffect, useRef, useState } from 'react';
import type { PublishingApi, YoutubeUpload } from '../daemon/publishing.js';
import { newerUpload, publishingError, uploadActive } from './model.js';

export function useUploads(api: PublishingApi, projectId: string | null, docId: string | null) {
  const [uploads, setUploads] = useState<readonly YoutubeUpload[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshId, setRefreshId] = useState(0);
  const generation = useRef(0);
  const refresh = useCallback(() => {
    generation.current += 1;
    setRefreshId((value) => value + 1);
  }, []);
  const accept = useCallback(
    (record: YoutubeUpload) => {
      if (
        (projectId !== null && record.projectId !== projectId) ||
        (docId !== null && record.docId !== docId)
      )
        return;
      setUploads((previous) => [
        newerUpload(
          previous?.find((item) => item.uploadId === record.uploadId),
          record,
        ),
        ...(previous ?? []).filter((item) => item.uploadId !== record.uploadId),
      ]);
    },
    [projectId, docId],
  );
  useEffect(() => {
    setUploads(null);
    setError(null);
  }, [projectId, docId]);
  useEffect(() => {
    const mine = ++generation.current;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const read = async () => {
      try {
        const records = (await api.listYoutubeUploads(projectId ?? undefined)).filter(
          (item) =>
            (docId === null || item.docId === docId) &&
            (projectId === null || item.projectId === projectId),
        );
        if (mine !== generation.current) return;
        setUploads((previous) =>
          records.map((item) =>
            newerUpload(
              previous?.find((old) => old.uploadId === item.uploadId),
              item,
            ),
          ),
        );
        setError(null);
        if (records.some((item) => uploadActive(item.state)))
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
  }, [api, projectId, docId, refreshId]);
  return { uploads, error, refresh, accept };
}
