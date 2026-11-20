import { goTo } from '@/lib/routes';

export type SettingsSection = 'capture-one' | 'camera' | 'export-and-previews' | 'shopify' | 'pricing' | 'storage' | 'research' | 'writer' | 'shooting-templates';

/** A28: another page can open Settings on one section (the conversations panel → Pricing). */
export const SETTINGS_SECTION_EVENT = 'psm:settings-section';
let requested: SettingsSection | null = null;

/** The section asked for before Settings opened, taken once (Settings starts on it). */
export function takeRequestedSection(): SettingsSection | null {
  const section = requested;
  requested = null;
  return section;
}

/** Opens Settings on `section`: switches section when Settings is on screen, else goes there (through the leave guard). */
export function openSettingsSection(section: SettingsSection): void {
  if (window.location.hash === '#/settings') {
    window.dispatchEvent(new CustomEvent<SettingsSection>(SETTINGS_SECTION_EVENT, { detail: section }));
    return;
  }
  requested = section;
  goTo('#/settings');
}

const sections: { id: SettingsSection; label: string }[] = [
  { id: 'capture-one', label: 'Capture One' },
  { id: 'camera', label: 'Camera' },
  { id: 'export-and-previews', label: 'Export and previews' },
  { id: 'shopify', label: 'Shopify' },
  { id: 'pricing', label: 'Pricing' },
  { id: 'storage', label: 'Storage' },
  { id: 'research', label: 'Research' },
  { id: 'writer', label: 'Writer' },
  { id: 'shooting-templates', label: 'Shooting templates' },
];

export function SettingsNav({ active, showResearch, onSelect }: { active: SettingsSection; showResearch: boolean; onSelect: (section: SettingsSection) => void }) {
  return <nav aria-label="Settings sections" className="settings-nav">
    {sections.filter(section => showResearch || (section.id !== 'research' && section.id !== 'writer')).map(section =>
      <button key={section.id} type="button" aria-current={section.id === active ? 'page' : undefined} onClick={() => onSelect(section.id)}>
        {section.label}
      </button>)}
  </nav>;
}
