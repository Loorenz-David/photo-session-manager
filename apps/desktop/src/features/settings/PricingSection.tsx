import { useCallback, useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  conversationsKeySet,
  conversationsSettingsGet,
  conversationsSettingsSave,
  conversationsTest,
  type ConversationsConnection,
  type ConversationsSettings,
} from '@/services/conversations';
import { onConversationsChanged } from '@/services/events';
import { clock, connectionLine } from '@/features/conversations/connectionLine';

export { connectionLine };

/** How often the section re-reads the connection while it is open (the checks run every 10 s). */
const REFRESH_MS = 10_000;

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function code(error: unknown): string | undefined {
  return (error as { code?: string } | null)?.code;
}

/** "Last checked": the time of the last successful check, or the last error in words when it is newer. */
export function lastChecked(connection: ConversationsConnection): string | null {
  const okAt = connection.last_ok_at ?? null;
  const errorAt = connection.last_error_at ?? null;
  if (connection.last_error && errorAt !== null && (okAt === null || errorAt > okAt)) return connection.last_error;
  return okAt === null ? null : clock(okAt);
}

function Switch({ id, label, on, disabled, onChange }: { id: string; label: string; on: boolean; disabled: boolean; onChange: (on: boolean) => void }) {
  return <div className="flex items-center gap-2">
    <button id={id} type="button" role="switch" aria-checked={on} aria-label={label} disabled={disabled} data-on={on || undefined}
      className="toggle-switch" onClick={() => onChange(!on)}>{on ? 'On' : 'Off'}</button>
    <label htmlFor={id}>{label}</label>
  </div>;
}

/**
 * C1 Settings → Pricing (09B §4): the server address, the API key (kept in the Keychain, never shown again), Test connection,
 * the connection in words, the sound and banner switches and "Last checked" (re-read every 10 s and on `conversations_changed`).
 * Every control saves by itself, so the section is outside the Settings page's unsaved-changes guard (like the research keys).
 */
export function PricingSection() {
  const [settings, setSettings] = useState<ConversationsSettings | null>(null);
  const [server, setServer] = useState<string | null>(null);
  const [typedKey, setTypedKey] = useState('');
  const [replacing, setReplacing] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  // A28: each Test connection press shows "Checking…" and then the time of that press, also when the words are unchanged.
  const [testing, setTesting] = useState(false);
  const [checkedAt, setCheckedAt] = useState<number | null>(null);

  const load = useCallback(() => {
    Promise.resolve().then(conversationsSettingsGet).then(value => { setSettings(value); setProblem(null); }).catch(() => undefined);
  }, []);
  useEffect(() => {
    load();
    let unlisten: (() => void) | undefined;
    let alive = true;
    Promise.resolve().then(() => onConversationsChanged(() => load()))
      .then(stop => { if (alive) unlisten = stop; else stop(); })
      .catch(() => undefined);
    // A quiet check emits nothing (MC-C1.9); "Last checked" is re-read as often as the checks run.
    const timer = setInterval(load, REFRESH_MS);
    return () => { alive = false; unlisten?.(); clearInterval(timer); };
  }, [load]);

  /** Runs one save; a `provider` failure (Pricing unreachable or answering oddly) is shown in the connection line. */
  const run = async (work: () => Promise<ConversationsSettings>, after?: () => void) => {
    setBusy(true);
    setError(null);
    try {
      setSettings(await work());
      setProblem(null);
      after?.();
    } catch (failure) {
      if (code(failure) === 'provider') setProblem(message(failure));
      else setError(message(failure));
    } finally { setBusy(false); }
  };

  const test = async () => {
    setTesting(true);
    setCheckedAt(null);
    await run(conversationsTest);
    setCheckedAt(Date.now());
    setTesting(false);
  };

  if (!settings) return <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Pricing"><h2>Pricing</h2><p>Loading…</p></section>;

  const shownServer = server ?? settings.server;
  const line = problem ?? connectionLine(settings.connection);
  const checked = lastChecked(settings.connection);
  const keyField = !settings.key_saved || replacing;

  return <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Pricing">
    <h2>Pricing</h2>
    <div className="flex flex-col gap-1">
      <label htmlFor="pricing-server">Server address</label>
      <div className="flex gap-2">
        <Input id="pricing-server" value={shownServer} onChange={event => setServer(event.target.value)} />
        <Button type="button" variant="outline" disabled={busy || server === null}
          onClick={() => void run(() => conversationsSettingsSave({ server: shownServer }), () => setServer(null))}>Save address</Button>
      </div>
    </div>
    <div className="flex flex-col gap-1">
      <label htmlFor="pricing-key">API key</label>
      {keyField
        ? <div className="flex gap-2">
          <Input id="pricing-key" type="password" autoComplete="off" value={typedKey} onChange={event => setTypedKey(event.target.value)} />
          <Button type="button" variant="outline" disabled={busy || typedKey.trim() === ''}
            onClick={() => void run(() => conversationsKeySet(typedKey), () => { setTypedKey(''); setReplacing(false); })}>Save key</Button>
          {replacing && <Button type="button" variant="ghost" onClick={() => { setReplacing(false); setTypedKey(''); }}>Cancel</Button>}
        </div>
        : <div className="flex items-center gap-2">
          <span data-testid="pricing-key-state">Saved</span>
          <Button type="button" variant="outline" disabled={busy} onClick={() => setReplacing(true)}>Replace</Button>
          <Button type="button" variant="outline" disabled={busy} onClick={() => void run(() => conversationsKeySet(''))}>Remove</Button>
        </div>}
      <p className="health-detail">Kept in the macOS Keychain and never shown again.</p>
    </div>
    <div className="flex flex-col gap-1">
      <Button type="button" variant="outline" className="self-start" disabled={busy} onClick={() => void test()}>{testing ? 'Checking…' : 'Test connection'}</Button>
      {line && <p role="status" data-testid="pricing-connection">{line}</p>}
      {checkedAt !== null && !testing && <p className="health-detail" data-testid="pricing-checked-at">Checked at {clock(checkedAt)}</p>}
    </div>
    <Switch id="pricing-sound" label="Sound for new questions and replies" on={settings.sound} disabled={busy}
      onChange={sound => void run(() => conversationsSettingsSave({ sound }))} />
    <Switch id="pricing-banner" label="Mac banner when Session Manager is in the background" on={settings.banner} disabled={busy}
      onChange={banner => void run(() => conversationsSettingsSave({ banner }))} />
    <p className="health-detail">If no banner appears, allow notifications for Session Manager in System Settings → Notifications.</p>
    {checked && <p className="health-detail" data-testid="pricing-last-checked">Last checked: {checked}</p>}
    {error && <p role="alert">{error}</p>}
  </section>;
}
