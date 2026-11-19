import { X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import type { WriterExample } from '@/services/writer';

const EMPTY: WriterExample = { name: '', fields: '', title: '', description: '', condition_report: '', seo_title: '', seo_description: '' };

const PARTS: { key: keyof WriterExample; label: string; long?: boolean }[] = [
  { key: 'name', label: 'Name' },
  { key: 'fields', label: 'Fields', long: true },
  { key: 'title', label: 'Title' },
  { key: 'description', label: 'Description', long: true },
  { key: 'condition_report', label: 'Condition report', long: true },
  { key: 'seo_title', label: 'Search title' },
  { key: 'seo_description', label: 'Search description', long: true },
];

/** Settings → Writer → Style examples of one language (08C §7, §8): shown to the writer as examples, never copied. */
export function WriterExamples({ language, examples, onChange }: {
  language: string;
  examples: WriterExample[];
  onChange: (examples: WriterExample[]) => void;
}) {
  const edit = (index: number, patch: Partial<WriterExample>) => onChange(examples.map((example, i) => (i === index ? { ...example, ...patch } : example)));
  return (
    <div role="group" aria-label={`${language} examples`} className="flex flex-col gap-2">
      <h4>{language}</h4>
      <ol className="flex flex-col gap-3">
        {examples.map((example, index) => (
          <li key={index} data-example={index} className="flex flex-col gap-1 rounded-md border p-2">
            <div className="flex items-center">
              <strong>Example {example.name || index + 1}</strong>
              <Button type="button" variant="ghost" size="icon-sm" className="ml-auto" aria-label={`Remove ${language} example ${index + 1}`}
                onClick={() => onChange(examples.filter((_, i) => i !== index))}><X /></Button>
            </div>
            {PARTS.map(part => {
              const label = `${language} example ${index + 1} ${part.label.toLowerCase()}`;
              return part.long
                ? <Textarea key={part.key} aria-label={label} placeholder={part.label} rows={3} value={example[part.key]} onChange={event => edit(index, { [part.key]: event.target.value })} />
                : <Input key={part.key} aria-label={label} placeholder={part.label} value={example[part.key]} onChange={event => edit(index, { [part.key]: event.target.value })} />;
            })}
          </li>
        ))}
      </ol>
      <Button type="button" variant="outline" className="self-start" onClick={() => onChange([...examples, { ...EMPTY }])}>Add {language} example</Button>
    </div>
  );
}
