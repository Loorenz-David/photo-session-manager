import type { ReactNode } from 'react';
import { ContextMenu, DropdownMenu } from 'radix-ui';
import { MoreHorizontal } from 'lucide-react';
import { canCompose, COMPOSE_WAITING } from '@/services/compositions';
import { useClipboardComposition } from '@/features/compose/useClipboardComposition';
import type { Photo } from '@/services/photos';

export type PhotoMenuAction = 'compose' | 'split' | 'normal' | 'apply' | 'toggle' | 'assign' | 'move' | 'copy' | 'delete';

type Entry = { action: PhotoMenuAction; label: string; hint?: string; danger?: boolean; disabled?: boolean };

/**
 * Entries of the ⋯ menu and the right-click menu (COMPONENTS.md §Workspace: Photo ⋯ menu / right-click; 10D: Compose…,
 * and Split / Normal — the one that changes the photo's kind, for Main and Condition photos with a frame).
 */
function entries(photo: Photo, canApply: boolean): Entry[] {
  const kindEntry: Entry =
    photo.composition.kind === 'split'
      ? { action: 'normal', label: 'Normal', hint: 'S', disabled: !canCompose(photo) }
      : { action: 'split', label: 'Split', hint: 'S', disabled: !canCompose(photo) };
  return [
    { action: 'compose', label: 'Compose…', hint: 'R', disabled: !canCompose(photo) },
    photo.classification === 'condition'
      ? { action: 'toggle', label: 'Move to Main', hint: 'A' }
      : { action: 'toggle', label: 'Move to Condition', hint: 'C' },
    kindEntry,
    ...(canApply ? [{ action: 'apply' as const, label: 'Apply composition to selected' }] : []),
    { action: 'assign', label: 'Assign to product…' },
    { action: 'move', label: 'Move to session…' },
    { action: 'copy', label: 'Copy to session…' },
    { action: 'delete', label: 'Deny', hint: '⌫', danger: true },
  ];
}

type MenuProps = { photo: Photo; onAction: (action: PhotoMenuAction, photo: Photo) => void; canApply?: boolean };

/** The ⋯ button on the stage bar: actions for the active photo. */
export function PhotoMenuButton({ photo, onAction, canApply = false }: MenuProps) {
  const hasClipboard = useClipboardComposition() !== null;
  return (
    <DropdownMenu.Root modal={false}>
      <DropdownMenu.Trigger className="stage-icon-button" aria-label="Photo actions">
        <MoreHorizontal />
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content className="photo-menu" align="center" sideOffset={6}>
          {entries(photo, canApply && hasClipboard).map(entry => (
            <DropdownMenu.Item
              key={entry.action}
              className="photo-menu-item"
              data-danger={entry.danger || undefined}
              disabled={entry.disabled}
              onSelect={() => onAction(entry.action, photo)}
            >
              <span>{entry.label}</span>
              {entry.action === 'compose' && entry.disabled ? <small className="photo-menu-why">{COMPOSE_WAITING}</small> : entry.hint && <kbd>{entry.hint}</kbd>}
            </DropdownMenu.Item>
          ))}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}

/** Right-click menu around a tile. */
export function PhotoContextMenu({ photo, onAction, children, canApply = false }: MenuProps & { children: ReactNode }) {
  const hasClipboard = useClipboardComposition() !== null;
  return (
    <ContextMenu.Root modal={false}>
      <ContextMenu.Trigger asChild>{children}</ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content className="photo-menu">
          {entries(photo, canApply && hasClipboard).map(entry => (
            <ContextMenu.Item
              key={entry.action}
              className="photo-menu-item"
              data-danger={entry.danger || undefined}
              disabled={entry.disabled}
              onSelect={() => onAction(entry.action, photo)}
            >
              <span>{entry.label}</span>
              {entry.action === 'compose' && entry.disabled ? <small className="photo-menu-why">{COMPOSE_WAITING}</small> : entry.hint && <kbd>{entry.hint}</kbd>}
            </ContextMenu.Item>
          ))}
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  );
}
