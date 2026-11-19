import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { ChoiceSelect } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { listDefinitions, type Definition } from '@/services/attributes';
import type { ResearchSettings } from '@/services/settings';
import { writerSettingsExport, writerSettingsGet, writerSettingsImport, writerSettingsSave, type WriterSettings } from '@/services/writer';
import { WriterExamples } from './WriterExamples';
import { WriterInstruction } from './WriterInstruction';

/** The research model provider when none is chosen (`psm_research::config::DEFAULT_MODEL_PROVIDER`). */
const DEFAULT_MODEL_PROVIDER = 'openai';
const LANGUAGES = ['English', 'Swedish'];
/** The safeguard's finding types in the order of 08C §6, with the label shown. */
export const SAFEGUARD_TYPES = [
  { id: 'field_vs_photos', label: 'Field vs photos' },
  { id: 'field_vs_field', label: 'Field vs field' },
  { id: 'field_vs_listings', label: 'Field vs listings' },
  { id: 'missing_in_text', label: 'Missing in text' },
  { id: 'text_vs_facts', label: 'Text vs facts' },
] as const;
type SafeguardType = (typeof SAFEGUARD_TYPES)[number]['id'];
const MODES = [{ value: 'off', label: 'off' }, { value: 'note', label: 'note' }, { value: 'ask', label: 'ask' }];
const LENGTHS: { key: keyof WriterSettings['lengths']; label: string }[] = [
  { key: 'title_max', label: 'Title, at most (characters)' },
  { key: 'description_min_words', label: 'Description, at least (words)' },
  { key: 'description_max_words', label: 'Description, at most (words)' },
  { key: 'condition_max_words', label: 'Condition report, at most (words)' },
  { key: 'seo_title_max', label: 'Search title, at most (characters)' },
  { key: 'seo_description_min', label: 'Search description, at least (characters)' },
  { key: 'seo_description_max', label: 'Search description, at most (characters)' },
];
/** A23: the built-in parts the writer adds only when switched on (all off by default). */
export const SWITCHES: { key: 'grade_word' | 'stability_sentence' | 'title_formula' | 'dimensions_line'; label: string }[] = [
  { key: 'grade_word', label: 'Start the condition report with the grade word' },
  { key: 'stability_sentence', label: 'Always end the condition report with a stability sentence' },
  { key: 'title_formula', label: 'Send the built-in title formula' },
  { key: 'dimensions_line', label: 'End the description with a dimensions line' },
];
/** The texts the writer writes itself: never among the fields it may see (WC2). */
const OWN_TEXTS = ['product.title', 'product.description', 'product.seo_title', 'product.seo_description'];

function message(error: unknown): string {
  return error instanceof Error ? error.message : (error as { message?: string }).message ?? String(error);
}

/** The fields the writer may see when no list is saved (WC2): Details and Condition fields that are not prices, media or references. */
function seenByDefault(definition: Definition): boolean {
  return !['money', 'media_list', 'reference'].includes(definition.core_type);
}

/**
 * Settings → Writer (08C §8): the writer's model and effort, language, automatic writing, the instruction, the fields it
 * may see, style examples, extra banned words, target lengths, the safeguard and the shop name. Loaded when first opened;
 * **Save** sends the whole `writer` slot; the writer reads it at its next run.
 */
