import { Skeleton } from '@/components/ui/skeleton';

export function SkeletonRows({ n = 6 }: { n?: number }) {
  return <div aria-busy="true" className="flex flex-col gap-3">{Array.from({ length: n }, (_, i) => <Skeleton key={i} className="h-12 w-full" />)}</div>;
}
