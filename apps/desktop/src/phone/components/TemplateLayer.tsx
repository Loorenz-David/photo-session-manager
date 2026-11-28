import { exampleUrl } from '../api';
import type { PhoneShot } from '../api';

type Props = {
  token: string;
  /** The current template shot; `null` in Free shot (no template). */
  shot: PhoneShot | null;
  /** 1 in template mode, the overlay value while live shows, 0 in Free shot (design §2). */
  opacity: number;
  /** The shot's zoom preset as a scale about the centre (the same as the live picture's). */
  scale: number;
};

/**
 * Design §2 layer 2: the template example of the current shot, covering the viewport, black and
 * white by the phone's own display filter (H3: the file is served as stored).
 */
export function TemplateLayer({ token, shot, opacity, scale }: Props) {
  return <div className="ph-layer ph-template-layer" data-testid="template-layer" style={{ opacity }}>
    {shot?.example && <img className="ph-cover ph-grey" alt="" draggable={false} src={exampleUrl(token, shot.container_id)} style={{ transform: `scale(${scale})` }} />}
    {shot && <span className="ph-image-label ph-image-label-left">TEMPLATE · {shot.name.toUpperCase()}</span>}
  </div>;
}
