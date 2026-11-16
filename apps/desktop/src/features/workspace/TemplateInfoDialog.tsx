import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog';
import { TemplateExample } from '@/features/settings/TemplateExample';
import type { TemplateView } from '@/services/templates';

function presetText(p: TemplateView['containers'][number]['presets']): string {
  return [p.split && 'Split', p.shape && `Shape ${p.shape}`, p.zoom > 0 && `Zoom ${p.zoom} %`].filter(Boolean).join(' · ');
}

export function TemplateInfoDialog({ view, open, onOpenChange }: { view: TemplateView; open: boolean; onOpenChange: (open: boolean) => void }) {
  return <Dialog open={open} onOpenChange={onOpenChange}>
    <DialogContent className="template-info-dialog">
      <DialogTitle>Shooting template</DialogTitle>
      <DialogDescription>Containers in final image order.</DialogDescription>
      <div className="template-info-list">
        {view.containers.map(container => <div key={container.id} className="template-info-entry" data-container={container.final_no}>
          <strong>{container.final_no}</strong>
          <TemplateExample path={container.example_path} />
          <div><span>Shot {container.shoot_no}</span>{presetText(container.presets) && <p>{presetText(container.presets)}</p>}</div>
        </div>)}
      </div>
    </DialogContent>
  </Dialog>;
}
