export function GroupHeader({ label, count }: { label: string; count: number }) {
  return (
    <h2 className="group-header">
      {label} <span className="group-count">{count}</span>
    </h2>
  );
}
