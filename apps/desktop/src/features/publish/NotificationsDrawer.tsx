import { Button } from '@/components/ui/button';
import type { NotificationDto } from '@/services/publish';

export function NotificationsDrawer({ notifications, onClose, onOpen }: { notifications: NotificationDto[]; onClose: () => void; onOpen: (sessionId: number) => void }) {
  return <aside role="dialog" aria-label="Notifications" className="fixed right-0 top-0 z-50 h-screen w-80 border-l bg-background p-4 shadow-xl">
    <div className="flex items-center justify-between"><h2 className="font-semibold">Notifications</h2><Button variant="ghost" onClick={onClose}>Close</Button></div>
    {notifications.length === 0 && <p>No notifications yet</p>}
    {notifications.map(note => <div key={note.id} className="border-b py-3">
      <p>{note.text}</p><small>{note.kind}</small>
      <Button variant="outline" size="sm" onClick={() => onOpen(note.session_id)}>Open</Button>
    </div>)}
  </aside>;
}
