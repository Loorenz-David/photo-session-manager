export type SettingsSection = 'capture-one' | 'camera' | 'export-and-previews' | 'shopify' | 'pricing' | 'storage' | 'research' | 'writer' | 'shooting-templates';

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
