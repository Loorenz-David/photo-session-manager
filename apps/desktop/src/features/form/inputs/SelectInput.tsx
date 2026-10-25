import { useState } from 'react';
import { Check, ChevronDown } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Command, CommandEmpty, CommandInput, CommandItem, CommandList } from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import type { FieldInputProps } from '../MetafieldInput';

/** `select`: many or long options in a dropdown, searchable when there are more than 8. */
export function SelectInput({ id, label, definition, value, onChange }: FieldInputProps) {
  const [open, setOpen] = useState(false);
  const current = typeof value === 'string' ? value : '';
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button id={id} variant="outline" role="combobox" aria-label={label} aria-expanded={open} className="w-full justify-between font-normal">
          {current || <span className="text-muted-foreground">Select…</span>}
          <ChevronDown className="size-4 opacity-60" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-(--radix-popover-trigger-width) p-0" align="start">
        <Command>
          {definition.choices.length > 8 && <CommandInput placeholder="Search…" />}
          <CommandList>
            <CommandEmpty>No match.</CommandEmpty>
            {definition.choices.map(choice => (
              <CommandItem
                key={choice}
                value={choice}
                onSelect={() => {
                  onChange(choice === current ? '' : choice);
                  setOpen(false);
                }}
              >
                <Check className={choice === current ? 'size-4' : 'size-4 opacity-0'} />
                {choice}
              </CommandItem>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
