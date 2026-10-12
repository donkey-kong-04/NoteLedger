import { describe, it, expect } from 'vitest';
import { daysSince, openSinceLabel, compareLogs } from './types';
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

describe('compareLogs', () => {
  const mk = (id: number, due_date: string | null, is_closed: boolean): Log =>
    ({ ...log('2026-01-01', is_closed), id, due_date });

  it('orders open by due ASC, then closed by due DESC, undated last in each group', () => {
    const logs = [
      mk(1, '2026-03-01', true),
      mk(2, '2026-05-01', false),
      mk(3, null, true),
      mk(4, '2026-04-01', true),
      mk(5, null, false),
      mk(6, '2026-02-01', false),
    ];
    expect([...logs].sort(compareLogs).map(l => l.id)).toEqual([6, 2, 5, 4, 1, 3]);
  });
});
