import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';

import { EditorialBrief } from '../src/results/EditorialBrief.js';

afterEach(cleanup);

it('searches the selected run and submits a new brief only on confirmation', async () => {
  const run = { projectId: 'project-a', sourceId: 'source-b', jobId: 'run-c' };
  const findMoments = vi.fn(async () => [
    { units: 's2-s4', start: 14, end: 27, text: 'We had priced it by the seat.' },
  ]);
  const reprompt = vi.fn(async () => 'job-new');
  const onStarted = vi.fn();
  render(
    <EditorialBrief
      api={{ findMoments, reprompt }}
      run={run}
      prompt="Pricing"
      defaults={{ minSeconds: 20, maxSeconds: 90, count: 5 }}
      onStarted={onStarted}
    />,
  );
  fireEvent.click(screen.getByRole('button', { name: 'Find moments' }));
  await screen.findByText(/priced it by the seat/);
  expect(findMoments).toHaveBeenCalledWith(run, 'Pricing');
  expect(reprompt).not.toHaveBeenCalled();
  fireEvent.change(screen.getByLabelText('Maximum seconds'), { target: { value: '40' } });
  fireEvent.click(screen.getByRole('button', { name: 'Make clips from brief' }));
  await waitFor(() => expect(onStarted).toHaveBeenCalledWith('job-new'));
  expect(reprompt).toHaveBeenCalledWith(run, 'Pricing', { minSeconds: 20, maxSeconds: 40 });
});
