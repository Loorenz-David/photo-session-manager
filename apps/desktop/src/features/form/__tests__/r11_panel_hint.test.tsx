import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, expect, it } from 'vitest';
import { ResearchPanel } from '../ResearchPanel';

afterEach(cleanup);

it('a25_c27a_no_run_explains_when_research_starts', () => {
  render(<ResearchPanel status={{ reason: null, run: null, steps: [], fields_filled: 0, to_review: 0,
    can_run: true, wait_ends_at: null }} fields={[]} focusKey={null} onFocus={() => undefined}
    onRun={async () => undefined} onAcceptAll={async () => undefined} onClose={() => undefined} />);
  expect(screen.getByText('Research starts when you press Arranged — next.')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Research' })).toBeInTheDocument();
});
