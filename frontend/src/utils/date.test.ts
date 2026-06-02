import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { formatDate, formatDateTime, formatRelativeTime, calculateDaysUntil, isOverdue } from './date';

describe('formatDate', () => {
  it('formats ISO date string to en-GB date', () => {
    const result = formatDate('2024-06-15T10:00:00Z');
    expect(result).toMatch(/15\/06\/2024/);
  });

  it('accepts Date object', () => {
    const result = formatDate(new Date('2024-01-01T00:00:00Z'));
    expect(result).toMatch(/01\/01\/2024/);
  });
});

describe('formatDateTime', () => {
  it('includes time component', () => {
    const result = formatDateTime('2024-06-15T14:30:00Z');
    // Should contain date and time
    expect(result).toMatch(/15\/06\/2024/);
    expect(result).toMatch(/\d{1,2}:\d{2}/);
  });
});

describe('formatRelativeTime', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-06-20T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns "Today" for same day', () => {
    expect(formatRelativeTime('2024-06-20T08:00:00Z')).toBe('Today');
  });

  it('returns "Yesterday" for one day ago', () => {
    expect(formatRelativeTime('2024-06-19T08:00:00Z')).toBe('Yesterday');
  });

  it('returns "X days ago" for 2-6 days', () => {
    expect(formatRelativeTime('2024-06-17T08:00:00Z')).toBe('3 days ago');
  });

  it('returns "1 week ago" for 7 days', () => {
    expect(formatRelativeTime('2024-06-13T08:00:00Z')).toBe('1 week ago');
  });

  it('returns "X weeks ago" for multiple weeks', () => {
    expect(formatRelativeTime('2024-06-01T08:00:00Z')).toBe('2 weeks ago');
  });

  it('returns "1 month ago" for 30+ days', () => {
    expect(formatRelativeTime('2024-05-20T08:00:00Z')).toBe('1 month ago');
  });

  it('returns "X months ago" for multiple months', () => {
    expect(formatRelativeTime('2024-03-20T08:00:00Z')).toBe('3 months ago');
  });
});

describe('calculateDaysUntil', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-06-20T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns 0 when lastDate is null', () => {
    expect(calculateDaysUntil(null, 7)).toBe(0);
  });

  it('calculates days remaining until next due', () => {
    // Last performed June 18, interval 7 days → due June 25 → 5 days from June 20
    expect(calculateDaysUntil('2024-06-18T10:00:00Z', 7)).toBe(5);
  });

  it('returns 0 when overdue', () => {
    // Last performed June 10, interval 7 days → due June 17 → overdue
    expect(calculateDaysUntil('2024-06-10T10:00:00Z', 7)).toBe(0);
  });
});

describe('isOverdue', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-06-20T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns true when lastDate is null (never performed)', () => {
    expect(isOverdue(null, 7)).toBe(true);
  });

  it('returns true when past due', () => {
    // Last performed June 10, interval 7 → due June 17 → overdue on June 20
    expect(isOverdue('2024-06-10T10:00:00Z', 7)).toBe(true);
  });

  it('returns false when not yet due', () => {
    // Last performed June 18, interval 7 → due June 25 → not overdue on June 20
    expect(isOverdue('2024-06-18T10:00:00Z', 7)).toBe(false);
  });

  it('returns true when exactly due (boundary)', () => {
    // Last performed June 13, interval 7 → due June 20 at 10:00
    // Current time is June 20 at 12:00 → past due
    expect(isOverdue('2024-06-13T10:00:00Z', 7)).toBe(true);
  });
});
