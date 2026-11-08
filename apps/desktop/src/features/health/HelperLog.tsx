import { useState } from 'react';
import { cameraHelperLog } from '@/services/camera';

export function HelperLog() {
  const [lines, setLines] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  return <details onToggle={event => { if (event.currentTarget.open) void cameraHelperLog().then(setLines).catch(e => setError(String(e))); }}>
    <summary>Camera helper log</summary>
    {error && <p role="alert">{error}</p>}
    <pre className="whitespace-pre-wrap font-mono text-xs">{lines.join('\n')}</pre>
  </details>;
}
