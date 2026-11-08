import { useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cameraFingerprintConfirm, cameraFingerprintReject, cameraFtpPasswordGenerate, cameraSecretsSet, cameraStatus, cameraSupportGet, cameraSupportInstall, cameraTestConnection } from '@/services/camera';
import type { CameraSettingsDto, CameraSupportDto, TestConnectionResponse } from '@/services/bindings';
import { onFingerprintPending } from '@/services/events';
import { pickArchive } from '@/services/settings';
import * as cameraSteps from './cameraSteps';

export function CameraSection({ ports, onPortsChange }: { ports: CameraSettingsDto; onPortsChange: (ports: CameraSettingsDto) => void }) {
  const [support, setSupport] = useState<CameraSupportDto | null>(null);
  const [password, setPassword] = useState<string | null>(null);
  const [user, setUser] = useState('');
  const [loginPassword, setLoginPassword] = useState('');
  const [fingerprint, setFingerprint] = useState<string | null>(null);
  const [fingerprintChanged, setFingerprintChanged] = useState(false);
  const [fingerprintNeedsAction, setFingerprintNeedsAction] = useState(false);
  const [check, setCheck] = useState<TestConnectionResponse | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    let alive = true;
    let off: (() => void) | undefined;
    Promise.resolve().then(cameraSupportGet).then(value => { if (alive) setSupport(value); }).catch(() => undefined);
    Promise.resolve().then(() => onFingerprintPending(value => { if (alive) { setFingerprint(value.fingerprint); setFingerprintChanged(value.replace); setFingerprintNeedsAction(true); } })).then(stop => { if (alive) off = stop; else stop(); }).catch(() => undefined);
    Promise.resolve().then(cameraStatus).then(status => {
      if (!alive || status.kind !== 'needs_action') return;
      if (status.reason === 'fingerprint_mismatch' || status.reason === 'fingerprint_unconfirmed') {
        setFingerprintChanged(status.reason === 'fingerprint_mismatch');
        setFingerprintNeedsAction(true);
      }
    }).catch(() => undefined);
    return () => { alive = false; off?.(); };
  }, []);
  const run = async (task: () => Promise<void>) => {
    setBusy(true);
    try { await task(); } catch (error) {
      const e = error as { code?: string; data?: Record<string, string>; message?: string };
      toast.error(e.code === 'support_archive_invalid' ? `The archive is missing or has a changed file: ${e.data?.file}`
        : e.code === 'support_version_unsupported' ? `This SDK version (${e.data?.version}) is not supported; use 2.02.00`
        : e.code === 'fingerprint_changed' ? 'The camera shows a different fingerprint now. Compare the new one.' : e.message ?? String(error));
    } finally { setBusy(false); }
  };
  const field = (label: string, key: keyof CameraSettingsDto) => <label className="flex flex-col gap-1">{label}<Input type="number" min={1024} max={65535} value={ports[key]} onChange={e => onPortsChange({...ports, [key]: Number(e.target.value)})} /></label>;
  return <section className="flex flex-col gap-4" aria-label="Camera">
    <h2>Camera</h2>
    <p>Session Manager sets where the camera saves photos: card and computer on the cable, card only on Wi-Fi. The card always keeps a copy.</p>
    <h3>Camera support</h3>
    {support ? <p>Sony Camera Remote SDK {support.version} installed</p> : <>
      <p>Session Manager can control a Sony camera directly. Download Sony&apos;s Camera Remote SDK for Mac from Sony&apos;s developer site (you accept Sony&apos;s terms there), then add it here.</p>
      <Button type="button" disabled={busy} onClick={() => void run(async () => { const path = await pickArchive(); if (!path) return; const installed = await cameraSupportInstall(path); setSupport(installed); toast.success(`Camera support for Sony ${installed.version} installed`); })}>Add camera support → Sony</Button>
    </>}
    {support && <>
      <h3>Transfer from the camera</h3><p>User: sessionmanager</p>
      <Button type="button" disabled={busy} onClick={() => void run(async () => { const value = await cameraFtpPasswordGenerate(); setPassword(value.password); })}>Generate password</Button>
      {password && <div className="rounded border p-3"><p>Enter this password on the camera now; it is shown only once.</p><code>{password}</code>
        <Button type="button" onClick={() => void navigator.clipboard.writeText(password).then(() => { setCopied(true); window.setTimeout(() => setCopied(false), 1500); })}>{copied ? 'Copied' : 'Copy'}</Button>
        <ol className="list-decimal pl-5">{cameraSteps.ftp("<this Mac's address on your network>", ports.receiver_port, 'sessionmanager').map(step => <li key={step}>{step}</li>)}</ol>
      </div>}
      <div className="grid grid-cols-3 gap-2">{field('Receiver port', 'receiver_port')}{field('Passive ports from', 'passive_port_start')}{field('Passive ports to', 'passive_port_end')}</div>
      <h3>Wi-Fi control</h3>
      <label>Access Authentication user<Input value={user} autoComplete="off" onChange={e => setUser(e.target.value)} /></label>
      <label>Access Authentication password<Input type="password" value={loginPassword} autoComplete="new-password" onChange={e => setLoginPassword(e.target.value)} /></label>
      <Button type="button" disabled={busy} onClick={() => void run(async () => { await cameraSecretsSet({access_auth_user: user, access_auth_password: loginPassword}); setUser(''); setLoginPassword(''); toast.success('Camera login saved'); })}>Save camera login</Button>
      <ol className="list-decimal pl-5">{cameraSteps.accessAuthentication.map(step => <li key={step}>{step}</li>)}</ol>
      {(fingerprint || fingerprintNeedsAction) && <div className="rounded border p-3"><h4>{fingerprintChanged ? 'Fingerprint changed' : 'Confirm camera fingerprint'}</h4><p>Compare with the camera: MENU › Network › Network Option › Access Authen. Info</p>
        {fingerprint ? <><code>{fingerprint}</code><div className="flex gap-2"><Button type="button" onClick={() => void run(async () => { await cameraFingerprintConfirm(fingerprint); setFingerprint(null); setFingerprintNeedsAction(false); })}>Confirm</Button><Button type="button" variant="outline" onClick={() => void run(async () => { await cameraFingerprintReject(); setFingerprint(null); setFingerprintNeedsAction(false); })}>Not mine</Button></div></> : <p>The fingerprint will appear when the camera reports it.</p>}
      </div>}
    </>}
    <h3>Check</h3>
    <Button type="button" disabled={busy} onClick={() => void run(async () => setCheck(await cameraTestConnection()))}>Test camera connection</Button>
    {check && <div role="status"><p>{check.holder ? `${check.holder} is holding the camera` : check.programs.length ? `These programs could take the camera: ${check.programs.join(', ')}` : 'Nothing is holding the camera'}</p>{check.holder && check.sentence && <p>{check.sentence}</p>}</div>}
  </section>;
}
