import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cameraFtpAddress, cameraFtpPasswordGenerate, cameraFtpPasswordGet, cameraSupportGet, cameraSupportInstall, cameraTestConnection } from '@/services/camera';
import type { CameraSettingsDto, CameraSupportDto, TestConnectionResponse } from '@/services/bindings';
import { pickArchive } from '@/services/settings';
import * as cameraSteps from './cameraSteps';
import { FingerprintBox } from './FingerprintBox';

/** The camera's Access Authentication login as typed; saved by Save settings (P68). */
export type CameraLogin = { user: string; password: string };
const EMPTY_LOGIN: CameraLogin = { user: '', password: '' };

/** How often step 3 re-reads this Mac's address (P68: the network can change while Settings is open). */
export const ADDRESS_REFRESH_MS = 5000;

/** The FTP password in groups of 4, as typed on the camera (P68). */
export const grouped = (password: string) => password.match(/.{1,4}/g)?.join(' ') ?? password;

export function CameraSection({ ports, onPortsChange, login: givenLogin, onLoginChange }: {
  ports: CameraSettingsDto; onPortsChange: (ports: CameraSettingsDto) => void;
  login?: CameraLogin; onLoginChange?: (login: CameraLogin) => void;
}) {
  const [support, setSupport] = useState<CameraSupportDto | null>(null);
  const [password, setPassword] = useState<string | null>(null);
  const [fresh, setFresh] = useState(false);
  const [ownLogin, setOwnLogin] = useState<CameraLogin>(EMPTY_LOGIN);
  const login = givenLogin ?? ownLogin;
  const setLogin = onLoginChange ?? setOwnLogin;
  const [address, setAddress] = useState<string | null>(null);
  const [check, setCheck] = useState<TestConnectionResponse | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState<'password' | 'address' | null>(null);
  useEffect(() => {
    let alive = true;
    Promise.resolve().then(cameraSupportGet).then(value => { if (alive) setSupport(value); }).catch(() => undefined);
    const readAddress = () => { Promise.resolve().then(() => cameraFtpAddress()).then(value => { if (alive) setAddress(value.address); }).catch(() => undefined); };
    readAddress();
    const timer = window.setInterval(readAddress, ADDRESS_REFRESH_MS);
    return () => { alive = false; window.clearInterval(timer); };
  }, []);
  const run = async (task: () => Promise<void>) => {
    setBusy(true);
    try { await task(); } catch (error) {
      const e = error as { code?: string; data?: Record<string, string>; message?: string };
      toast.error(e.code === 'support_archive_invalid' ? `The archive ${e.data?.file?.startsWith('changed ') ? 'has' : 'is'} ${e.data?.file}`
        : e.code === 'support_version_unsupported' ? `This SDK version (${e.data?.version}) is not supported; use 2.02.00`
        : e.message ?? String(error));
    } finally { setBusy(false); }
  };
  const copy = (text: string, what: 'password' | 'address') => void navigator.clipboard.writeText(text).then(() => { setCopied(what); window.setTimeout(() => setCopied(null), 1500); });
  const field = (label: string, key: 'receiver_port' | 'passive_port_start' | 'passive_port_end') => <label className="flex flex-col gap-1">{label}<Input type="number" min={1024} max={65535} value={ports[key]} onChange={e => onPortsChange({...ports, [key]: Number(e.target.value)})} /></label>;
  const steps = cameraSteps.ftp(address ?? "<this Mac's address on your network>", ports.receiver_port, 'sessionmanager');
  return <section className="flex flex-col gap-4" aria-label="Camera">
    <h2>Camera</h2>
    <p>Session Manager sets where the camera saves photos: card and computer on the cable, card only on Wi-Fi. The card always keeps a copy.</p>
    <label className="flex items-center gap-2"><input type="checkbox" checked={ports.auto_take_back ?? true} onChange={e => onPortsChange({...ports, auto_take_back: e.target.checked})} />Take the cable back from Capture One automatically when the camera is plugged in</label>
    <h3>Camera support</h3>
    {support ? <p>Sony Camera Remote SDK {support.version} installed</p> : <>
      <p>Session Manager can control a Sony camera directly. Download Sony&apos;s Camera Remote SDK for Mac from Sony&apos;s developer site (you accept Sony&apos;s terms there), then add it here.</p>
      <Button type="button" disabled={busy} onClick={() => void run(async () => { const path = await pickArchive(); if (!path) return; const installed = await cameraSupportInstall(path); setSupport(installed); toast.success(`Camera support for Sony ${installed.version} installed`); })}>Add camera support → Sony</Button>
    </>}
    {support && <>
      <h3>Transfer from the camera</h3><p>User: sessionmanager</p>
      <div className="flex gap-2">
        <Button type="button" disabled={busy} onClick={() => void run(async () => { const value = await cameraFtpPasswordGenerate(); setPassword(value.password); setFresh(true); })}>Generate password</Button>
        <Button type="button" variant="outline" disabled={busy} onClick={() => void run(async () => {
          const stored = await cameraFtpPasswordGet();
          if (stored) { setPassword(stored); setFresh(false); } else toast.error('No password yet: use Generate password');
        })}>Show password</Button>
      </div>
      {password && <div className="rounded border p-3">
        {fresh && <p>Enter this password on the camera now. You can show it again here later.</p>}
        <code data-testid="ftp-password">{grouped(password)}</code>
        <p className="health-detail">Type it without the spaces.</p>
        <Button type="button" onClick={() => copy(password, 'password')}>{copied === 'password' ? 'Copied' : 'Copy'}</Button>
      </div>}
      <ol className="list-decimal pl-5">{steps.map((step, index) => <li key={step}>{step}
        {index === 2 && address && <Button type="button" variant="outline" className="ml-2" onClick={() => copy(address, 'address')}>{copied === 'address' ? 'Copied' : 'Copy address'}</Button>}
      </li>)}</ol>
      <div className="grid grid-cols-3 gap-2">{field('Receiver port', 'receiver_port')}{field('Passive ports from', 'passive_port_start')}{field('Passive ports to', 'passive_port_end')}</div>
      <h3>Wi-Fi control</h3>
      <label>Access Authentication user<Input value={login.user} autoComplete="off" onChange={e => setLogin({...login, user: e.target.value})} /></label>
      <label>Access Authentication password<Input type="password" value={login.password} autoComplete="new-password" onChange={e => setLogin({...login, password: e.target.value})} /></label>
      <p className="health-detail">Saved with Save settings, in the macOS Keychain.</p>
      <ol className="list-decimal pl-5">{cameraSteps.accessAuthentication.map(step => <li key={step}>{step}</li>)}</ol>
      <FingerprintBox />
    </>}
    <h3>Check</h3>
    <Button type="button" disabled={busy} onClick={() => void run(async () => setCheck(await cameraTestConnection()))}>Test camera connection</Button>
    {check && <div role="status"><p>{check.holder ? `${check.holder} is holding the camera` : check.programs.length ? `These programs could take the camera: ${check.programs.join(', ')}` : 'Nothing is holding the camera'}</p>{check.holder && check.sentence && <p>{check.sentence}</p>}</div>}
  </section>;
}
