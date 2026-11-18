import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, expect, it } from 'vitest';
import { ResearchPanel } from '../ResearchPanel';

afterEach(cleanup);

it('a25_c27a_no_run_explains_when_research_starts', () => {
  render(<ResearchPanel status={{ reason: null, run: null, steps: [], fields_filled: 0, to_review: 0,
    can_run: true, wait_ends_at: null, queue: null }} fields={[]} focusKey={null} onFocus={() => undefined}
    onRun={async () => undefined} onAcceptAll={async () => undefined} onClose={() => undefined} />);
  expect(screen.getByText('Research starts when you press Arranged — next.')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Research' })).toBeInTheDocument();
});

it('a19_after_a_finished_run_the_panel_says_run_again_is_the_way_to_research_again', () => {
  render(<ResearchPanel status={{ reason: null, run: { id: 1, state: 'finished', trigger: 'arranged', stage_reached: null, error: null }, steps: [], fields_filled: 2, to_review: 0,
    can_run: true, wait_ends_at: null, queue: null } as never} fields={[]} focusKey={null} onFocus={() => undefined}
    onRun={async () => undefined} onAcceptAll={async () => undefined} onClose={() => undefined} />);
  expect(screen.getByText(/Research ran by itself once for this session\. Use Run again/)).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Run again' })).toBeInTheDocument();
  expect(screen.queryByText('Research starts when you press Arranged — next.')).not.toBeInTheDocument();
});
