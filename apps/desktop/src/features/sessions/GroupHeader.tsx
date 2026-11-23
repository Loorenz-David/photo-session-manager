/** A time group's header; `count` is the group's sessions on every page (H2, MC-H2.1). */
export function GroupHeader({ label, count }: { label: string; count: number }) {
  return (
    <h2 className="group-header">
      {label} <span className="group-count">{count}</span>
    </h2>
  );
}
