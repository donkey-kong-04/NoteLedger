export interface UserSettings {
  category1_label: string;
  category2_label: string;
  category3_label: string;
  category4_label: string;
  dark_mode: boolean;
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
  project_id: number | null;
  category1_ids: number[];
  category2_ids: number[];
  category3_ids: number[];
  category4_ids: number[];
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

export function deadlineColor(due: string): string {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const dueDate = new Date(due);
  const weekEnd = new Date(today);
  weekEnd.setDate(weekEnd.getDate() + 7);
  if (dueDate < today) return '#ef4444';
  if (dueDate <= weekEnd) return '#f59e0b';
  return '#22c55e';
}
