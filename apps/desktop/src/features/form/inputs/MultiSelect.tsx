import { useState } from 'react';
import { Check, Plus, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { FieldInputProps } from '../MetafieldInput';

/** `multiselect`: many list options in a dropdown with checks; chosen values show as removable tags. */
export function MultiSelect({ id, label, definition, value, onChange }: FieldInputProps) {
  const [open, setOpen] = useState(false);
  const current = Array.isArray(value) ? (value as string[]) : [];
  const toggle = (choice: string) => onChange(current.includes(choice) ? current.filter(v => v !== choice) : [...current, choice]);
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {current.map(choice => (
        <span key={choice} className="inline-flex items-center gap-1 rounded-full bg-muted px-2.5 py-1 text-xs">
          {choice}
          <button type="button" aria-label={`Remove ${choice}`} onClick={() => toggle(choice)}>
            <X className="size-3" />
          </button>
        </span>
      ))}
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button id={id} variant="outline" size="sm" aria-label={`Add to ${label}`}>
            <Plus className="size-3.5" /> Add
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-64 p-0" align="start">
          <Command>
            <CommandInput placeholder="Search…" />
            <CommandList>
              <CommandEmpty>No match.</CommandEmpty>
              {definition.choices.map(choice => (
                <CommandItem key={choice} value={choice} onSelect={() => toggle(choice)}>
                  <Check className={current.includes(choice) ? 'size-4' : 'size-4 opacity-0'} />
                  {choice}
                </CommandItem>
              ))}
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </div>
  );
}
