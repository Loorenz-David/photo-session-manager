import { useState, type ReactNode } from 'react';
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog';
import { activityLabel } from '@/lib/format';
import { previewUrl } from '@/services/sessions';
import type { ConversationMessage, PendingReply } from '@/services/conversations';
import { QuoteBlock } from './QuoteBlock';

type Attachment = ConversationMessage['attachments'][number];

/** A photo opened large inside Session Manager (nothing is downloaded or cached by the app). */
function PhotoDialog({ url, onClose }: { url: string | null; onClose: () => void }) {
  return (
    <Dialog open={url !== null} onOpenChange={open => !open && onClose()}>
      <DialogContent className="message-photo-dialog">
        <DialogTitle className="sr-only">Photo</DialogTitle>
        {url && <img src={url} alt="Photo" className="message-photo-large" />}
      </DialogContent>
    </Dialog>
  );
}

function Attachments({ attachments }: { attachments: Attachment[] }) {
  const [large, setLarge] = useState<string | null>(null);
  if (attachments.length === 0) return null;
  return (
    <div className="message-photos">
      {attachments.map((a, index) =>
        a.thumb_url || a.url ? (
          <button
            key={`${a.media_id}-${index}`}
            type="button"
            className="message-thumb"
            aria-label={`Open photo ${index + 1}`}
            onClick={() => setLarge(a.url ?? a.thumb_url)}
          >
            <img src={a.thumb_url ?? a.url ?? undefined} alt={`Photo ${index + 1}`} />
          </button>
        ) : (
          <span key={`${a.media_id}-${index}`} className="message-thumb message-thumb-missing">
            Photo not recorded
          </span>
        ),
      )}
      <PhotoDialog url={large} onClose={() => setLarge(null)} />
    </div>
  );
}

/** The reply line: which message this one answers. */
export function ReplyLine({ authorName, excerpt }: { authorName: string; excerpt: string }) {
  return <p className="message-reply-line">{`↩ ${authorName}: ${excerpt}`}</p>;
}

/**
 * C3 (09B §7; CHAT_UX §2): one message. The pricer's (`asker`) and another account's (`other`) are received: a dark
 * bubble on the left with the author's name and the time under it; this app's own (`me`) are sent: a light bubble on
 * the right labelled "You". Mirrored from Pricing, where the pricer is "You".
 */
export function MessageBubble({
  message,
  now,
  onReply,
  sessionId = null,
}: {
  message: ConversationMessage;
  now?: number;
  /** Makes this message the reply target of the composer. */
  onReply?: () => void;
  /** C4: the conversation's session (null = not on this Mac): the quote chip jumps only when it is on screen. */
  sessionId?: number | null;
}) {
  const sent = message.author.kind === 'me';
  return (
    <div className="message" data-side={sent ? 'sent' : 'received'} data-testid={`message-${message.id}`}>
      <div className="message-bubble">
        {message.reply_to && <ReplyLine authorName={message.reply_to.author_name} excerpt={message.reply_to.excerpt} />}
        {message.quote && <QuoteBlock quote={message.quote} sessionId={sessionId} />}
        <Attachments attachments={message.attachments} />
        <p className="message-text">{message.text}</p>
      </div>
      <div className="message-meta">
        <span className="message-author">{sent ? 'You' : message.author.name}</span>
        <time dateTime={new Date(message.created_at).toISOString()}>{activityLabel(message.created_at, now)}</time>
        {message.edited_at !== null && <span className="message-edited">· edited</span>}
        {onReply && (
          <button type="button" className="message-action" onClick={onReply}>
            Reply
          </button>
        )}
      </div>
    </div>
  );
}

/**
 * C3 (09B §8; CHAT_UX §5): a reply not yet confirmed, on the sent side and dimmed: "Sending…", or its failure.
 * `actions` = what the failed one offers (the composer's Retry and Remove).
 */
export function PendingBubble({ pending, actions }: { pending: PendingReply; actions?: ReactNode }) {
  return (
    <div
      className="message message-pending"
      data-side="sent"
      data-state={pending.state}
      data-testid={`pending-${pending.client_mutation_id}`}
    >
      <div className="message-bubble">
        {pending.reply_to && <ReplyLine authorName={pending.reply_to.author_name} excerpt={pending.reply_to.excerpt} />}
        {pending.attachments.length > 0 && (
          <div className="message-photos">
            {pending.attachments.map(a =>
              a.preview_path ? (
                <img key={a.media_id} className="message-thumb" src={previewUrl(a.preview_path)} alt="Attached photo" />
              ) : (
                <span key={a.media_id} className="message-thumb message-thumb-missing">Photo</span>
              ),
            )}
          </div>
        )}
        <p className="message-text">{pending.text}</p>
      </div>
      <div className="message-meta">
        <span className="message-author">You</span>
        {pending.state === 'failed' ? (
          <span className="message-failure" role="alert">{pending.failure ?? "Couldn't send"}</span>
        ) : (
          <span className="message-sending">Sending…</span>
        )}
        {pending.state === 'failed' && actions}
      </div>
    </div>
  );
}
