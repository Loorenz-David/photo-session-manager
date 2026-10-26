import { useEffect, useState } from 'react';
import { getAppPaths, getHealth } from '../../services/health';
import { Button } from '../../components/ui/button';
import { ActionButton } from '@/components/ui/action-button';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { Skeleton } from '@/components/ui/skeleton';
import { usePageData } from '@/lib/pending';

const names: Record<string, string> = {
  data_dir: 'Data directory', hooks_compiled: 'Capture One hooks', nudge_listening: 'Event listener',
  document: 'Capture One document', document_not_open: 'Capture One document', document_not_session: 'Capture One document',
  hooks: 'Installed hooks', hooks_not_installed: 'Installed hooks', recipes: 'Recipes', recipe_missing: 'Recipe missing',
  capture_folder_not_set: 'Capture folder', camera: 'Camera', camera_disconnected: 'Camera',
  integration_token_invalid: 'Shopify token', condition_field_missing: 'Condition photos field',
};

export function HealthPage() {
  const { data, loading, error, reload } = usePageData(() => Promise.all([getHealth(), getAppPaths()]));
  const health = data?.[0];
  const paths = data?.[1];
  const [checkedAt, setCheckedAt] = useState<string | null>(null);
  useEffect(() => { if (health) setCheckedAt(new Date().toLocaleTimeString()); }, [health]);
  const refresh = async () => { await reload(); };

  return <main className="health-shell">
    <header className="health-header"><div><h1 className="health-title">Session Manager</h1><p className="health-subtitle">Health</p></div><div className="flex gap-2"><Button variant="outline" onClick={() => { window.location.hash = '#/'; }}>Sessions</Button><Button variant="outline" onClick={() => { window.location.hash = '#/settings'; }}>Settings</Button><ActionButton variant="outline" action={refresh}>Retry checks</ActionButton></div></header>
    <section className="health-section" aria-label="System status"><h2>System status</h2>
      {loading ? <SkeletonRows n={4} /> : error ? <p role="alert">{String(error)}</p> : <div className="health-list">{health?.items.map((item, index) => <div className="health-row" key={`${item.code}-${index}`} data-ok={item.ok}>
        <div className="health-label"><span className="health-dot" data-ok={item.ok} />{names[item.code] ?? item.code}</div><div className="health-detail">{item.detail}{item.code.startsWith('document') && checkedAt && <span className="health-checked"> · checked {checkedAt}</span>}</div>
      </div>)}</div>}
    </section>
    <section className="health-section" aria-label="Files"><h2>Files</h2><div className="health-list"><div className="health-row"><div className="health-label">Application data</div><div className="health-detail">{paths?.data_dir ?? <Skeleton className="h-4 w-48" />}</div></div></div></section>
  </main>;
}

