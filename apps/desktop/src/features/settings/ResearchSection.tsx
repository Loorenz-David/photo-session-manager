import { useEffect, useState } from 'react';
import { ArrowDown, ArrowUp, X } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { listDefinitions } from '@/services/attributes';
import { searchItems, type Item } from '@/services/items';
import { addResearchSite, exportResearch, removeResearchSite, reorderResearchSites, researchSecretsState, setResearchSecrets, type ResearchSecretsState } from '@/services/research';
import { pickFolder, saveResearchSettings, type ResearchSettings } from '@/services/settings';

/** The three research keys, by the account each is stored under (master_plan_research §3, §6.6). */
const KEYS = [
  { id: 'model', label: 'Model key', input: 'model_key' },
  { id: 'search', label: 'Visual search key', input: 'search_key' },
  { id: 'reader', label: 'Page reader key', input: 'reader_key' },
] as const;
type KeyId = (typeof KEYS)[number]['id'];
const NO_KEYS: Record<KeyId, string> = { model: '', search: '', reader: '' };

/** Shown under an attribution key that is not one of the shop's field keys. */
export const NOT_A_SHOP_FIELD = 'Not one of your shop’s fields';

function message(error: unknown): string {
  return error instanceof Error ? error.message : (error as { message?: string }).message ?? String(error);
}

