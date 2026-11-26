import { useEffect, useState } from 'react';

/** iPhone's home-screen app (and any installed app) already runs without browser bars. */
function standalone(): boolean {
  const media = typeof window.matchMedia === 'function' && window.matchMedia('(display-mode: standalone)').matches;
  return media || (navigator as Navigator & { standalone?: boolean }).standalone === true;
}

/** PH7: shown iff the browser can go full screen, is not already, and the app is not installed. */
function offered(): boolean {
  return document.fullscreenEnabled === true && !document.fullscreenElement && !standalone();
}

/**
 * Card LV3: Android Chrome does not install the app over plain HTTP, so one tap hides the browser
 * bars (the Fullscreen API works on HTTP). The button sits in its own row above the stage, so it
 * never covers a gesture surface.
 */
export function FullscreenButton() {
  const [show, setShow] = useState(offered);
  useEffect(() => {
    const update = () => setShow(offered());
    document.addEventListener('fullscreenchange', update);
    return () => document.removeEventListener('fullscreenchange', update);
  }, []);
  if (!show) return null;
  return <div className="ph-fullscreen-row">
    <button type="button" className="ph-fullscreen" onClick={() => {
      // A refusal (a browser that needs another gesture) leaves the button where it is.
      void Promise.resolve(document.documentElement.requestFullscreen()).catch(() => undefined);
    }}>Full screen</button>
  </div>;
}
