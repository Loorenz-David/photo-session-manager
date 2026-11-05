import { useEffect, useState } from 'react';
import type { ComposeController } from './useComposeGesture';

export function FrameMeter({ controller }: { controller: ComposeController }) {
  const [reading, setReading] = useState({ avg: 0, worst: 0 });
  useEffect(() => {
    let raf = 0;
    let last: number | null = null;
    const samples: { at: number; ms: number }[] = [];
    const tick = (at: number) => {
      const active = !controller.isIdle();
      if (active && last !== null) {
        samples.push({ at, ms: at - last });
        while (samples.length && at - samples[0].at > 1000) samples.shift();
        const sum = samples.reduce((n, sample) => n + sample.ms, 0);
        setReading({ avg: Math.round(sum / samples.length), worst: Math.round(Math.max(...samples.map(sample => sample.ms))) });
      }
      last = active ? at : null;
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [controller]);
  return <output className="frame-meter">avg {reading.avg} ms · worst {reading.worst} ms</output>;
}
