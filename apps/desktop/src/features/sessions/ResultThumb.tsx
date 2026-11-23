import type { Item } from '@/services/items';
import { previewUrl } from '@/services/sessions';
import type { Pictures } from './usePictures';

/**
 * H2 (10H §5, MC-H2.4–5): a search result's tile: its picture on this Mac (a shop picture that arrived since the
 * answer, a local result's cover that arrived since, or the answer's own path), else its first letter while the
 * picture loads or when there is none.
 */
export function ResultThumb({ item, pictures }: { item: Item; pictures: Pictures }) {
  const path =
    (item.external_id ? pictures.search(item.external_id) : undefined) ??
    (item.session_id != null ? pictures.cover(item.session_id) : undefined) ??
    item.picture_path;
  return (
    <span className="result-thumb" aria-hidden>
      {path ? <img src={previewUrl(path)} alt="" /> : (item.item_type || item.title).slice(0, 1).toUpperCase()}
    </span>
  );
}
