import type { ReactNode } from 'react';
import { ChevronDown } from 'lucide-react';

export type ToolSection = { id: string; title: string; content: ReactNode };

type ToolsPanelProps = {
  hidden: boolean;
  sections: readonly ToolSection[];
  isOpen: (id: string) => boolean;
  onToggle: (id: string) => void;
};

/**
 * The Tools panel on the editor's left (10D §What the photographer sees, bullet 3): a list of sections that fold open and
 * closed on their header; Composition is the first. Colour and light tools arrive later as more entries of `sections`.
 */
export function ToolsPanel({ hidden, sections, isOpen, onToggle }: ToolsPanelProps) {
  return (
    <aside className="tools-panel" aria-label="Tools" hidden={hidden}>
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
