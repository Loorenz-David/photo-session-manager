import { X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { previewUrl } from '@/services/sessions';
import type { AttachablePhoto, ConversationAttachable } from '@/services/conversations';

/** The most photos one reply carries (09B §8). */
export const PHOTOS_MAX = 10;

function Thumb({ photo, label }: { photo: AttachablePhoto; label: string }) {
  return photo.preview_path ? (
    <img src={previewUrl(photo.preview_path)} alt={label} />
  ) : (
    <span className="photo-picker-missing">{label}</span>
  );
}

/** A photo's name in the picker: its role and its place in the offered list. */
export function photoLabel(photos: AttachablePhoto[], photo: AttachablePhoto): string {
  const index = photos.indexOf(photo) + 1;
  return `${photo.role === 'condition' ? 'Condition photo' : 'Photo'} ${index}`;
}

/**
 * C3 (09B §8): the published photos of the conversation's product, to tick; at most ten (further ones cannot be
 * ticked). "Publish first to attach photos" when none is published.
 */
export function PhotoPicker({
  attachable,
  chosen,
  onToggle,
  onClose,
}: {
  attachable: ConversationAttachable;
  chosen: string[];
  onToggle: (mediaId: string) => void;
  onClose: () => void;
}) {
  const full = chosen.length >= PHOTOS_MAX;
  return (
    <div className="photo-picker" role="group" aria-label="Published photos">
      <div className="photo-picker-head">
        <span>Published photos</span>
        <Button variant="ghost" size="icon" aria-label="Close photos" onClick={onClose}>
          <X />
        </Button>
      </div>
      {attachable.reason === 'nothing_published' && <p className="conversation-note">Publish first to attach photos</p>}
      <div className="photo-picker-grid">
        {attachable.photos.map(photo => {
          const ticked = chosen.includes(photo.media_id);
          const label = photoLabel(attachable.photos, photo);
          return (
            <label key={photo.media_id} className="photo-picker-item" data-ticked={ticked}>
              <input
                type="checkbox"
                aria-label={label}
                checked={ticked}
                disabled={!ticked && full}
                onChange={() => onToggle(photo.media_id)}
              />
              <Thumb photo={photo} label={label} />
            </label>
          );
        })}
      </div>
    </div>
  );
}
