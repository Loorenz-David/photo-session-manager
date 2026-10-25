import type { FieldInputProps } from '../MetafieldInput';

const PLATFORM: Record<string, string> = { shopify: 'Shopify admin' };

/** `unsupported` (reference and unknown types): read-only, edited on the platform (ch08 §Schema). */
export function Unsupported({ definition }: FieldInputProps) {
  return (
    <p className="rounded-lg border border-dashed px-3 py-2 text-xs text-muted-foreground">
      Edited in {PLATFORM[definition.integration] ?? definition.integration}
    </p>
  );
}
