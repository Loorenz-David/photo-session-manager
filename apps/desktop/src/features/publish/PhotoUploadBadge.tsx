export function PhotoUploadBadge({ state }: { state: string }) {
  const label = state === 'ready' ? 'On Shopify' : state === 'none' ? 'Not uploaded' : 'Uploading';
  return <span className="rounded bg-background/90 px-1 text-xs">{label}</span>;
}
