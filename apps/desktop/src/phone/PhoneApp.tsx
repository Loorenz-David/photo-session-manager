import { useEffect, useState } from 'react';
import { forgetToken, hello, session, takeToken, Unpaired } from './api';

/** master_plan_live §6: the phone's texts. */
export const NOT_PAIRED = 'Scan the QR code in Session Manager → Settings → Phone to connect this phone.';
export const REMOVED = 'This phone was removed in Session Manager. Scan a new QR code to connect it again.';

type Shown = { kind: 'loading' } | { kind: 'text'; text: string } | { kind: 'connected'; to: string };

/** LV2 placeholder (replaced by the Shoot Companion in LV3): shows whether this phone is paired and to what. */
export function PhoneApp() {
  const [shown, setShown] = useState<Shown>({ kind: 'loading' });
  useEffect(() => {
    const token = takeToken();
    if (!token) { setShown({ kind: 'text', text: NOT_PAIRED }); return; }
    Promise.all([hello(token), session(token)])
      .then(([mac, view]) => setShown({ kind: 'connected', to: view.session?.item.name ?? mac.mac_name }))
      .catch(error => {
        if (error instanceof Unpaired) { forgetToken(); setShown({ kind: 'text', text: REMOVED }); }
        else setShown({ kind: 'text', text: 'Could not reach Session Manager. Check the Wi-Fi and reload.' });
      });
  }, []);
  return <main style={{ fontFamily: '"Helvetica Neue", Helvetica, Arial, sans-serif', padding: 24 }}>
    {shown.kind === 'loading' && <p>Connecting…</p>}
    {shown.kind === 'text' && <p>{shown.text}</p>}
    {shown.kind === 'connected' && <p>Connected to {shown.to}</p>}
  </main>;
}
