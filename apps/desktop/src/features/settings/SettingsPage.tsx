import { useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { SkeletonFields } from '@/components/pending/SkeletonFields';
import { Input } from '@/components/ui/input';
import { useAction, usePageData } from '@/lib/pending';
import { getSettings, pickFolder, pickSessionFile, saveSettings, type IntegrationSettings, type Settings } from '@/services/settings';
import { PageHeader } from '@/features/shell/PageHeader';
import { CameraSection } from './CameraSection';

const emptyIntegration: IntegrationSettings = {
  kind: 'shopify', shop_domain: '', api_version: '2026-10', client_id: '', client_secret: '', condition_photos_key: 'custom.damage_report',
};

type TextKey = 'preview_recipe' | 'master_recipe' | 'style_name' | 'export_folder';
type IntegrationKey = 'shop_domain' | 'client_id' | 'client_secret' | 'condition_photos_key';

/** ch12 §Settings: a plain form over `settings.json` (undesigned in the handoff). The theme stays on the header toggle. */
export function SettingsPage() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const page = usePageData(getSettings);
  const loadedSecret = useRef('');
  useEffect(() => { if (page.data) setSettings(page.data); }, [page.data]);
  useEffect(() => { if (page.data) loadedSecret.current = page.data.integration?.client_secret ?? ''; }, [page.data]);

  const edit = (patch: Partial<Settings>) => setSettings(current => current ? { ...current, ...patch } : current);
  const editIntegration = (key: IntegrationKey, value: string) =>
    setSettings(current => current ? { ...current, integration: { ...(current.integration ?? emptyIntegration), [key]: value } } : current);

  const choose = useAction(async (key: 'export_folder', title: string) => {
    const folder = await pickFolder(title);
    if (folder) edit({ [key]: folder });
  }, e => setError(String(e)));
  const chooseSession = useAction(async () => {
    const file = await pickSessionFile('Use another session');
    if (file) edit({ document_id: file.slice(0, file.lastIndexOf('/')) });
  }, e => setError(String(e)));

  const save = useAction(async () => {
    if (!settings) return;
    const shopify = settings.integration;
    // An untouched, empty Shopify block is not saved (no integration stays no integration).
    const integration = shopify && [shopify.shop_domain, shopify.client_id, shopify.client_secret].some(v => v.trim() !== '') ? shopify : undefined;
    const saved = await saveSettings({
      document_id: settings.document_id ?? '',
      preview_recipe: settings.preview_recipe,
      output_format: settings.output_format,
      style_name: settings.style_name,
      default_shape: settings.default_shape,
      master_recipe: settings.master_recipe,
      export_folder: settings.export_folder,
      preview_delay_s: settings.preview_delay_s,
      camera: { ...settings.camera, auto_take_back: settings.camera.auto_take_back ?? true },
      integration,
    });
    setSettings(saved);
    loadedSecret.current = saved.integration?.client_secret ?? '';
    setError(null);
    toast.success('Settings saved');
  }, e => setError((e as {code?: string}).code === 'invalid_value' ? 'Ports must be 1024–65535, the passive range at least 2 ports and not containing the receiver port' : e instanceof Error ? e.message : String(e)));

  const header = <PageHeader title={<h1 className="health-title">Settings</h1>} escapeGoesBack>
    <Button variant="outline" onClick={() => { window.location.hash = '#/health'; }}>Health</Button>
  </PageHeader>;
  if (!settings) return <>{header}<main className="health-shell">{error || page.error ? <p role="alert">{error ?? String(page.error)}</p> : <SkeletonFields n={6} />}</main></>;

  const text = (label: string, key: TextKey, picker?: string) => <div className="flex flex-col gap-1">
    <label htmlFor={`settings-${key}`}>{label}</label>
    <div className="flex gap-2">
      <Input id={`settings-${key}`} value={settings[key] ?? ''} onChange={e => edit({ [key]: e.target.value })} />
      {picker && <Button type="button" variant="outline" disabled={choose.busy} aria-busy={choose.busy ? 'true' : undefined} onClick={() => void choose.run('export_folder', picker)}>Choose…{choose.busy ? '…' : ''}</Button>}
    </div>
  </div>;
  const shop = (label: string, key: IntegrationKey, type = 'text') => <div className="flex flex-col gap-1">
    <label htmlFor={`settings-${key}`}>{label}</label>
    <Input id={`settings-${key}`} type={type} autoComplete="off" value={settings.integration?.[key] ?? ''} onChange={e => editIntegration(key, e.target.value)}
      onFocus={key === 'client_secret' ? () => {
        const value = settings.integration?.client_secret ?? '';
        if (value.startsWith('••••') && value === loadedSecret.current) editIntegration('client_secret', '');
      } : undefined}
      onBlur={key === 'client_secret' ? () => {
        if (settings.integration?.client_secret === '' && loadedSecret.current.startsWith('••••')) editIntegration('client_secret', loadedSecret.current);
      } : undefined} />
  </div>;

  return <>{header}<main className="health-shell">
    <form onSubmit={event => { event.preventDefault(); void save.run(); }} className="health-section flex max-w-xl flex-col gap-4" aria-label="Settings">
      <h2>Capture One</h2>
      <div className="flex flex-col gap-1">
        <p>Capture One session</p>
        <p>{settings.document_id || 'Session Manager (created by the app)'}</p>
        <p className="health-detail">Effective path: {settings.document_id || settings.default_document_id}</p>
        <div className="flex gap-2">
          <Button type="button" variant="outline" disabled={chooseSession.busy} onClick={() => void chooseSession.run()}>Use another session…</Button>
          {settings.document_id && <Button type="button" variant="outline" onClick={() => edit({ document_id: null })}>Use the app&apos;s own session</Button>}
        </div>
      </div>
      {text('Preview recipe', 'preview_recipe')}
      {text('Master recipe', 'master_recipe')}
      {text('Style', 'style_name')}
      <CameraSection ports={settings.camera} onPortsChange={camera => edit({camera: {...camera, auto_take_back: camera.auto_take_back ?? true}})} />
      <div className="flex flex-col gap-1">
        <label htmlFor="settings-default_shape">Default shape</label>
        <select id="settings-default_shape" value={settings.default_shape} onChange={e => edit({ default_shape: e.target.value })}>
          <option value="original">Original</option>
          <option value="1:1">Square (1:1)</option>
          <option value="4:5">4:5</option>
          <option value="3:2">3:2</option>
          <option value="2:3">2:3</option>
          <option value="16:9">16:9</option>
          <option value="free">Free</option>
        </select>
      </div>
      <h2>Export and previews</h2>
      {text('Export folder', 'export_folder', 'Export folder')}
      <div className="flex flex-col gap-1">
        <label htmlFor="settings-output_long_edge">Output size (long edge, px)</label>
        <Input id="settings-output_long_edge" type="number" min={512} max={8192} value={settings.output_format.long_edge}
          onChange={e => edit({ output_format: { ...settings.output_format, long_edge: Math.round(Number(e.target.value) || 0) } })} />
      </div>
      <div className="flex flex-col gap-1">
        <label htmlFor="settings-output_quality">Output JPEG quality</label>
        <Input id="settings-output_quality" type="number" min={60} max={100} value={settings.output_format.quality}
          onChange={e => edit({ output_format: { ...settings.output_format, quality: Math.round(Number(e.target.value) || 0) } })} />
      </div>
      <div className="flex flex-col gap-1">
        <label htmlFor="settings-preview_delay_s">Preview refresh delay (seconds)</label>
        <Input id="settings-preview_delay_s" type="number" min={10} max={30} value={settings.preview_delay_s}
          onChange={e => edit({ preview_delay_s: Math.max(0, Math.round(Number(e.target.value) || 0)) })} />
        <p className="health-detail">In effect: {settings.effective_preview_delay_s} s (10–30 s)</p>
      </div>
      <h2>Shopify</h2>
      {shop('Shop domain', 'shop_domain')}
      {shop('Client ID', 'client_id')}
      {shop('Client secret', 'client_secret', 'password')}
      {shop('Condition photos key', 'condition_photos_key')}
      {error && <p role="alert">{error}</p>}
      <div><Button type="submit" disabled={save.busy} aria-busy={save.busy ? 'true' : undefined}>Save settings{save.busy ? '…' : ''}</Button></div>
    </form>
  </main></>;
}
