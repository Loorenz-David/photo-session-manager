import { Archive, ImageOff } from 'lucide-react';
import { FILES_CLEARED, type ArchivedPicture } from '@/services/retention';
import { previewUrl } from '@/services/sessions';

type ArchivedViewProps = {
  pictures: readonly ArchivedPicture[];
  /** Opens the product details form: attributes that need no images stay editable (10F Q3). */
  onEdit?: () => void;
};

/**
 * A cleared item (10F §3, Q3): its published photos as Shopify holds them (downloaded once at preview size and
 * cached), read-only — no review, reorder, compose or republish. Main photos first, then condition photos.
 */
export function ArchivedView({ pictures, onEdit }: ArchivedViewProps) {
  const groups = [
    { role: 'gallery', title: 'Main' },
    { role: 'condition', title: 'Condition' },
  ].map(group => ({ ...group, pictures: pictures.filter(picture => picture.role === group.role) }));
  return (
    <main className="workspace archived-workspace" aria-label="Archived in Shopify">
      <header className="archived-head">
        <span className="pill pill-published pill-archived"><Archive aria-hidden /> Archived in Shopify</span>
        <p role="note">{FILES_CLEARED}. The photos below come from Shopify; details that need no photos can still be edited.</p>
        {onEdit && <button type="button" className="item-card-edit" onClick={onEdit}>Edit details →</button>}
      </header>
      {groups.filter(group => group.pictures.length > 0).map(group => (
        <section key={group.role} className="archived-group" aria-label={group.title}>
          <h3>{group.title}</h3>
          <div className="archived-grid">
            {group.pictures.map(picture => (
              <figure key={`${picture.role}-${picture.position}`} className="archived-tile">
                {picture.path
                  ? <img src={previewUrl(picture.path)} alt={`${group.title} ${String(picture.position).padStart(2, '0')}`} loading="lazy" />
                  : <span className="archived-missing" role="img" aria-label={`${group.title} ${String(picture.position).padStart(2, '0')} not downloaded`}><ImageOff aria-hidden /></span>}
                <figcaption>{String(picture.position).padStart(2, '0')}</figcaption>
              </figure>
            ))}
          </div>
        </section>
      ))}
    </main>
  );
}
