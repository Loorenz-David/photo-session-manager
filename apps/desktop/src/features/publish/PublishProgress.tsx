import { Progress } from '@/components/ui/progress';

export function PublishProgress({ value }: { value: number }) {
  return <div role="status" className="min-w-28"><span>Publishing {value}%</span><Progress value={value} aria-label={`Publishing ${value}%`} /></div>;
}
