import type { ReactNode } from 'react';
import { ArrowLeft } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useBackKeys } from '@/lib/keyboard';
import { parentRoute, routeLabel } from '@/lib/routes';

type PageHeaderProps = {
  title: ReactNode;
  escapeGoesBack?: boolean;
  children?: ReactNode;
};

export function PageHeader({ title, escapeGoesBack = false, children }: PageHeaderProps) {
  const parent = parentRoute(window.location.hash);
  const label = parent === null ? null : routeLabel(parent);
  useBackKeys(parent, escapeGoesBack);
  return <header className="app-header">
    {parent && <Button variant="ghost" className="back-button" aria-label={`Back to ${label}`} onClick={() => { window.location.hash = parent; }}><ArrowLeft /> {label}</Button>}
    {parent ? <div className="header-identity">{title}</div> : title}
    <div className="header-actions">{children}</div>
  </header>;
}
