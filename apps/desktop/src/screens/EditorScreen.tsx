/**
 * The editor's container: find the document, fetch its plan, hand it over.
 *
 * The document is the newest one the project has, because approving a clip is
 * what creates one and the editor opens what you just approved. There is no
 * document picker in Phase 1's design, and inventing one here would be a
 * navigation surface nobody drew.
 */
import { useEffect, useState } from 'react';

import { type ShellApi, daemonApi } from '../daemon/api.js';
import type { PreviewPlan } from '../daemon/client.js';
import { Editor } from './Editor.js';

export interface EditorScreenProps {
  readonly onOpenResults: () => void;
  readonly api?: ShellApi;
}

const PROXY_FILE = 'proxy.mp4';
const PROXY_KIND = 'media.proxy.v1';

export function EditorScreen({ onOpenResults, api = daemonApi }: EditorScreenProps) {
  const [plan, setPlan] = useState<PreviewPlan | null>(null);
  const [docId, setDocId] = useState<string | null>(null);
  const [proxyUrl, setProxyUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const projects = await api.listProjects();
        const project = projects.at(-1);
        if (!project) {
          if (live) {
            setLoading(false);
          }
          return;
        }
        const docs = await api.listEditDocs(project.projectId);
        const newest = docs.at(-1);
        if (!newest) {
          if (live) {
            setLoading(false);
          }
          return;
        }
        const [fetched, jobs] = await Promise.all([
          api.previewPlan(project.projectId, newest.docId),
          api.listJobs(project.projectId).catch(() => []),
        ]);
        // The proxy the plan is previewed against comes off the analyze job
        // that published it, the same way the Inspector finds it.
        const proxy = jobs
          .flatMap((job) => job.tasks)
          .find((task) => task.outputKind === PROXY_KIND && task.outputArtifactId !== '');
        if (live) {
          setDocId(newest.docId);
          setPlan(fetched);
          setProxyUrl(
            proxy ? api.mediaUrl(project.projectId, proxy.outputArtifactId, PROXY_FILE) : null,
          );
          setLoading(false);
        }
      } catch (error) {
        if (live) {
          setProblem((error as Error).message);
          setLoading(false);
        }
      }
    })();
    return () => {
      live = false;
    };
  }, [api]);

  return (
    <Editor
      plan={plan}
      proxyUrl={proxyUrl}
      docId={docId}
      loading={loading}
      problem={problem}
      onOpenResults={onOpenResults}
    />
  );
}
