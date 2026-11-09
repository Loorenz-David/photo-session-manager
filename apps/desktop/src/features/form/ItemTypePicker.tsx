import { useEffect, useState } from 'react';
import { listItemTypes } from '@/services/items';

export function useItemTypes(active: boolean) {
  const [types, setTypes] = useState<string[] | null>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    if (!active) return;
    let alive = true;
    setTypes(null);
    setFailed(false);
    listItemTypes().then(value => {
      if (alive) setTypes(value);
    }).catch(error => {
      if (alive && (error as { code?: string }).code !== 'no_integration') setFailed(true);
    });
    return () => { alive = false; };
  }, [active]);
  return { types, failed };
}

export function ItemTypePicker({ value, onChange, onBlur, types, failed, id }: {
  value: string; onChange: (value: string) => void; onBlur?: () => void; types: string[] | null; failed: boolean; id: string;
}) {
  return <div className="flex flex-col gap-1">
    <label htmlFor={id}>Item type</label>
    <input id={id} aria-label="Item type" list={`${id}-options`} value={value} onChange={event => onChange(event.target.value)} onBlur={onBlur} autoComplete="off" placeholder={types?.length ? 'Choose a shop type' : 'Type an item type'} />
    {types && <datalist id={`${id}-options`}>{types.map(type => <option key={type} value={type} />)}</datalist>}
    {failed && <small>Couldn't load the shop's types — type one</small>}
  </div>;
}
