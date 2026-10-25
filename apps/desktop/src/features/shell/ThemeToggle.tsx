import { useEffect, useState } from 'react';
import { Moon, Sun } from 'lucide-react';
import { Button } from '@/components/ui/button';

export type Theme = 'light' | 'dark';

// Until the settings file is reachable from the UI (P16 `settings_set`), the choice lives in localStorage.
const STORAGE_KEY = 'psm.theme';

function storedTheme(): Theme {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === 'dark' ? 'dark' : 'light';
  } catch {
    return 'light';
  }
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  root.classList.toggle('dark', theme === 'dark');
  root.dataset.theme = theme;
}

/** Current theme, applied as `.dark` / `data-theme` on `<html>` and persisted. */
export function useTheme(): [Theme, (theme: Theme) => void] {
  const [theme, setTheme] = useState<Theme>(storedTheme);
  useEffect(() => {
    applyTheme(theme);
    try {
      window.localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      // storage unavailable: the theme still applies for this run
    }
  }, [theme]);
  return [theme, setTheme];
}

export function ThemeToggle({ theme, onChange }: { theme: Theme; onChange: (theme: Theme) => void }) {
  const next = theme === 'dark' ? 'light' : 'dark';
  return (
    <Button variant="ghost" size="icon" aria-label={`Switch to ${next} theme`} title={`Switch to ${next} theme`} onClick={() => onChange(next)}>
      {theme === 'dark' ? <Sun /> : <Moon />}
    </Button>
  );
}
