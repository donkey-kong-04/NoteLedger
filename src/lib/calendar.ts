// Date helpers for the Calendar view. Dates are local 'YYYY-MM-DD' keys (the
// format of Log.due_date); weeks run Monday → Sunday.

export function dateKey(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

// Built from y/m/d parts, so DST changes never shift the day.
export function addDays(d: Date, n: number): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate() + n);
}

/** Local date of a 'YYYY-MM-DD' key (no UTC shift, unlike new Date(key)). */
export function parseDateKey(key: string): Date {
  const [y, m, d] = key.split('-').map(Number);
  return new Date(y, m - 1, d);
}

export function isWeekend(d: Date): boolean {
  return d.getDay() === 0 || d.getDay() === 6;
}

/** Monday of the week containing d. */
export function mondayOf(d: Date): Date {
  return addDays(d, -((d.getDay() + 6) % 7));
}

/** The key of the same weekday as `key`, in the week containing today. */
export function sameWeekdayInWeekOf(key: string, today: Date): string {
  const weekday = (parseDateKey(key).getDay() + 6) % 7; // Monday = 0
  return dateKey(addDays(mondayOf(today), weekday));
}

/** The key `weeks` weeks after `key` — same weekday (negative goes back). */
export function addWeeks(key: string, weeks: number): string {
  return dateKey(addDays(parseDateKey(key), 7 * weeks));
}

/** Monday–Sunday dates of every week that touches the month (month is 0-based). */
export function monthWeeks(year: number, month: number): Date[][] {
  const last = new Date(year, month + 1, 0);
  const weeks: Date[][] = [];
  for (let mon = mondayOf(new Date(year, month, 1)); mon <= last; mon = addDays(mon, 7)) {
    weeks.push(Array.from({ length: 7 }, (_, i) => addDays(mon, i)));
  }
  return weeks;
}

/** Months counted from year 0 (year * 12 + 0-based month), for month ranges. */
export function monthIndexOf(d: Date): number {
  return d.getFullYear() * 12 + d.getMonth();
}

/** Month index of a 'YYYY-MM-DD' key. */
export function monthIndexOfKey(key: string): number {
  return Number(key.slice(0, 4)) * 12 + Number(key.slice(5, 7)) - 1;
}

export function fromMonthIndex(index: number): { year: number; month: number } {
  return { year: Math.floor(index / 12), month: ((index % 12) + 12) % 12 };
}

/**
 * Weeks of a month block worth showing: the current week (starting on
 * `weekStart`, a Monday key) and later ones, plus past weeks with something
 * dated on one of this month's days. Empty past weeks are hidden — nothing
 * gets planned in the past.
 */
export function shownWeeks(weeks: Date[][], month: number, weekStart: string, datedKeys: Set<string>): Date[][] {
  return weeks.filter(w =>
    dateKey(w[0]) >= weekStart || w.some(d => d.getMonth() === month && datedKeys.has(dateKey(d))));
}

/** Groups items by a 'YYYY-MM-DD' key. */
export function groupByDate<T>(items: T[], key: (item: T) => string): Map<string, T[]> {
  const map = new Map<string, T[]>();
  for (const item of items) {
    const k = key(item);
    const list = map.get(k);
    if (list) list.push(item);
    else map.set(k, [item]);
  }
  return map;
}
