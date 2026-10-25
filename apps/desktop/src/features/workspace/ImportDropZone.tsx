import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { onFileDrop } from '@/services/events';
import { importPhotos, pickImportFiles } from '@/services/photos';

/** Sends files to the import path (ch06 §Import, MC-2): they arrive like captures, with `source = import`. */
export async function importFiles(paths: string[]): Promise<void> {
  if (paths.length === 0) return;
  try {
    await importPhotos(paths);
    toast(`Importing ${paths.length} ${paths.length === 1 ? 'file' : 'files'}`);
  } catch (error) {
    toast.error(`Import failed: ${(error as Error).message}`);
  }
}

/** Opens the file picker and imports what was chosen. */
export async function pickAndImport(): Promise<void> {
  try {
    await importFiles(await pickImportFiles());
  } catch (error) {
    toast.error(`Could not open the file picker: ${(error as Error).message}`);
  }
}

/** Files dropped from Finder onto the workspace window are imported; an overlay shows while they hover. */
export function ImportDropZone() {
  const [hovering, setHovering] = useState(false);
  useEffect(() => {
    let stop: (() => void) | undefined;
    let alive = true;
    onFileDrop(event => {
      if (event.type === 'drop') {
        setHovering(false);
        void importFiles(event.paths);
      } else {
        setHovering(event.type !== 'leave');
      }
    })
      .then(unlisten => {
        if (alive) stop = unlisten;
        else unlisten();
      })
      .catch(() => undefined);
    return () => {
      alive = false;
      stop?.();
    };
  }, []);
  if (!hovering) return null;
  return (
    <div className="import-overlay" aria-live="polite">
      Drop to import into this session
    </div>
  );
}
