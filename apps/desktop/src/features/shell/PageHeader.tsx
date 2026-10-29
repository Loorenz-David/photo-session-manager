import type { ReactNode } from 'react';
import { ArrowLeft } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useBackKeys } from '@/lib/keyboard';

type PageHeaderProps = {
  back?: { label: string; to: string };
  title: ReactNode;
  escapeGoesBack?: boolean;
  children?: ReactNode;
};

export function PageHeader({ back, title, escapeGoesBack = false, children }: PageHeaderProps) {
  useBackKeys(back?.to ?? null, escapeGoesBack);
  return <header className="app-header">
    {back && <Button variant="ghost" className="back-button" aria-label={`Back to ${back.label}`} onClick={() => { window.location.hash = back.to; }}><ArrowLeft /> {back.label}</Button>}
    {back ? <div className="header-identity">{title}</div> : title}
    <div className="header-actions">{children}</div>
  </header>;
}
