import { useEffect, useState } from 'react';
import { getAppPaths, getHealth } from '../../services/health';
import { ActionButton } from '@/components/ui/action-button';
import { SkeletonRows } from '@/components/pending/SkeletonRows';
import { Skeleton } from '@/components/ui/skeleton';
import { usePageData } from '@/lib/pending';
import { PageHeader } from '@/features/shell/PageHeader';
import { cameraFilesNotReceivedDismiss, cameraTakeBack } from '@/services/camera';
import { HelperLog } from './HelperLog';
import { toast } from 'sonner';
import { takeBackError } from '@/features/shell/CameraCard';

const names: Record<string, string> = {
  data_dir: 'Data directory', hooks_compiled: 'Capture One hooks', nudge_listening: 'Event listener',
  document: 'Capture One document', document_not_open: 'Capture One document', document_not_session: 'Capture One document',
  document_not_created: 'Capture One document', document_not_front: 'Capture One document',
  editor_not_installed: 'Capture One', editor_not_pro: 'Capture One', editor_automation_denied: 'Capture One', editor_not_running: 'Capture One', style_not_found: 'Style',
  hooks: 'Installed hooks', hooks_not_installed: 'Installed hooks', recipes: 'Recipes', recipe_missing: 'Recipe missing',
  capture_folder_not_set: 'Capture folder', camera: 'Camera', camera_disconnected: 'Camera',
  camera_connecting: 'Camera', camera_busy: 'Camera', camera_auth_refused: 'Camera',
  camera_fingerprint_mismatch: 'Camera', camera_fingerprint_unconfirmed: 'Camera',
  camera_phone_connected: 'Camera', camera_held: 'Camera', camera_store_destination_unverified: 'Camera',
  camera_files_not_received: 'Camera', camera_photo_session_uncertain: 'Camera',
  camera_support_not_installed: 'Camera support', camera_sidecar_unavailable: 'Camera helper',
  editor_not_following: 'Capture One follows',
  integration_token_invalid: 'Shopify token', condition_field_missing: 'Condition photos field',
};

export function HealthPage() {
  const { data, loading, error, reload } = usePageData(() => Promise.all([getHealth(), getAppPaths()]));
  const health = data?.[0];
  const paths = data?.[1];
  const [checkedAt, setCheckedAt] = useState<string | null>(null);
  useEffect(() => { if (health) setCheckedAt(new Date().toLocaleTimeString()); }, [health]);
  const refresh = async () => { await reload(); };
  const takeBack = async () => { try { const result = await cameraTakeBack(); toast.success(result.camera_held ? 'Camera back on the cable' : 'Capture One reopened; the camera is still held'); await reload(); } catch (e) { toast.error(takeBackError(e)); } };

  return <><PageHeader title={<h1 className="health-title">Health</h1>} escapeGoesBack><ActionButton variant="outline" action={refresh}>Retry checks</ActionButton></PageHeader><main className="health-shell">
    <section className="health-section" aria-label="System status"><h2>System status</h2>
      {loading ? <SkeletonRows n={4} /> : error ? <p role="alert">{String(error)}</p> : <div className="health-list">{health?.items.map((item, index) => <div className="health-row" key={`${item.code}-${index}`} data-ok={item.ok}>
        <div className="health-label"><span className="health-dot" data-ok={item.ok} />{names[item.code] ?? item.code}</div><div className="health-detail">{item.detail}{item.code.startsWith('document') && checkedAt && <span className="health-checked"> · checked {checkedAt}</span>}</div>
        {item.code === 'camera_files_not_received' && <ActionButton variant="outline" action={async () => { await cameraFilesNotReceivedDismiss(); await reload(); }}>Dismiss</ActionButton>}
        {item.code === 'camera_held' && <ActionButton variant="outline" action={takeBack}>Take the camera back</ActionButton>}
      </div>)}</div>}
      <HelperLog />
    </section>
    <section className="health-section" aria-label="Files"><h2>Files</h2><div className="health-list"><div className="health-row"><div className="health-label">Application data</div><div className="health-detail">{paths?.data_dir ?? <Skeleton className="h-4 w-48" />}</div></div></div></section>
  </main></>;
}

