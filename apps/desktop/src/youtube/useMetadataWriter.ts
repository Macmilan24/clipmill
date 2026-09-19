import { useEffect, useRef, useState } from 'react';
import type {
  PublishingApi,
  YoutubeMetadataDraft,
  YoutubeMetadataGenerationAction,
} from '../daemon/publishing.js';
import { publishingError } from './model.js';

export function useMetadataWriter(
  api: PublishingApi,
  exportJobId: string,
  revision: number,
  renderArtifactId: string,
  initial: YoutubeMetadataDraft,
) {
  const [draft, setDraft] = useState(initial);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const live = useRef(true);
  const sequence = useRef(0);
  const operating = useRef(false);
  useEffect(() => {
    live.current = true;
    return () => {
      live.current = false;
      sequence.current += 1;
    };
  }, []);

  const state = draft.generationState || 'idle';
  const active = state === 'queued' || state === 'running';
  const jobId = draft.generationJobId || '';

  useEffect(() => {
    if (!active || !jobId || pending) return;
    let stopped = false;
    const mine = sequence.current;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const value = await api.draftYoutubeMetadata(exportJobId, revision, 'status', jobId);
        if (stopped || !live.current || mine !== sequence.current) return;
        validate(value, revision, renderArtifactId);
        if (value.generationJobId !== jobId)
          throw new Error('The writing job changed. Refresh its saved status before continuing.');
        setDraft(value);
        setError(null);
        if (value.generationState === 'queued' || value.generationState === 'running')
          timer = setTimeout(() => void poll(), 1000);
      } catch (cause) {
        if (!stopped && live.current && mine === sequence.current) setError(publishingError(cause));
      }
    };
    timer = setTimeout(() => void poll(), 1000);
    return () => {
      stopped = true;
      clearTimeout(timer);
    };
  }, [api, exportJobId, revision, renderArtifactId, active, jobId, pending, refresh]);

  const act = async (action: YoutubeMetadataGenerationAction) => {
    if (operating.current) return;
    operating.current = true;
    setPending(true);
    setError(null);
    const mine = ++sequence.current;
    try {
      const value = await api.draftYoutubeMetadata(
        exportJobId,
        revision,
        action,
        action === 'start' ? undefined : jobId,
      );
      if (!live.current || mine !== sequence.current) return;
      validate(value, revision, renderArtifactId);
      if ((action === 'status' || action === 'cancel') && value.generationJobId !== jobId)
        throw new Error('The writing job changed. Refresh its saved status before continuing.');
      setDraft(value);
    } catch (cause) {
      if (live.current && mine === sequence.current) setError(publishingError(cause));
    } finally {
      operating.current = false;
      if (live.current && mine === sequence.current) setPending(false);
    }
  };

  return {
    draft,
    state,
    active,
    pending,
    error,
    act,
    refresh: () => {
      setError(null);
      setRefresh((value) => value + 1);
    },
  };
}

function validate(value: YoutubeMetadataDraft, revision: number, renderArtifactId: string) {
  if (value.revision !== revision || value.renderArtifactId !== renderArtifactId)
    throw new Error(
      'This suggestion belongs to another rendered revision. Keep your current details and reopen the latest export.',
    );
  if (['queued', 'running'].includes(value.generationState || '') && !value.generationJobId)
    throw new Error(
      'The writing job was not identified. Your details are unchanged; refresh or try again.',
    );
}
