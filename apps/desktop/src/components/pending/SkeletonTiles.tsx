import { Skeleton } from '@/components/ui/skeleton';

export function SkeletonTiles({ n = 8 }: { n?: number }) {
  return <div aria-busy="true" className="grid grid-cols-2 gap-2 sm:grid-cols-3">{Array.from({ length: n }, (_, i) => <Skeleton key={i} className="aspect-square w-full" />)}</div>;
}
