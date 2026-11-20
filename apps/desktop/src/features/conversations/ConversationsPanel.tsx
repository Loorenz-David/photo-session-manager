import { useCallback, useEffect, useRef, useState } from 'react';
import { Bell, BellOff, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  conversationsList,
  conversationsSettingsGet,
  conversationsSettingsSave,
  type ConversationList,
  type ConversationTab,
} from '@/services/conversations';
import { ConversationRow } from './ConversationRow';
import { ConversationView } from './ConversationView';
import { useConversationsChanged, useConversationsContext, type ConversationsState } from './useConversations';

/** The states in which the copy is hidden and the panel asks to connect (09B §4). */
const HIDDEN = new Set(['no_key', 'no_shop', 'store_unmatched']);

/** 09B §4, §9: the mute button is the Settings → Pricing sound switch. Pressed = muted. */
function MuteButton() {
  const [sound, setSound] = useState<boolean | null>(null);
  useEffect(() => {
    let alive = true;
    Promise.resolve()
      .then(() => conversationsSettingsGet())
      .then(settings => alive && setSound(settings.sound))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, []);
  const muted = sound === false;
  const toggle = async () => {
    if (sound === null) return;
    try {
      const saved = await conversationsSettingsSave({ sound: !sound });
      setSound(saved.sound);
    } catch {
      // The switch stays as it was; Settings → Pricing shows the stored value.
    }
  };
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label="Mute"
      aria-pressed={muted}
      title={muted ? 'Sound off for new questions and replies' : 'Sound on for new questions and replies'}
      disabled={sound === null}
      onClick={() => void toggle()}
    >
      {muted ? <BellOff /> : <Bell />}
    </Button>
  );
}

/** The panel's list for the state's scope and the tab, re-read on `conversations_changed`. */
function useList(state: ConversationsState, tab: ConversationTab): ConversationList | null {
  const [list, setList] = useState<ConversationList | null>(null);
  const latest = useRef(0);
  const { scope } = state;
  const load = useCallback(() => {
    const mine = ++latest.current;
    Promise.resolve()
      .then(() => conversationsList(scope, tab))
      .then(next => {
        if (mine === latest.current) setList(next);
      })
      .catch(() => undefined);
  }, [scope, tab]);
  useEffect(load, [load]);
  useConversationsChanged(load);
  return list;
}

function Body({ state, tab, setTab }: { state: ConversationsState; tab: ConversationTab; setTab: (tab: ConversationTab) => void }) {
  const list = useList(state, tab);
  if (state.selected) return <ConversationView key={state.selected.id} id={state.selected.id} onBack={state.back} />;
  if (list && HIDDEN.has(list.state)) {
    return (
      <div className="conversations-connect">
        <p>Connect Pricing in Settings → Pricing</p>
        <Button variant="outline" onClick={() => { window.location.hash = '#/settings'; }}>Open Settings</Button>
      </div>
    );
  }
  const elsewhere = state.scope.kind === 'session' ? list?.news_elsewhere ?? 0 : 0;
  return (
    <>
      <div className="conversations-tabs" role="tablist">
        <button type="button" role="tab" aria-selected={tab === 'open'} onClick={() => setTab('open')}>Open</button>
        <button type="button" role="tab" aria-selected={tab === 'resolved'} onClick={() => setTab('resolved')}>Resolved</button>
      </div>
      {list?.problem && <p role="alert" className="conversations-problem">{list.problem}</p>}
      <div className="conversations-list">
        {list && list.rows.length === 0 && (
          <p className="conversations-empty">{tab === 'open' ? 'No open conversations' : 'No resolved conversations'}</p>
        )}
        {list?.rows.map(row => <ConversationRow key={row.id} row={row} onSelect={() => state.select(row)} />)}
      </div>
      {elsewhere > 0 && (
        <button type="button" className="conversations-footer" onClick={state.showAll}>
          {elsewhere} new in other sessions → All conversations
        </button>
      )}
    </>
  );
}

/** C2 (09B §7 Panel → List): the right-side panel the header icon opens. */
export function ConversationsPanel() {
  const state = useConversationsContext();
  const [tab, setTab] = useState<ConversationTab>('open');
  const open = state?.open ?? false;
  useEffect(() => {
    if (open) setTab('open');
  }, [open]);
  if (!state || !state.open) return null;
  return (
    <aside role="dialog" aria-label="Conversations" className="conversations-panel">
      <div className="conversations-head">
        <h2>Conversations</h2>
        <MuteButton />
        <Button variant="ghost" size="icon" aria-label="Close" onClick={state.close}>
          <X />
        </Button>
      </div>
      <Body state={state} tab={tab} setTab={setTab} />
    </aside>
  );
}
