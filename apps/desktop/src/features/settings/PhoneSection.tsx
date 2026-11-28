import { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { saveSettings } from '@/services/settings';
import {
  phonePairingCreate, phonePairingRename, phonePairingRevoke, phonePairingsList, phoneServerStatus,
  type Pairing, type PairingCreated, type PhoneServerStatus,
} from '@/services/phone';

/** The line under the QR code (master_plan_live §5 PH, LV2·U3 task 5). */
export const PAIR_INSTRUCTIONS = 'Open the camera on the phone and scan this code. On iPhone, then Share → Add to Home Screen; on Android, add a shortcut and tap Full screen in the app.';

function serverLine(status: PhoneServerStatus): string {
  if (status.error) return `Phone server could not start: ${status.error}`;
  if (!status.enabled) return 'Phone server is off';
  if (status.running) return `Phone server on port ${status.port}`;
  return 'Phone server is not running';
}

function lastSeen(at: number | null): string {
  return at === null ? 'Never connected' : `Last seen ${new Date(at).toLocaleString()}`;
}

/** Settings → Phone (LV2, book 11 §Phone remote): pair a phone with a QR code, the pairings, the server's port and switch. */
export function PhoneSection() {
  const [status, setStatus] = useState<PhoneServerStatus | null>(null);
  const [pairings, setPairings] = useState<Pairing[]>([]);
  const [pairing, setPairing] = useState(false);
  const [name, setName] = useState('');
  const [created, setCreated] = useState<PairingCreated | null>(null);
  const [byAddress, setByAddress] = useState(false);
  const [renaming, setRenaming] = useState<{ id: number; name: string } | null>(null);
  const [removing, setRemoving] = useState<Pairing | null>(null);
  const [port, setPort] = useState('');
  const [enabled, setEnabled] = useState(true);
  const [busy, setBusy] = useState(false);

  const reload = useCallback(async () => {
    try {
      const [found, server] = await Promise.all([phonePairingsList(), phoneServerStatus()]);
      setPairings(found);
      setStatus(server);
      setPort(String(server.port));
      setEnabled(server.enabled);
    } catch (error) {
      toast.error(`Phone settings not read: ${(error as Error).message ?? String(error)}`);
    }
  }, []);
  useEffect(() => { void reload(); }, [reload]);
  useEffect(() => {
    let mounted = true;
    const timer = window.setInterval(() => {
      void phonePairingsList().then(found => { if (mounted) setPairings(found); }).catch(error => {
        if (mounted) toast.error(`Phone settings not read: ${(error as Error).message ?? String(error)}`);
      });
    }, 5_000);
    return () => { mounted = false; window.clearInterval(timer); };
  }, []);

  const create = async () => {
    setBusy(true);
    try {
      setCreated(await phonePairingCreate(name.trim() || null));
      setByAddress(false);
      setPairings(await phonePairingsList());
    } catch (error) {
      toast.error(`Not paired: ${(error as Error).message ?? String(error)}`);
    } finally { setBusy(false); }
  };
  const rename = async () => {
    if (!renaming || !renaming.name.trim()) return;
    try {
      await phonePairingRename(renaming.id, renaming.name.trim());
      setRenaming(null);
      setPairings(await phonePairingsList());
    } catch (error) {
      toast.error(`Not renamed: ${(error as Error).message ?? String(error)}`);
    }
  };
  const remove = async (target: Pairing) => {
    try {
      await phonePairingRevoke(target.id);
      setPairings(await phonePairingsList());
    } catch (error) {
      toast.error(`Not removed: ${(error as Error).message ?? String(error)}`);
    }
  };
  const saveServer = async () => {
    const value = Number(port);
    if (!Number.isInteger(value) || value < 1 || value > 65535) { toast.error('Enter a port number'); return; }
    setBusy(true);
    try {
      await saveSettings({ phone: { enabled, port: value } });
      setStatus(await phoneServerStatus());
      toast.success('Phone server saved');
    } catch (error) {
      toast.error(`Not saved: ${(error as Error).message ?? String(error)}`);
    } finally { setBusy(false); }
  };

  const qr = created && (byAddress ? created.qr_address_svg : created.qr_name_svg);
  const url = created && (byAddress ? created.url_address : created.url_name);
  return <section className="health-section flex max-w-xl flex-col gap-4" aria-label="Phone">
    <h2>Phone</h2>
    {status && <p role="status">{serverLine(status)}</p>}
    {!pairing && !created && <div><Button type="button" onClick={() => setPairing(true)}>Pair a phone</Button></div>}
    {pairing && !created && <div className="flex items-end gap-2">
      <label className="flex flex-col gap-1">
        <span>Name (optional)</span>
        <Input value={name} placeholder="Phone" onChange={e => setName(e.target.value)} />
      </label>
      <Button type="button" disabled={busy} onClick={() => void create()}>Create</Button>
    </div>}
    {created && qr && url && <div className="flex flex-col gap-2" data-testid="phone-pairing-code">
      <div aria-label={byAddress ? 'QR code of the address' : 'QR code of the Mac name'} className="w-60" dangerouslySetInnerHTML={{ __html: qr }} />
      <code className="break-all">{url}</code>
      <p>{PAIR_INSTRUCTIONS}</p>
      <div className="flex gap-2">
        {!byAddress && <Button type="button" variant="outline" onClick={() => setByAddress(true)}>Show the address instead</Button>}
        {byAddress && <Button type="button" variant="outline" onClick={() => setByAddress(false)}>Show the Mac name instead</Button>}
        <Button type="button" variant="outline" onClick={() => { setCreated(null); setPairing(false); setName(''); }}>Done</Button>
      </div>
    </div>}
    <ul aria-label="Paired phones" className="flex flex-col gap-2">
      {pairings.map(p => <li key={p.id} className="flex items-center gap-2" data-pairing={p.id}>
        <span aria-label={p.connected ? 'Connected' : 'Not connected'} data-connected={p.connected}
          className={`inline-block size-2 rounded-full ${p.connected ? 'bg-green-500' : 'bg-gray-300'}`} />
        {renaming?.id === p.id
          ? <>
            <Input aria-label={`Name of ${p.name}`} value={renaming.name} onChange={e => setRenaming({ id: p.id, name: e.target.value })} />
            <Button type="button" variant="outline" onClick={() => void rename()}>Save name</Button>
            <Button type="button" variant="ghost" onClick={() => setRenaming(null)}>Cancel</Button>
          </>
          : <>
            <span>{p.name}</span>
            <span className="text-muted-foreground">{p.connected ? 'Connected now' : lastSeen(p.last_seen_at)}</span>
            <Button type="button" variant="ghost" aria-label={`Rename ${p.name}`} onClick={() => setRenaming({ id: p.id, name: p.name })}>Rename</Button>
            <Button type="button" variant="ghost" aria-label={`Remove ${p.name}`} onClick={() => setRemoving(p)}>Remove</Button>
          </>}
      </li>)}
    </ul>
    <div className="flex items-end gap-4">
      <label className="flex flex-col gap-1">
        <span>Phone server port</span>
        <Input aria-label="Phone server port" inputMode="numeric" value={port} onChange={e => setPort(e.target.value)} className="w-28" />
      </label>
      <label className="flex items-center gap-2">
        <Checkbox aria-label="Phone server on" checked={enabled} onCheckedChange={v => setEnabled(v === true)} />
        <span>Phone server on</span>
      </label>
      <Button type="button" variant="outline" disabled={busy} onClick={() => void saveServer()}>Save phone server</Button>
    </div>
    <Dialog open={removing !== null} onOpenChange={open => { if (!open) setRemoving(null); }}>
      <DialogContent>
        <DialogTitle>Remove this phone?</DialogTitle>
        <DialogDescription>{removing?.name} will no longer connect. Pair it again with a new QR code.</DialogDescription>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => setRemoving(null)}>Cancel</Button>
          <Button type="button" onClick={() => { const target = removing; setRemoving(null); if (target) void remove(target); }}>Remove phone</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </section>;
}
