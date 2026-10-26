import type { ComponentProps, ReactNode } from 'react';
import { useAction } from '@/lib/pending';
import { Button } from './button';

type Props = Omit<ComponentProps<typeof Button>, 'onClick'> & {
  action: () => Promise<unknown>;
  onError?: (error: unknown) => void;
  busyLabel?: ReactNode;
};

export function ActionButton({ action, onError, busyLabel, children, disabled, ...props }: Props) {
  const pending = useAction(action, onError);
  return <Button {...props} disabled={disabled || pending.busy} aria-busy={pending.busy ? 'true' : undefined} onClick={() => void pending.run()}>
    {children}{pending.busy && (busyLabel ?? <span aria-hidden>…</span>)}
  </Button>;
}
