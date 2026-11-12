import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { getSettings, saveSettings } from '@/services/settings';
import { getStorage, setMinFreeGb, type Storage } from '@/services/storage';
import { getRetentionDays, setRetentionDays } from '@/services/retention';

/** Bytes as the photographer reads them: GB with one decimal from 1 GB, else whole MB. */
export function formatBytes(bytes: number | null | undefined): string {
  const value = bytes ?? 0;
  return value >= 1e9 ? `${(value / 1e9).toFixed(1)} GB` : `${Math.round(value / 1e6)} MB`;
}

/** Re-reads the storage report every `everyMs` while mounted (the sizes are computed in the background). */
function useStorage(everyMs: number): [Storage | null, (value: Storage) => void] {
  const [storage, setStorage] = useState<Storage | null>(null);
  useEffect(() => {
    let alive = true;
    const load = () => { Promise.resolve().then(getStorage).then(value => { if (alive) setStorage(value); }).catch(() => undefined); };
    load();
    const timer = setInterval(load, everyMs);
    return () => { alive = false; clearInterval(timer); };
  }, [everyMs]);
  return [storage, setStorage];
}

/** Settings → Storage (10F §4): one line per kind of file, and the minimum free space in GB. */
export function StorageSection() {
  const [storage, setStorage] = useStorage(5000);
  const [minFree, setMinFree] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const shown = minFree ?? (storage ? String(storage.min_free_gb) : '');
  const save = async () => {
    const gb = Math.round(Number(shown));
    if (!Number.isFinite(gb) || gb < 0) { toast.error('Enter the minimum free space in GB'); return; }
    setBusy(true);
    try {
      setStorage(await setMinFreeGb(gb));
      setMinFree(null);
      toast.success(`Minimum free space set to ${gb} GB`);
    } catch (error) {
      toast.error(`Not saved: ${(error as Error).message ?? String(error)}`);
    } finally { setBusy(false); }
  };
  return <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Storage">
    <h2>Storage</h2>
    {!storage || !storage.computed ? <p>Calculating…</p> : <dl className="health-list">
      <div className="health-row"><dt className="health-label">Free space{storage.disk ? ` on ${storage.disk}` : ''}</dt><dd className="health-detail">{storage.free_bytes == null ? 'Unknown' : formatBytes(storage.free_bytes)}</dd></div>
      <div className="health-row"><dt className="health-label">RAW files</dt><dd className="health-detail">{formatBytes(storage.raws_bytes)}</dd></div>
      <div className="health-row"><dt className="health-label">Duplicates</dt><dd className="health-detail">{formatBytes(storage.duplicates_bytes)}</dd></div>
      <div className="health-row"><dt className="health-label">Capture One preview exports</dt><dd className="health-detail">{formatBytes(storage.preview_exports_bytes)}</dd></div>
      <div className="health-row"><dt className="health-label">Previews</dt><dd className="health-detail">{formatBytes(storage.preview_cache_bytes)}</dd></div>
      <div className="health-row"><dt className="health-label">Masters</dt><dd className="health-detail">{formatBytes(storage.masters_bytes)}</dd></div>
    </dl>}
    {storage?.warning && <p role="status">{storage.warning}</p>}
    <div className="flex flex-col gap-1">
      <label htmlFor="settings-min_free_gb">Minimum free space (GB)</label>
      <div className="flex gap-2">
        <Input id="settings-min_free_gb" type="number" min={0} value={shown} onChange={e => setMinFree(e.target.value)} />
        <Button type="button" variant="outline" disabled={busy || !storage} onClick={() => void save()}>Save minimum</Button>
      </div>
      <p className="health-detail">Below it, duplicates, Capture One&apos;s preview exports and masters are cleared, in that order, then the files of published items (oldest publish first), until there is enough space again. Unpublished photos are never cleared.</p>
    </div>
    <RetentionSetting />
    <PrepareMastersSetting />
  </section>;
}

/**
 * Changelog 47 (P69): `export.prepare_masters` — once a session is Ready, Capture One prepares its full-size masters in the
 * background while it is idle, so publishing starts with them ready. On by default.
 */
export function PrepareMastersSetting() {
  const [on, setOn] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let alive = true;
    Promise.resolve().then(getSettings)
      .then(settings => { if (alive) setOn(settings.export?.prepare_masters ?? true); })
      .catch(() => undefined);
    return () => { alive = false; };
  }, []);
  const change = async (next: boolean) => {
    setBusy(true);
    try {
      const saved = await saveSettings({ export: { prepare_masters: next } });
      setOn(saved.export?.prepare_masters ?? next);
      toast.success(next ? 'Masters are prepared in advance' : 'Masters are made when publishing');
    } catch (error) {
      toast.error(`Not saved: ${(error as Error).message ?? String(error)}`);
    } finally { setBusy(false); }
  };
  return <div className="flex flex-col gap-1">
    <div className="flex items-center gap-2">
      <Checkbox id="settings-prepare_masters" checked={on ?? false} disabled={busy || on === null}
        onCheckedChange={checked => void change(checked === true)} aria-label="Prepare masters in advance" />
      <label htmlFor="settings-prepare_masters">Prepare masters in advance</label>
    </div>
    <p className="health-detail">When a session is Ready, Capture One makes its full-size masters in the background while it has nothing else to do (never during a capture, a preview or another publish), so publishing starts sooner. They are deleted after publishing, like any master.</p>
  </div>;
}

/**
 * 10F §3 (P62): days a published, unchanged item keeps its local files (RAWs included) before they are cleared, once
 * Shopify is verified to hold every published image. Items marked "Keep files on this Mac" are never cleared.
 */
export function RetentionSetting() {
  const [days, setDays] = useState<number | null>(null);
  const [edited, setEdited] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let alive = true;
    Promise.resolve().then(getRetentionDays).then(value => { if (alive) setDays(value); }).catch(() => undefined);
    return () => { alive = false; };
  }, []);
  const shown = edited ?? (days === null ? '' : String(days));
  const save = async () => {
    const value = Math.round(Number(shown));
    if (shown.trim() === '' || !Number.isFinite(value) || value < 0) { toast.error('Enter the number of days'); return; }
    setBusy(true);
    try {
      setDays(await setRetentionDays(value));
      setEdited(null);
      toast.success(`Published items keep their files for ${value} ${value === 1 ? 'day' : 'days'}`);
    } catch (error) {
      toast.error(`Not saved: ${(error as Error).message ?? String(error)}`);
    } finally { setBusy(false); }
  };
  return <div className="flex flex-col gap-1">
    <label htmlFor="settings-retention_days">Keep files of published items (days)</label>
    <div className="flex gap-2">
      <Input id="settings-retention_days" type="number" min={0} value={shown} onChange={e => setEdited(e.target.value)} />
      <Button type="button" variant="outline" disabled={busy || days === null} onClick={() => void save()}>Save days</Button>
    </div>
    <p className="health-detail">After that many days, a published item that has not changed since is cleared from this Mac (RAW files included) once Shopify is checked to hold every published photo; its photos are then shown from Shopify. Items marked &quot;Keep files on this Mac&quot; are kept.</p>
  </div>;
}

/** The header's warning while nothing more can be cleared safely (10F §4). */
export function DiskWarning() {
  const [storage] = useStorage(30000);
  if (!storage?.warning) return null;
  return <span className="disk-warning" role="status">{storage.warning}</span>;
}
