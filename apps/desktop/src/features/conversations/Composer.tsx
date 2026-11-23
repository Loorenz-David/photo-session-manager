import { useState } from 'react';
import { ArrowUp, ImagePlus, X } from 'lucide-react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { previewUrl } from '@/services/sessions';
import {
  conversationReply,
  type ConversationAttachable,
  type ConversationView,
} from '@/services/conversations';
import { PhotoPicker, photoLabel } from './PhotoPicker';
import { ReplyLine } from './MessageBubble';

/** The longest reply, in characters = Unicode code points (the backend's scalar values, MC-C2.1). */
export const REPLY_MAX = 4000;
/** The counter appears from here. */
const COUNTER_FROM = 3800;

/** The message a reply answers, as shown above the box. */
export type ReplyTarget = { message_id: string; author_name: string; excerpt: string };

/** The excerpt of a reply target: its first line, at most 80 code points (the backend's rule, projection L9). */
export function replyExcerpt(text: string): string {
  return [...(text.split('\n')[0] ?? '')].slice(0, 80).join('');
}

/** Characters as the backend counts them: code points of the trimmed text, never UTF-16 units. */
function characters(text: string): number {
  return [...text.trim()].length;
}

/**
 * C3 (09B §8; CHAT_UX §4): the reply box. Enter sends, Shift+Enter is a new line; nothing is sent when the trimmed text
 * is empty or longer than 4000 characters. Each press makes one `client_mutation_id` (`crypto.randomUUID()`), never
 * again for that reply. The box stays usable while earlier replies are sending (they queue in order).
 */
export function Composer({
  view,
  attachable,
  replyTarget,
  onClearReply,
  onSent,
}: {
  view: ConversationView;
  attachable: ConversationAttachable | null;
  replyTarget: ReplyTarget | null;
  onClearReply: () => void;
  onSent: () => void;
}) {
  const [text, setText] = useState('');
  const [chosen, setChosen] = useState<string[]>([]);
  const [picking, setPicking] = useState(false);
  const count = characters(text);
  const photos = attachable?.photos ?? [];

  const toggle = (mediaId: string) =>
    setChosen(current => (current.includes(mediaId) ? current.filter(id => id !== mediaId) : [...current, mediaId]));

  const send = () => {
    if (!view.can_send || count === 0 || count > REPLY_MAX) return;
    const input = {
      conversation_id: view.id,
      client_mutation_id: crypto.randomUUID(),
      text: text.trim(),
      reply_to_message_id: replyTarget?.message_id ?? null,
      attachment_media_ids: chosen,
    };
    setText('');
    setChosen([]);
    setPicking(false);
    onClearReply();
    Promise.resolve()
      .then(() => conversationReply(input))
      .catch(error => toast(error instanceof Error ? error.message : String(error)))
      .finally(onSent);
  };

  return (
    <div className="composer">
      {!view.can_send && view.off_reason && (
        <p className="composer-off" role="status">
          {view.off_reason}
        </p>
      )}
      {replyTarget && (
        <div className="composer-reply">
          <ReplyLine authorName={replyTarget.author_name} excerpt={replyTarget.excerpt} />
          <Button variant="ghost" size="icon" aria-label="Cancel reply" onClick={onClearReply}>
            <X />
          </Button>
        </div>
      )}
      {chosen.length > 0 && (
        <div className="composer-chosen">
          {chosen.map(mediaId => {
            const photo = photos.find(p => p.media_id === mediaId);
            const label = photo ? photoLabel(photos, photo) : 'Photo';
            return (
              <span key={mediaId} className="composer-chosen-item" data-testid={`chosen-${mediaId}`}>
                {photo?.preview_path ? <img src={previewUrl(photo.preview_path)} alt={label} /> : <span>{label}</span>}
                <button type="button" aria-label={`Remove ${label}`} onClick={() => toggle(mediaId)}>
                  ✕
                </button>
              </span>
            );
          })}
        </div>
      )}
      {picking && attachable && (
        <PhotoPicker attachable={attachable} chosen={chosen} onToggle={toggle} onClose={() => setPicking(false)} />
      )}
      <div className="composer-box">
        <Button
          variant="ghost"
          size="icon"
          aria-label="Add photos"
          disabled={!view.can_send || !attachable || attachable.reason === 'not_on_this_mac'}
          onClick={() => setPicking(open => !open)}
        >
          <ImagePlus />
        </Button>
        <textarea
          aria-label="Reply"
          className="composer-text"
          rows={1}
          value={text}
          disabled={!view.can_send}
          placeholder={view.can_send ? 'Reply…' : (view.off_reason ?? '')}
          onChange={event => setText(event.target.value)}
          onKeyDown={event => {
            if (event.key === 'Enter' && !event.shiftKey && !event.nativeEvent.isComposing) {
              event.preventDefault();
              send();
            }
          }}
        />
        <Button
          size="icon"
          className="composer-send"
          aria-label="Send"
          disabled={!view.can_send || count === 0 || count > REPLY_MAX}
          onClick={send}
        >
          <ArrowUp />
        </Button>
      </div>
      {count >= COUNTER_FROM && (
        <p className="composer-counter" data-over={count > REPLY_MAX}>
          {count} / {REPLY_MAX}
        </p>
      )}
    </div>
  );
}
