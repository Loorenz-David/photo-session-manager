import { useCallback, useEffect, useState } from 'react';
import { getAppPaths, getHealth } from '../../services/health';
import type { AppPathsDto, HealthReportDto } from '../../services/bindings';
import { Button } from '../../components/ui/button';

const names: Record<string, string> = {
  data_dir: 'Data directory', hooks_compiled: 'Capture One hooks', nudge_listening: 'Event listener', capture_one: 'Capture One'
};

export function HealthPage() {
  const [health, setHealth] = useState<HealthReportDto | null>(null);
  const [paths, setPaths] = useState<AppPathsDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isTesting, setIsTesting] = useState(false);
  const [checkedAt, setCheckedAt] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    setIsTesting(true);
    try {
      const [report, appPaths] = await Promise.all([getHealth(), getAppPaths()]);
      setHealth(report); setPaths(appPaths); setError(null);
    } catch (e) { setError(String(e)); }
    finally {
      setCheckedAt(new Date().toLocaleTimeString());
      setIsTesting(false);
    }
  }, []);
  useEffect(() => { void refresh(); }, [refresh]);

  return <main className="health-shell">
    <header className="health-header"><div><h1 className="health-title">Session Manager</h1><p className="health-subtitle">Health</p></div><Button variant="outline" disabled={isTesting} onClick={() => void refresh()}>{isTesting ? 'Testing…' : 'Test Capture One'}</Button></header>
    <section className="health-section" aria-label="System status"><h2>System status</h2>
      {error ? <p role="alert">{error}</p> : <div className="health-list">{health?.items.map(item => <div className="health-row" key={item.code}>
        <div className="health-label"><span className="health-dot" data-ok={item.ok} />{names[item.code] ?? item.code}</div><div className="health-detail">{item.detail}{item.code === 'capture_one' && checkedAt && <span className="health-checked"> · checked {checkedAt}</span>}</div>
      </div>)}</div>}
    </section>
    <section className="health-section" aria-label="Files"><h2>Files</h2><div className="health-list"><div className="health-row"><div className="health-label">Application data</div><div className="health-detail">{paths?.data_dir ?? 'Loading…'}</div></div></div></section>
  </main>;
}