export function WriterSection({ research, active }: { research: ResearchSettings; active: boolean }) {
  const [settings, setSettings] = useState<WriterSettings | null>(null);
  const [banned, setBanned] = useState('');
  const [definitions, setDefinitions] = useState<Definition[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [opened, setOpened] = useState(active);

  useEffect(() => { if (active) setOpened(true); }, [active]);
  useEffect(() => {
    if (!opened) return;
    let alive = true;
    writerSettingsGet().then(loaded => {
      if (!alive) return;
      setSettings(loaded);
      setBanned(loaded.banned_extra.join('\n'));
    }).catch(failure => { if (alive) setError(message(failure)); });
    listDefinitions().then(fields => { if (alive) setDefinitions(fields); }).catch(() => undefined);
    return () => { alive = false; };
  }, [opened]);

  if (!opened) return null;
  if (!settings) return <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Writer"><h2>Writer</h2>{error && <p role="alert">{error}</p>}</section>;

  const edit = (patch: Partial<WriterSettings>) => setSettings(current => (current ? { ...current, ...patch } : current));
  const provider = research.providers.model.trim() || DEFAULT_MODEL_PROVIDER;
  const priced = [...new Set(research.prices.filter(row => row.provider === provider && row.model.trim() !== '').map(row => row.model))];
  const models = [...priced.map(model => ({ value: model, label: model })),
    ...(priced.includes(settings.model) ? [] : [{ value: settings.model, label: `${settings.model} (no price)` }])];
  const language = settings.language;
  const instruction = settings.instructions[language];
  const visibleFields = definitions.filter(d => (d.group === 'details' || d.group === 'condition')
    && !OWN_TEXTS.includes(d.key) && d.key !== settings.condition_report_key);
  const visible = settings.visible_keys ?? visibleFields.filter(seenByDefault).map(d => d.key);
  const toggleVisible = (key: string, on: boolean) => {
    const next = on ? [...visible, key] : visible.filter(k => k !== key);
    edit({ visible_keys: visibleFields.map(d => d.key).filter(k => next.includes(k)).concat(next.filter(k => !visibleFields.some(d => d.key === k))) });
  };
  const editInstruction = (text: string) => edit({ instructions: { ...settings.instructions, [language]: { ...instruction, text } } });
  const count = (type: SafeguardType) => settings.counts[type] ?? { answered: 0, dismissed: 0 };
  const number = (value: string) => Math.max(0, Math.round(Number(value) || 0));

  const save = async () => {
    setBusy(true);
    try {
      const banned_extra = banned.split('\n').map(word => word.trim()).filter(word => word !== '');
      const saved = await writerSettingsSave({ ...settings, banned_extra });
      setSettings(saved);
      setBanned(saved.banned_extra.join('\n'));
      setError(null);
      toast.success('Writer settings saved');
    } catch (failure) {
      setError(message(failure));
    } finally {
      setBusy(false);
    }
  };

  const loaded = (next: WriterSettings) => {
    setSettings(next);
    setBanned(next.banned_extra.join('\n'));
    setError(null);
  };
  const exportFile = async () => {
    try {
      if (await writerSettingsExport()) toast.success('Writer settings exported');
    } catch (failure) {
      setError(message(failure));
    }
  };
  const importFile = async () => {
    setBusy(true);
    try {
      const imported = await writerSettingsImport();
      if (imported) {
        loaded(imported);
        toast.success('Writer settings imported and saved');
      }
    } catch (failure) {
      setError(message(failure));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Writer">
      <h2>Writer</h2>

      <div role="group" aria-label="Export and import" className="flex flex-col gap-2">
        <p className="health-detail">Move the writer settings to another Mac: Export saves them to a file; Import reads such a file and saves it here (your earlier instructions stay among the earlier versions). Only the writer settings are in the file.</p>
        <div className="flex gap-2">
          <Button type="button" variant="outline" onClick={() => void exportFile()}>Export…</Button>
          <Button type="button" variant="outline" disabled={busy} onClick={() => void importFile()}>Import…</Button>
        </div>
      </div>

      <div role="group" aria-label="Model and effort" className="flex flex-col gap-2">
        <h3>Model and effort</h3>
        <p className="health-detail">The models with a price in Settings → Research.</p>
        <label htmlFor="writer-model">Model</label>
        <ChoiceSelect id="writer-model" className="w-full" value={settings.model} onValueChange={model => edit({ model })} choices={models} />
        <label htmlFor="writer-effort">Effort</label>
        <Input id="writer-effort" value={settings.effort} onChange={event => edit({ effort: event.target.value })} />
      </div>

      <div role="group" aria-label="Language" className="flex flex-col gap-2">
        <h3>Language</h3>
        <label htmlFor="writer-language">Writing language</label>
        <ChoiceSelect id="writer-language" className="w-full" value={language} onValueChange={next => edit({ language: next })}
          choices={LANGUAGES.map(name => ({ value: name, label: name }))} />
      </div>

      <label className="flex items-center gap-2">
        <Checkbox aria-label="Write texts automatically on Next to review" checked={settings.auto} onCheckedChange={value => edit({ auto: value === true })} />
        Write texts automatically on &quot;Next to review&quot;
      </label>

      <div role="group" aria-label="Built-in parts" className="flex flex-col gap-2">
        <h3>Built-in parts</h3>
        <p className="health-detail">Each is added to the instruction, and checked, only when ticked. With the grade word off, the writer never writes the grade.</p>
        {SWITCHES.map(item => (
          <label key={item.key} className="flex items-center gap-2">
            <Checkbox aria-label={item.label} checked={settings[item.key]} onCheckedChange={value => edit({ [item.key]: value === true })} />
            {item.label}
          </label>
        ))}
      </div>

      {instruction && <WriterInstruction language={language} instruction={instruction} onChange={editInstruction}
        onReset={saved => setSettings(current => (current ? { ...current, instructions: { ...current.instructions, [language]: saved.instructions[language] } } : current))} />}

      <div role="group" aria-label="Fields the writer may see" className="flex flex-col gap-2">
        <h3>Fields the writer may see</h3>
        <p className="health-detail">The Details and Condition fields the writer may read. Prices are left out unless you tick them.</p>
        {visibleFields.length === 0 ? <p className="health-detail">No fields to choose from yet.</p> : (
          <ul aria-label="Fields the writer may see" className="max-h-56 overflow-y-auto rounded-md border p-2">
            {visibleFields.map(field => (
              <li key={field.key} data-field={field.key} className="py-1">
                <label className="flex items-start gap-2">
                  <Checkbox aria-label={`${field.label} (${field.key})`} checked={visible.includes(field.key)} onCheckedChange={value => toggleVisible(field.key, value === true)} />
                  <span><strong>{field.label}</strong> <small className="health-detail">{field.key}</small></span>
                </label>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div role="group" aria-label="Style examples" className="flex flex-col gap-2">
        <h3>Style examples</h3>
        <p className="health-detail">Shown to the writer to follow their style; it never copies their sentences.</p>
        {LANGUAGES.map(name => (
          <WriterExamples key={name} language={name} examples={settings.examples[name] ?? []}
            onChange={examples => edit({ examples: { ...settings.examples, [name]: examples } })} />
        ))}
      </div>

      <div className="flex flex-col gap-1">
        <h3>Extra banned words</h3>
        <label htmlFor="writer-banned">Words and phrases never to use, one per line (added to the built-in list)</label>
        <Textarea id="writer-banned" rows={4} value={banned} onChange={event => setBanned(event.target.value)} />
      </div>

      <div role="group" aria-label="Target lengths" className="flex flex-col gap-2">
        <h3>Target lengths</h3>
        {LENGTHS.map(length => (
          <div key={length.key} className="flex flex-col gap-1">
            <label htmlFor={`writer-${length.key}`}>{length.label}</label>
            <Input id={`writer-${length.key}`} type="number" min={1} value={settings.lengths[length.key]}
              onChange={event => edit({ lengths: { ...settings.lengths, [length.key]: number(event.target.value) } })} />
          </div>
        ))}
      </div>

      <div role="group" aria-label="Safeguard" className="flex flex-col gap-2">
        <h3>Safeguard</h3>
        <p className="health-detail">What the writer does with each kind of doubt: nothing (off), a note to dismiss, or a question to answer.</p>
        <ul aria-label="Safeguard types" className="flex flex-col gap-2">
          {SAFEGUARD_TYPES.map(type => (
            <li key={type.id} data-safeguard={type.id} className="flex items-center gap-2">
              <span className="min-w-36">{type.label}</span>
              <ChoiceSelect aria-label={`${type.label} mode`} value={settings.safeguard[type.id]}
                onValueChange={mode => edit({ safeguard: { ...settings.safeguard, [type.id]: mode } })} choices={MODES} />
              <span className="health-detail" data-counts>answered {count(type.id).answered} · dismissed {count(type.id).dismissed}</span>
            </li>
          ))}
        </ul>
        <label htmlFor="writer-max_questions">Questions per write</label>
        <Input id="writer-max_questions" type="number" min={0} max={10} value={settings.max_questions}
          onChange={event => edit({ max_questions: number(event.target.value) })} />
      </div>

      <div className="flex flex-col gap-1">
        <h3>Shop name</h3>
        <label htmlFor="writer-shop_name">Shop name for search titles</label>
        <Input id="writer-shop_name" value={settings.shop_name} onChange={event => edit({ shop_name: event.target.value })} />
        <p className="health-detail">Added to a search title only when the title still fits in {settings.lengths.seo_title_max} characters.</p>
      </div>

      {error && <p role="alert">{error}</p>}
      <Button type="button" className="self-start" disabled={busy} onClick={() => void save()}>Save writer settings</Button>
    </section>
  );
}
