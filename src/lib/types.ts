export type Density = 'compact' | 'normal' | 'comfortable';

export interface UserSettings {
  category1_label: string;
  category2_label: string;
  category3_label: string;
  category4_label: string;
  dark_mode: boolean;
  density: Density;
}

export interface PicklistValue {
  id: number;
  picklist_type: string;
  label: string;
}

export interface Project {
  id: number;
  title: string;
  description: string;
  parent_id: number | null;
  is_closed: boolean;
  is_template: boolean;
  start_date: string | null;
  end_date: string | null;
  category1_ids: number[];
  category2_ids: number[];
  category3_ids: number[];
  category4_ids: number[];
}

export interface ProjectLink {
  id: number;
  project_id: number;
  label: string;
  url: string;
}

export interface Log {
  id: number;
  type_id: number;
  title: string;
  description: string;
  start_date: string;
  due_date: string | null;
  is_closed: boolean;
  closed_date: string | null;
  project_id: number;
  category1_ids: number[];
  category2_ids: number[];
  category3_ids: number[];
  category4_ids: number[];
}

import { openUrl } from '@tauri-apps/plugin-opener';

// Only schemes that are safe to hand to the OS. Stored links and hrefs inside
// description HTML are user data, so anything else (file:, smb:, etc.) is refused.
const SAFE_PROTOCOLS = new Set(['http:', 'https:', 'mailto:']);

export async function openLink(url: string) {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    console.warn('openLink: not a valid URL, refusing to open:', url);
    return;
  }
  if (!SAFE_PROTOCOLS.has(parsed.protocol)) {
    console.warn('openLink: blocked non-http(s)/mailto URL:', url);
    return;
  }
  await openUrl(parsed.href);
}

export function handleLinkClick(e: MouseEvent) {
  const target = (e.target as HTMLElement).closest('a');
  if (!target) return;
  const href = target.getAttribute('href');
  if (!href) return;
  e.preventDefault();
  e.stopPropagation();
  openLink(href);
}

export const CAT_COLORS: Record<string, { hex: string; rgb: string }> = {
  category_1: { hex: '#6366f1', rgb: '99,102,241' },
  category_2: { hex: '#10b981', rgb: '16,185,129' },
  category_3: { hex: '#e11d48', rgb: '225,29,72' },
  category_4: { hex: '#06b6d4', rgb: '6,182,212' },
};

export function contrastText(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return lum > 0.45 ? '#000000' : '#ffffff';
}

// Returns projects in hierarchy order (parents sorted A-Z, children immediately below their parent, indented with spaces).
export function sortedProjectOptions(projects: Project[], excludeId?: number): { id: number; label: string }[] {
  const filtered = excludeId != null ? projects.filter(p => p.id !== excludeId) : projects;
  const result: { id: number; label: string }[] = [];
  function walk(parentId: number | null, depth: number) {
    const children = filtered
      .filter(p => (p.parent_id ?? null) === parentId)
      .sort((a, b) => a.title.localeCompare(b.title));
    for (const p of children) {
      result.push({ id: p.id, label: '    '.repeat(depth) + p.title });
      walk(p.id, depth + 1);
    }
  }
  walk(null, 0);
  return result;
}

// Whole days between an ISO date (YYYY-MM-DD) and today, in local time.
export function daysSince(dateStr: string | null): number {
  if (!dateStr) return 0;
  const [y, m, d] = dateStr.split('-').map(Number);
  if (!y || !m || !d) return 0;
  const start = new Date(y, m - 1, d);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  return Math.floor((today.getTime() - start.getTime()) / 86400000);
}

// "Open since today" / "Open since 1 day" / "Open since N days" — empty for closed logs.
export function openSinceLabel(log: Log): string {
  if (log.is_closed) return '';
  const d = daysSince(log.start_date);
  if (d <= 0) return 'Open since today';
  if (d === 1) return 'Open since 1 day';
  return `Open since ${d} days`;
}

export function deadlineColor(due: string): string {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const dueDate = new Date(due);
  // End of this calendar week (Sunday), or today if today is Sunday
  const endOfThisWeek = new Date(today);
  endOfThisWeek.setDate(today.getDate() + (7 - today.getDay()) % 7);
  // End of next calendar week (the Sunday after)
  const endOfNextWeek = new Date(endOfThisWeek);
  endOfNextWeek.setDate(endOfThisWeek.getDate() + 7);
  if (dueDate <= endOfThisWeek) return '#ef4444';
  if (dueDate <= endOfNextWeek) return '#f59e0b';
  return '#22c55e';
}
