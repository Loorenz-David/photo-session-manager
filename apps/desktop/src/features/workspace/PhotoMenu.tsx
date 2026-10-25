import type { ReactNode } from 'react';
import { ContextMenu, DropdownMenu } from 'radix-ui';
import { MoreHorizontal } from 'lucide-react';
import type { Photo } from '@/services/photos';

export type PhotoMenuAction = 'toggle' | 'assign' | 'move' | 'copy' | 'delete';

type Entry = { action: PhotoMenuAction; label: string; hint?: string; danger?: boolean };

/** Entries of the ⋯ menu and the right-click menu (COMPONENTS.md §Workspace: Photo ⋯ menu / right-click). */
function entries(photo: Photo): Entry[] {
  return [
    photo.classification === 'condition'
      ? { action: 'toggle', label: 'Move to Main', hint: 'A' }
      : { action: 'toggle', label: 'Move to Condition', hint: 'C' },
    { action: 'assign', label: 'Assign to product…' },
    { action: 'move', label: 'Move to session…' },
    { action: 'copy', label: 'Copy to session…' },
    { action: 'delete', label: 'Delete', hint: '⌫', danger: true },
  ];
}

type MenuProps = { photo: Photo; onAction: (action: PhotoMenuAction, photo: Photo) => void };

/** The ⋯ button on the stage bar: actions for the active photo. */
export function PhotoMenuButton({ photo, onAction }: MenuProps) {
  return (
    <DropdownMenu.Root modal={false}>
      <DropdownMenu.Trigger className="stage-icon-button" aria-label="Photo actions">
        <MoreHorizontal />
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content className="photo-menu" align="center" sideOffset={6}>
          {entries(photo).map(entry => (
            <DropdownMenu.Item
              key={entry.action}
              className="photo-menu-item"
              data-danger={entry.danger || undefined}
              onSelect={() => onAction(entry.action, photo)}
            >
              <span>{entry.label}</span>
              {entry.hint && <kbd>{entry.hint}</kbd>}
            </DropdownMenu.Item>
          ))}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}

/** Right-click menu around a tile. */
export function PhotoContextMenu({ photo, onAction, children }: MenuProps & { children: ReactNode }) {
  return (
    <ContextMenu.Root modal={false}>
      <ContextMenu.Trigger asChild>{children}</ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content className="photo-menu">
          {entries(photo).map(entry => (
            <ContextMenu.Item
              key={entry.action}
              className="photo-menu-item"
              data-danger={entry.danger || undefined}
              onSelect={() => onAction(entry.action, photo)}
            >
              <span>{entry.label}</span>
              {entry.hint && <kbd>{entry.hint}</kbd>}
            </ContextMenu.Item>
          ))}
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
}
