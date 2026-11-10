import { commands, type EditorStylesDto } from './bindings';

export type EditorStyles = EditorStylesDto;

/** The styles Capture One lists (the photographer's own first, then built-in), or null when they cannot be read (P63). */
export async function listStyles(): Promise<EditorStyles | null> {
  try {
    const result = await commands.editorStyles();
    return result.status === 'ok' ? result.data : null;
  } catch {
    return null;
  }
}
