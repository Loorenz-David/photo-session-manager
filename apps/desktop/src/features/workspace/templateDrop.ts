import type { Photo } from '@/services/photos';
import type { TemplateView } from '@/services/templates';

/** What a drop does in the photo panel: put the photo into a container (10G §6.1), or today's reorder/classify. */
export type DropAction = { kind: 'template'; container: number } | { kind: 'default' };

const DEFAULT: DropAction = { kind: 'default' };

export const containerDropId = (finalNo: number) => `container:${finalNo}`;

/**
 * Pure (T3 core mechanism). In a templated session a drop onto a container placeholder (`container:<n>`) or onto a
 * Main photo that sits in a container is a template drop into that container, whether or not the dragged photo has a
 * container itself (without one, onto a filled container, it is a retake; 10G §6.3); everything else, and every drop
 * in an untemplated session, keeps today's behaviour.
 */
export function dropAction(photoId: number, overId: string | number, view: TemplateView | null | undefined, photos: readonly Photo[]): DropAction {
  if (!view?.templated) return DEFAULT;
  const over = String(overId);
  const containers = new Set(view.containers.map(container => container.final_no));
  if (over.startsWith('container:')) {
    const n = Number(over.slice('container:'.length));
    return containers.has(n) ? { kind: 'template', container: n } : DEFAULT;
  }
  if (over.startsWith('photo:')) {
    const overPhotoId = Number(over.slice('photo:'.length));
    if (overPhotoId === photoId) return DEFAULT;
    const target = photos.find(photo => photo.id === overPhotoId);
    const slot = view.photos.find(entry => entry.photo_id === overPhotoId)?.slot ?? null;
    if (target?.classification === 'main' && slot !== null && containers.has(slot)) return { kind: 'template', container: slot };
  }
  return DEFAULT;
}
