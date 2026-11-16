import { useState } from 'react';
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { TemplateExample } from '@/features/settings/TemplateExample';
import type { TemplateView } from '@/services/templates';

type Order = 'final' | 'shooting';

function presetText(p: TemplateView['containers'][number]['presets']): string {
  return [p.split && 'Split', p.shape && `Shape ${p.shape}`, p.zoom > 0 && `Zoom ${p.zoom} %`].filter(Boolean).join(' · ');
}

/**
 * The session's template (10G §5): the containers with their examples and presets, either in final image order (the
 * order they are published in) or in shooting order (the route around the piece); the next shot is marked.
 */
export function TemplateInfoDialog({ view, open, onOpenChange }: { view: TemplateView; open: boolean; onOpenChange: (open: boolean) => void }) {
  const [order, setOrder] = useState<Order>('final');
  const shooting = order === 'shooting';
  const containers = [...view.containers].sort((a, b) => (shooting ? a.shoot_no - b.shoot_no : a.final_no - b.final_no));
  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent className="template-info-dialog">
      <DialogTitle>Shooting template</DialogTitle>
      <DialogDescription>{shooting ? 'Containers in the order you shoot them.' : 'Containers in final image order.'}</DialogDescription>
      <div className="template-order-switch" role="group" aria-label="Order">
        <button type="button" aria-pressed={!shooting} onClick={() => setOrder('final')}>Final order</button>
        <button type="button" aria-pressed={shooting} onClick={() => setOrder('shooting')}>Shooting order</button>
      </div>
      <div className="template-info-list">
        {containers.map(container => {
          const next = container.shoot_no === view.next_shot;
          return <div key={container.id} className="template-info-entry" data-container={container.final_no} data-next={next || undefined}>
            <strong>{shooting ? container.shoot_no : container.final_no}</strong>
            <TemplateExample path={container.example_path} />
            <div>
              {container.name && <span className="template-info-name">{container.name}</span>}
              <span>{shooting ? `Image ${container.final_no}` : `Shot ${container.shoot_no}`}</span>
              {next && <span className="template-next-mark">Next</span>}
              {presetText(container.presets) && <p>{presetText(container.presets)}</p>}
              {container.description && <p className="template-info-description">{container.description}</p>}
            </div>
          </div>;
        })}
      </div>
    </DialogContent>
  </Dialog>;
}
