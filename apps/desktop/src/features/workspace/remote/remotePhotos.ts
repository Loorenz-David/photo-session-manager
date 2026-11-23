import type { Photo } from '@/services/photos';
import type { RemoteView } from '@/services/remote';

/**
 * H4 (10H §6, MC-H3.5): a remote item's Shopify pictures as the workspace's photo objects, gallery then condition in
 * record order. Each gets a negative id (`-(index + 1)`) so nothing can send it to a photo command, the picture's
 * cached file as both preview paths (`null` until it is downloaded), and a neutral composition with a non-null rect and
 * frame (so the menus' Compose…, Split and Normal stay enabled and reach the edit gate). Pure.
 */
export function remotePhotos(view: RemoteView): Photo[] {
  return view.pictures.map((picture, index) => {
    const id = -(index + 1);
    return {
      id,
      session_id: 0,
      classification: picture.role === 'condition' ? 'condition' : 'main',
      position: picture.position,
      captured_at: null,
      original_preview_path: picture.path,
      adjusted_preview_path: picture.path,
      thumbnail_path: null,
      composition: {
        photo_id: id,
        kind: 'normal',
        shape: 'original',
        orientation: 'named',
        quarter_turns: 0,
        flip: 'none',
        straighten: 0,
        rect: { x: 0, y: 0, w: 1, h: 1 },
        frame: { w: 1, h: 1 },
        frame_source: 'preview',
        origin: 'default',
        revision: 0,
        state: 'ok',
        review_reason: null,
        updated_at: 0,
      },
      orientation: null,
      source: 'import',
      integrity: 'ok',
      upload_state: 'ready',
    };
  });
}
