import { Bell } from 'lucide-react';
import { Button } from '@/components/ui/button';

/** Bell with the failed-publish count. The drawer and the count arrive with publishing (P15); until then 0. */
export function NotificationsBell({ failedCount = 0 }: { failedCount?: number }) {
  return (
    <Button variant="ghost" size="icon" className="relative" aria-label="Notifications" title="Notifications (publish history arrives with publishing)">
      <Bell />
      {failedCount > 0 && <span className="bell-count">{failedCount}</span>}
    </Button>
  );
}
