import type { ReactNode } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { toast } from 'sonner';
import { cn } from '@/lib/utils';

/** Only web addresses leave the app (the opener capability allows http/https only). */
export function isWebUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'https:' || parsed.protocol === 'http:';
  } catch {
    return false;
  }
}

/** Opens a web address in the default browser (A6: the app's window ignores `target="_blank"`). */
export async function openExternal(url: string): Promise<void> {
  if (!isWebUrl(url)) return;
  try {
    await openUrl(url);
  } catch (error) {
    toast.error(`Could not open the link: ${(error as Error).message ?? String(error)}`);
  }
}

/** A link to a web page outside the app: opens in the default browser; anything else is shown as plain text. */
export function ExternalLink({ href, className, children }: { href: string; className?: string; children?: ReactNode }) {
  if (!isWebUrl(href)) return <span className={className}>{children ?? href}</span>;
  return (
    <a
      href={href}
      rel="noreferrer"
      title={href}
      className={cn('cursor-pointer underline underline-offset-2', className)}
      onClick={event => {
        event.preventDefault();
        void openExternal(href);
      }}
    >
      {children ?? href}
    </a>
  );
}
