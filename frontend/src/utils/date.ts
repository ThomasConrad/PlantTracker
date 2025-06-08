export function formatDate(date: string | Date): string {
  const d = new Date(date);
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(d);
}

export function formatDateTime(date: string | Date): string {
  const d = new Date(date);
  return new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  }).format(d);
}

export function formatRelativeTime(date: string | Date): string {
  const d = new Date(date);
  const now = new Date();
  const diffInMs = now.getTime() - d.getTime();
  const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

  if (diffInDays === 0) {
    return 'Today';
  } else if (diffInDays === 1) {
    return 'Yesterday';
  } else if (diffInDays < 7) {
    return `${diffInDays} days ago`;
  } else if (diffInDays < 30) {
    const weeks = Math.floor(diffInDays / 7);
    return weeks === 1 ? '1 week ago' : `${weeks} weeks ago`;
  } else {
    const months = Math.floor(diffInDays / 30);
    return months === 1 ? '1 month ago' : `${months} months ago`;
  }
}

export function calculateDaysUntil(lastDate: string | null, intervalDays: number): number {
  if (!lastDate) return 0;
  
  const last = new Date(lastDate);
  const next = new Date(last.getTime() + intervalDays * 24 * 60 * 60 * 1000);
  const now = new Date();
  
  const diffInMs = next.getTime() - now.getTime();
  const diffInDays = Math.ceil(diffInMs / (1000 * 60 * 60 * 24));
  
  return Math.max(0, diffInDays);
}

export function isOverdue(lastDate: string | null, intervalDays: number): boolean {
  if (!lastDate) return true;
  
  const last = new Date(lastDate);
  const due = new Date(last.getTime() + intervalDays * 24 * 60 * 60 * 1000);
  const now = new Date();
  
  return now > due;
}