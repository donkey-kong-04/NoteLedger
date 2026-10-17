import { describe, it, expect } from 'vitest';
import { dateKey, parseDateKey, isWeekend, addDays, mondayOf, sameWeekdayInWeekOf, monthWeeks, shownWeeks, groupByDate, monthIndexOf, monthIndexOfKey, fromMonthIndex } from './calendar';

describe('dateKey', () => {
  it('formats local dates as YYYY-MM-DD', () => {
    expect(dateKey(new Date(2026, 0, 5))).toBe('2026-01-05');
    expect(dateKey(new Date(2026, 11, 31))).toBe('2026-12-31');
  });
});

describe('addDays / mondayOf', () => {
  it('crosses month and year boundaries', () => {
    expect(dateKey(addDays(new Date(2026, 11, 30), 3))).toBe('2027-01-02');
    expect(dateKey(addDays(new Date(2026, 2, 1), -1))).toBe('2026-02-28');
  });

  it('finds the Monday of the week, Sunday belonging to the week before', () => {
    expect(dateKey(mondayOf(new Date(2026, 9, 17)))).toBe('2026-10-12'); // Saturday
    expect(dateKey(mondayOf(new Date(2026, 9, 18)))).toBe('2026-10-12'); // Sunday
    expect(dateKey(mondayOf(new Date(2026, 9, 19)))).toBe('2026-10-19'); // Monday
  });
});

describe('parseDateKey / isWeekend', () => {
  it('parses keys as local dates', () => {
    expect(dateKey(parseDateKey('2026-10-17'))).toBe('2026-10-17');
    expect(parseDateKey('2026-10-17').getDay()).toBe(6);
  });

  it('flags Saturdays and Sundays only', () => {
    expect(isWeekend(parseDateKey('2026-10-17'))).toBe(true); // Saturday
    expect(isWeekend(parseDateKey('2026-10-18'))).toBe(true); // Sunday
    expect(isWeekend(parseDateKey('2026-10-19'))).toBe(false); // Monday
    expect(isWeekend(parseDateKey('2026-10-16'))).toBe(false); // Friday
  });
});

describe('monthWeeks', () => {
  it('returns Monday–Sunday rows covering the month', () => {
    const weeks = monthWeeks(2026, 9); // October 2026 starts on a Thursday
    expect(weeks).toHaveLength(5);
    expect(weeks.every(w => w.length === 7)).toBe(true);
    expect(dateKey(weeks[0][0])).toBe('2026-09-28');
    expect(dateKey(weeks[4][6])).toBe('2026-11-01');
    expect(weeks.every(w => w[0].getDay() === 1 && w[6].getDay() === 0)).toBe(true);
  });

  it('includes the week of a Sunday 1st', () => {
    const weeks = monthWeeks(2026, 1); // February 2026 starts on a Sunday
    expect(dateKey(weeks[0][0])).toBe('2026-01-26');
    expect(dateKey(weeks[0][6])).toBe('2026-02-01');
    expect(dateKey(weeks[weeks.length - 1][5])).toBe('2026-02-28');
  });

  it('adds a sixth row when the month needs it', () => {
    expect(monthWeeks(2026, 2)).toHaveLength(6); // March 2026: Sun 1st → Tue 31st
  });
});

describe('groupByDate', () => {
  it('groups by key, keeping input order', () => {
    const map = groupByDate(
      [{ d: '2026-10-01', n: 1 }, { d: '2026-10-02', n: 2 }, { d: '2026-10-01', n: 3 }],
      x => x.d,
    );
    expect(map.get('2026-10-01')!.map(x => x.n)).toEqual([1, 3]);
    expect(map.get('2026-10-02')!.map(x => x.n)).toEqual([2]);
  });
});

describe('month indexes', () => {
  it('agree between dates and date keys', () => {
    expect(monthIndexOfKey('2026-10-17')).toBe(monthIndexOf(new Date(2026, 9, 17)));
    expect(monthIndexOfKey('2027-01-01') - monthIndexOfKey('2026-12-31')).toBe(1);
  });

  it('round-trip to year and 0-based month', () => {
    expect(fromMonthIndex(monthIndexOfKey('2026-01-05'))).toEqual({ year: 2026, month: 0 });
    expect(fromMonthIndex(monthIndexOfKey('2026-12-05'))).toEqual({ year: 2026, month: 11 });
  });
});

describe('sameWeekdayInWeekOf', () => {
  const today = new Date(2026, 9, 17); // Saturday — current week Mon 12 – Sun 18 Oct

  it('keeps the weekday, from any past week', () => {
    expect(sameWeekdayInWeekOf('2026-10-06', today)).toBe('2026-10-13'); // last week's Tuesday
    expect(sameWeekdayInWeekOf('2026-07-07', today)).toBe('2026-10-13'); // a Tuesday in July
    expect(sameWeekdayInWeekOf('2026-07-06', today)).toBe('2026-10-12'); // Monday
    expect(sameWeekdayInWeekOf('2026-07-12', today)).toBe('2026-10-18'); // Sunday stays Sunday
  });

  it('works across year boundaries', () => {
    expect(sameWeekdayInWeekOf('2025-12-31', today)).toBe('2026-10-14'); // a Wednesday
    expect(sameWeekdayInWeekOf('2026-12-29', new Date(2027, 0, 1))).toBe('2026-12-29'); // same week
  });
});

describe('shownWeeks', () => {
  const october = monthWeeks(2026, 9); // rows start Sep 28, Oct 5, 12, 19, 26
  const weekStart = '2026-10-12'; // current week: Mon 12 – Sun 18 Oct
  const mondays = (weeks: Date[][]) => weeks.map(w => dateKey(w[0]));

  it('keeps the current and future weeks, hides empty past ones', () => {
    expect(mondays(shownWeeks(october, 9, weekStart, new Set()))).toEqual(['2026-10-12', '2026-10-19', '2026-10-26']);
  });

  it('keeps a past week with a due date in it (weekends included)', () => {
    const due = new Set(['2026-10-11']); // last week's Sunday
    expect(mondays(shownWeeks(october, 9, weekStart, due))).toEqual(['2026-10-05', '2026-10-12', '2026-10-19', '2026-10-26']);
  });

  it("ignores a due date on the other month's part of a shared week", () => {
    const due = new Set(['2026-09-29']); // Tuesday of the Sep 28 row, but September's day
    expect(mondays(shownWeeks(october, 9, weekStart, due))).not.toContain('2026-09-28');
    expect(mondays(shownWeeks(monthWeeks(2026, 8), 8, weekStart, due))).toContain('2026-09-28');
  });
});
