import type { ReactNode } from 'react';
import { ChevronDown, PanelLeftClose, PanelLeftOpen } from 'lucide-react';
import { Hint } from '@/components/ui/tooltip';

export type ToolSection = { id: string; title: string; content: ReactNode };

type ToolsPanelProps = {
  hidden: boolean;
  sections: readonly ToolSection[];
  isOpen: (id: string) => boolean;
  onToggle: (id: string) => void;
  onToggleHidden: () => void;
};

/**
 * The Tools panel on the editor's left (10D §What the photographer sees, bullet 3): a list of sections that fold open and
 * closed on their header; Composition is the first. Colour and light tools arrive later as more entries of `sections`.
 */
export function ToolsPanel({ hidden, sections, isOpen, onToggle, onToggleHidden }: ToolsPanelProps) {
  if (hidden) return <aside className="tools-rail" aria-label="Tools"><Hint text={{ name: 'Show Tools panel', key: '⌘T', description: 'Brings the Tools panel back' }}><button type="button" className="stage-icon-button" aria-label="Show Tools panel" onClick={onToggleHidden}><PanelLeftOpen aria-hidden /></button></Hint></aside>;
  return (
    <aside className="tools-panel" aria-label="Tools">
      <div className="tools-panel-head"><span className="tools-panel-title">Tools</span><Hint text={{ name: 'Hide Tools panel', key: '⌘T', description: 'Folds the panel into a rail on the left' }}><button type="button" className="stage-icon-button" aria-label="Hide Tools panel" onClick={onToggleHidden}><PanelLeftClose aria-hidden /></button></Hint></div>
      {sections.map(section => {
        const open = isOpen(section.id);
        return (
          <section key={section.id} className="tools-section" data-open={open ? 'true' : 'false'}>
            <button type="button" className="tools-section-header" aria-expanded={open} onClick={() => onToggle(section.id)}>
              <span className="tools-section-title">{section.title}</span>
              <ChevronDown className="tools-section-chevron" aria-hidden />
            </button>
            {open && <div className="tools-section-body">{section.content}</div>}
          </section>
        );
      })}
    </aside>
  );
}
