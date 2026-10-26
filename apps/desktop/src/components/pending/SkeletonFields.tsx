import { Skeleton } from '@/components/ui/skeleton';

export function SkeletonFields({ n = 4 }: { n?: number }) {
  return <div aria-busy="true" className="flex flex-col gap-5">{Array.from({ length: n }, (_, i) => <div key={i} className="flex flex-col gap-2"><Skeleton className="h-3 w-24" /><Skeleton className="h-8 w-full" /></div>)}</div>;
}
