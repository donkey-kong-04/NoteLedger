import { describe, it, expect } from 'vitest';
import { daysSince, openSinceLabel } from './types';
import type { Log } from './types';

function isoDaysAgo(n: number): string {
  const d = new Date();
  d.setDate(d.getDate() - n);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function log(start_date: string, is_closed = false): Log {
  return {
    id: 1, type_id: 1, title: 'T', description: '', closed_description: '', start_date,
    due_date: null, is_closed, closed_date: null, project_id: 1,
    category1_ids: [], category2_ids: [], category3_ids: [], category4_ids: [],
  };
}

describe('daysSince', () => {
  it('returns 0 for today and null/empty input', () => {
    expect(daysSince(isoDaysAgo(0))).toBe(0);
    expect(daysSince(null)).toBe(0);
    expect(daysSince('')).toBe(0);
  });

  it('counts whole days elapsed', () => {
    expect(daysSince(isoDaysAgo(1))).toBe(1);
    expect(daysSince(isoDaysAgo(10))).toBe(10);
  });
});

describe('openSinceLabel', () => {
  it('formats open logs with singular/plural/today', () => {
    expect(openSinceLabel(log(isoDaysAgo(0)))).toBe('Open since today');
    expect(openSinceLabel(log(isoDaysAgo(1)))).toBe('Open since 1 day');
    expect(openSinceLabel(log(isoDaysAgo(7)))).toBe('Open since 7 days');
  });

  it('is empty for closed logs', () => {
    expect(openSinceLabel(log(isoDaysAgo(5), true))).toBe('');
  });
});
