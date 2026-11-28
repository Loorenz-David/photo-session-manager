/**
 * LV-D17: "Fill item properties" opens an empty full-page container for now; the phone's inputs
 * come later (a measuring tape's target dimension, properties easier to fill phone in hand).
 */
export function PropertiesPage({ onBack }: { onBack: () => void }) {
  return <div className="ph-page" role="dialog" aria-label="Item properties" data-testid="properties-page">
    <div className="ph-safe-top" />
    <div className="ph-page-head">
      <button type="button" className="ph-round ph-page-back" aria-label="Back" onClick={onBack}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M19 12H5M11 6l-6 6 6 6" /></svg>
      </button>
      <span className="ph-sheet-title">Item properties</span>
    </div>
    <div className="ph-page-body" data-testid="properties-container" />
  </div>;
}