/** Today's local date as an `<input type="date">` value (`YYYY-MM-DD`). */
function today(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** A local date `YYYY-MM-DD` as the `YYYYMMDD` number `research_export` takes. */
export function dateNumber(value: string): number {
  return Number(value.replaceAll('-', ''));
}

/**
 * Settings → Research → Export research (08A §10A; 08B MC-A11): the runs started in a range of local dates, or every
 * run of chosen items, written to a chosen folder as one zipped folder. The returned path is shown.
 */
function ExportResearch() {
  const [mode, setMode] = useState<'range' | 'items'>('range');
  const [from, setFrom] = useState(today);
  const [to, setTo] = useState(today);
  const [query, setQuery] = useState('');
  const [found, setFound] = useState<Item[]>([]);
  const [chosen, setChosen] = useState<Item[]>([]);
  const [busy, setBusy] = useState(false);
  const [path, setPath] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);

  const find = async () => {
    try {
      setFound(await searchItems(query.trim()));
    } catch (error) {
      setFailure(message(error));
    }
  };
  const toggle = (item: Item, on: boolean) =>
    setChosen(current => (on ? [...current.filter(i => i.id !== item.id), item] : current.filter(i => i.id !== item.id)));
  const ready = mode === 'range' ? from !== '' && to !== '' && from <= to : chosen.length > 0;

  const run = async () => {
    const folder = await pickFolder('Export research to');
    if (!folder) return;
    setBusy(true);
    try {
      const selection = mode === 'range' ? { kind: 'range' as const, from: dateNumber(from), to: dateNumber(to) } : { kind: 'items' as const, ids: chosen.map(item => item.id) };
      setPath(await exportResearch(selection, folder));
      setFailure(null);
    } catch (error) {
      setPath(null);
      setFailure(message(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-2" data-research-export>
      <h3>Export research</h3>
      <p className="health-detail">The research record of the chosen runs as one zipped folder: never a key, a RAW or a deleted photo.</p>
      <div role="radiogroup" aria-label="Export which runs" className="flex gap-4">
        <label className="flex items-center gap-2">
          <input type="radio" name="research-export-mode" checked={mode === 'range'} onChange={() => setMode('range')} /> Runs started between
        </label>
        <label className="flex items-center gap-2">
          <input type="radio" name="research-export-mode" checked={mode === 'items'} onChange={() => setMode('items')} /> Runs of chosen items
        </label>
      </div>
      {mode === 'range' ? (
        <div className="flex items-center gap-2">
          <Input aria-label="From" type="date" value={from} onChange={e => setFrom(e.target.value)} />
          <span>and</span>
          <Input aria-label="To" type="date" value={to} onChange={e => setTo(e.target.value)} />
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          <form className="flex gap-2" onSubmit={event => { event.preventDefault(); void find(); }}>
            <Input aria-label="Find items" placeholder="SKU or title" value={query} onChange={e => setQuery(e.target.value)} />
            <Button type="submit" variant="outline">Find</Button>
          </form>
          <ul aria-label="Items to export" className="flex flex-col gap-1">
            {[...chosen, ...found.filter(item => !chosen.some(c => c.id === item.id))].map(item => (
              <li key={item.id}>
                <label className="flex items-center gap-2">
                  <Checkbox aria-label={`Export ${item.sku}`} checked={chosen.some(c => c.id === item.id)} onCheckedChange={value => toggle(item, value === true)} />
                  {item.sku} · {item.title}
                </label>
              </li>
            ))}
          </ul>
        </div>
      )}
      <Button type="button" variant="outline" className="self-start" disabled={busy || !ready} onClick={() => void run()}>
        Export research
      </Button>
      {path && <p data-export-path>Exported to {path}</p>}
      {failure && <p role="alert" data-export-error>{failure}</p>}
    </div>
  );
}

type ResearchSectionProps = {
  /** The `research` block as loaded with the settings. */
  research: ResearchSettings;
  /** The connected shop's domain: always trusted, shown first (08A §6). */
  shopDomain: string | null;
};

/**
 * Settings → Research (08A §5, §6, §9): the keys (typed here, never read back), the trusted sites, model, effort and
 * image size per stage, cost limits, prices, language, automatic research, quiet period, record days, attribution keys.
 */
export function ResearchSection({ research: loaded, shopDomain }: ResearchSectionProps) {
  const [research, setResearch] = useState<ResearchSettings>(loaded);
  const [secrets, setSecrets] = useState<ResearchSecretsState | null>(null);
  const [typed, setTyped] = useState<Record<KeyId, string>>(NO_KEYS);
  const [site, setSite] = useState('');
  const [siteError, setSiteError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [shopKeys, setShopKeys] = useState<Set<string> | null>(null);
  const [busy, setBusy] = useState(false);

  const loadedText = JSON.stringify(loaded);
  useEffect(() => setResearch(JSON.parse(loadedText) as ResearchSettings), [loadedText]);
  useEffect(() => {
    let alive = true;
    researchSecretsState().then(state => { if (alive) setSecrets(state); }).catch(() => { if (alive) setSecrets({ model: false, search: false, reader: false }); });
    listDefinitions().then(definitions => { if (alive) setShopKeys(new Set(definitions.map(d => d.key))); }).catch(() => undefined);
    return () => { alive = false; };
  }, []);

  const edit = (patch: Partial<ResearchSettings>) => setResearch(current => ({ ...current, ...patch }));
  const sites = research.trusted_sites;

  /** A site command answers with the new list (saved by the command) or a refusal, shown as is; the list then stays. */
  const changeSites = async (change: () => Promise<string[]>) => {
    try {
      const next = await change();
      setResearch(current => ({ ...current, trusted_sites: next }));
      setSiteError(null);
      return true;
    } catch (failure) {
      setSiteError(message(failure));
      return false;
    }
  };
  const move = (index: number, by: -1 | 1) => {
    const next = [...sites];
    const [entry] = next.splice(index, 1);
    next.splice(index + by, 0, entry);
    void changeSites(() => reorderResearchSites(next));
  };

  const saveKeys = async () => {
    const input = { model_key: typed.model || null, search_key: typed.search || null, reader_key: typed.reader || null };
    if (!input.model_key && !input.search_key && !input.reader_key) return;
    setBusy(true);
    try {
      await setResearchSecrets(input);
      setTyped(NO_KEYS);
      setSecrets(await researchSecretsState());
      toast.success('Research keys saved');
    } catch (failure) {
      setError(message(failure));
    } finally {
      setBusy(false);
    }
  };

  const save = async () => {
    setBusy(true);
    try {
      const saved = await saveResearchSettings(research);
      if (saved) setResearch(saved);
      setError(null);
      toast.success('Research settings saved');
    } catch (failure) {
      setError(message(failure));
    } finally {
      setBusy(false);
    }
  };

  const stages = Object.entries(research.stages);
  const editStage = (name: string, patch: Partial<ResearchSettings['stages'][string]>) =>
    edit({ stages: { ...research.stages, [name]: { ...research.stages[name], ...patch } } });
  const editPrice = (index: number, patch: Partial<ResearchSettings['prices'][number]>) =>
    edit({ prices: research.prices.map((row, i) => (i === index ? { ...row, ...patch } : row)) });
  const optional = (value: string) => (value.trim() === '' ? null : value);

  return (
    <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Research">
      <h2>Research</h2>

      <h3>Keys</h3>
      <p className="health-detail">Keys are kept in the macOS Keychain and never shown again.</p>
      {KEYS.map(key => (
        <div key={key.id} className="flex flex-col gap-1">
          <label htmlFor={`research-${key.id}-key`}>{key.label}</label>
          <div className="flex items-center gap-2">
            <Input id={`research-${key.id}-key`} type="password" autoComplete="off" value={typed[key.id]} placeholder={secrets?.[key.id] ? 'Enter a new key to replace it' : ''}
              onChange={event => setTyped(current => ({ ...current, [key.id]: event.target.value }))} />
            <span className="health-detail" data-key-state={key.id}>{secrets === null ? '' : secrets[key.id] ? 'set' : 'not set'}</span>
          </div>
        </div>
      ))}
      <Button type="button" variant="outline" className="self-start" disabled={busy || Object.values(typed).every(v => v === '')} onClick={() => void saveKeys()}>
        Save keys
      </Button>

      <h3>Trusted sites</h3>
      <ol aria-label="Trusted sites" className="flex flex-col gap-1">
        {shopDomain && (
          <li data-site={shopDomain} className="flex items-center gap-2">
            <span>{shopDomain}</span>
            <span className="health-detail">your shop · always trusted</span>
          </li>
        )}
        {sites.map((entry, index) => (
          <li key={entry} data-site={entry} className="flex items-center gap-2">
            <span>{entry}</span>
            <span className="ml-auto flex gap-1">
              <Button type="button" variant="ghost" size="icon-sm" aria-label={`Move ${entry} up`} disabled={index === 0} onClick={() => move(index, -1)}><ArrowUp /></Button>
              <Button type="button" variant="ghost" size="icon-sm" aria-label={`Move ${entry} down`} disabled={index === sites.length - 1} onClick={() => move(index, 1)}><ArrowDown /></Button>
              <Button type="button" variant="ghost" size="icon-sm" aria-label={`Remove ${entry}`} onClick={() => void changeSites(() => removeResearchSite(entry))}><X /></Button>
            </span>
          </li>
        ))}
      </ol>
      <form className="flex gap-2" onSubmit={event => {
        event.preventDefault();
        const text = site;
        void changeSites(() => addResearchSite(text)).then(ok => { if (ok) setSite(''); });
      }}>
        <Input aria-label="Add a trusted site" placeholder="pamono.com or a page on it" value={site} onChange={event => setSite(event.target.value)} />
        <Button type="submit" variant="outline" disabled={site.trim() === ''}>Add site</Button>
      </form>
      {siteError && <p role="alert" data-site-error>{siteError}</p>}

      <h3>Stages</h3>
      <table className="text-sm">
        <thead><tr><th className="text-left">Stage</th><th className="text-left">Model</th><th className="text-left">Effort</th><th className="text-left">Image long edge (px)</th></tr></thead>
        <tbody>
          {stages.map(([name, stage]) => (
            <tr key={name}>
              <td>{name}</td>
              <td><Input aria-label={`${name} model`} value={stage.model} onChange={e => editStage(name, { model: e.target.value })} /></td>
              <td><Input aria-label={`${name} effort`} value={stage.effort} onChange={e => editStage(name, { effort: e.target.value })} /></td>
              <td><Input aria-label={`${name} image long edge`} type="number" min={1} value={stage.image_long_edge} onChange={e => editStage(name, { image_long_edge: Math.round(Number(e.target.value) || 0) })} /></td>
            </tr>
          ))}
        </tbody>
      </table>

      <h3>Cost limits</h3>
      <div className="flex flex-col gap-1">
        <label htmlFor="research-max_cost_per_run">Per run (USD)</label>
        <Input id="research-max_cost_per_run" inputMode="decimal" value={research.max_cost_per_run} onChange={e => edit({ max_cost_per_run: e.target.value })} />
      </div>
      <div className="flex flex-col gap-1">
        <label htmlFor="research-max_cost_per_day">Per day (USD)</label>
        <Input id="research-max_cost_per_day" inputMode="decimal" value={research.max_cost_per_day} onChange={e => edit({ max_cost_per_day: e.target.value })} />
      </div>

      <h3>Prices</h3>
      <table className="text-sm" aria-label="Prices">
        <thead><tr><th className="text-left">Provider</th><th className="text-left">Model</th><th className="text-left">Unit</th><th className="text-left">USD per million</th><th className="text-left">USD per call</th><th /></tr></thead>
        <tbody>
          {research.prices.map((row, index) => (
            <tr key={index}>
              <td><Input aria-label={`Price ${index + 1} provider`} value={row.provider} onChange={e => editPrice(index, { provider: e.target.value })} /></td>
              <td><Input aria-label={`Price ${index + 1} model`} value={row.model} onChange={e => editPrice(index, { model: e.target.value })} /></td>
              <td><Input aria-label={`Price ${index + 1} unit`} value={row.unit ?? ''} onChange={e => editPrice(index, { unit: optional(e.target.value) })} /></td>
              <td><Input aria-label={`Price ${index + 1} USD per million`} inputMode="decimal" value={row.usd_per_million ?? ''} onChange={e => editPrice(index, { usd_per_million: optional(e.target.value) })} /></td>
              <td><Input aria-label={`Price ${index + 1} USD per call`} inputMode="decimal" value={row.fallback_call_usd ?? ''} onChange={e => editPrice(index, { fallback_call_usd: optional(e.target.value) })} /></td>
              <td><Button type="button" variant="ghost" size="icon-sm" aria-label={`Remove price ${index + 1}`} onClick={() => edit({ prices: research.prices.filter((_, i) => i !== index) })}><X /></Button></td>
            </tr>
          ))}
        </tbody>
      </table>
      <Button type="button" variant="outline" className="self-start" onClick={() => edit({ prices: [...research.prices, { provider: '', model: '', unit: null, usd_per_million: null, fallback_call_usd: null }] })}>
        Add price row
      </Button>

      <h3>Behaviour</h3>
      <div className="flex flex-col gap-1">
        <label htmlFor="research-language">Language</label>
        <Input id="research-language" value={research.language} onChange={e => edit({ language: e.target.value })} />
      </div>
      <label className="flex items-center gap-2">
        <Checkbox aria-label="Automatic research" checked={research.auto} onCheckedChange={value => edit({ auto: value === true })} />
        Automatic research (when a session has a Main photo)
      </label>
      <div className="flex flex-col gap-1">
        <label htmlFor="research-quiet_seconds">Quiet period (seconds)</label>
        <Input id="research-quiet_seconds" type="number" value={research.quiet_seconds} onChange={e => edit({ quiet_seconds: Math.round(Number(e.target.value) || 0) })} />
      </div>
      <div className="flex flex-col gap-1">
        <label htmlFor="research-record_days">Keep research records (days)</label>
        <Input id="research-record_days" type="number" min={2} value={research.record_days} onChange={e => edit({ record_days: Math.round(Number(e.target.value) || 0) })} />
      </div>

      <h3>Attribution keys</h3>
      <p className="health-detail">The shop fields that name a designer or maker.</p>
      <ul aria-label="Attribution keys" className="flex flex-col gap-2">
        {research.attribution_keys.map((key, index) => (
          <li key={index} data-attribution={key} className="flex flex-col gap-1">
            <div className="flex gap-2">
              <Input aria-label={`Attribution key ${index + 1}`} value={key} onChange={e => edit({ attribution_keys: research.attribution_keys.map((k, i) => (i === index ? e.target.value : k)) })} />
              <Button type="button" variant="ghost" size="icon-sm" aria-label={`Remove attribution key ${key}`} onClick={() => edit({ attribution_keys: research.attribution_keys.filter((_, i) => i !== index) })}><X /></Button>
            </div>
            {shopKeys && key.trim() !== '' && !shopKeys.has(key.trim()) && <p className="text-xs text-destructive" data-attribution-warning>{NOT_A_SHOP_FIELD}</p>}
          </li>
        ))}
      </ul>
      <Button type="button" variant="outline" className="self-start" onClick={() => edit({ attribution_keys: [...research.attribution_keys, ''] })}>
        Add attribution key
      </Button>

      {error && <p role="alert">{error}</p>}
      <Button type="button" className="self-start" disabled={busy} onClick={() => void save()}>
        Save research settings
      </Button>

      <ExportResearch />
    </section>
  );
}
