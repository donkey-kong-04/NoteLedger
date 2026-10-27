import { writable, derived } from 'svelte/store';
import { parseSearchTokens, logMatchesSearch } from './search';
import { logMatchesSlot, computeVisibility } from './filters';
import { invoke } from '@tauri-apps/api/core';
import type { UserSettings, PicklistValue, Log, Project, ProjectLink } from './types';

export const settings = writable<UserSettings>({
  category1_label: 'Category 1',
  category2_label: 'Category 2',
  category3_label: 'Category 3',
  category4_label: 'Category 4',
  dark_mode: false,
  density: 'normal',
});

export const picklists = writable<PicklistValue[]>([]);
export const projects = writable<Project[]>([]);
export const logs = writable<Log[]>([]);
export const projectLinks = writable<ProjectLink[]>([]);

// Set before navigating to the Project view to have it clear its filters and
// focus the project filter on this project (used after cloning a template and
// by clicking a project name in the Calendar view).
export const pendingProjectFocus = writable<number | null>(null);

// Project view filter state; kept in stores so navigating to another page and
// back keeps the same filters applied. (The Calendar view has no filters.)
export const selCat1 = writable<number[]>([]);
export const selCat2 = writable<number[]>([]);
export const selCat3 = writable<number[]>([]);
export const selCat4 = writable<number[]>([]);
export const selProject = writable<number | null>(null);
export const selLogType = writable<number | null>(null);

// Always-visible log search (title / open points / closed points).
export const searchQuery = writable('');
export const searchTokens = derived(searchQuery, parseSearchTokens);
export const searchActive = derived(searchTokens, t => t.length > 0);

// True while any filter narrows what's shown (project, log type, categories,
// search). Closed projects/logs are hidden in the unfiltered view and always
// included while a filter is active.
export const filtersActive = derived(
  [selCat1, selCat2, selCat3, selCat4, selProject, selLogType, searchActive],
  ([c1, c2, c3, c4, p, t, s]) =>
    c1.length + c2.length + c3.length + c4.length > 0 || p !== null || t !== null || s
);

// ── Filtered Project view ───────────────────────────────────────────────────
// Matching/visibility helpers live in ./filters (pure + unit-tested).

// Template projects (and their logs) only appear on the Templates page.
export const liveProjects = derived(projects, ps => ps.filter(p => !p.is_template));

// True when no filter that narrows the visible logs is active — projects with
// no matching logs are then still shown.
export const noLogFilter = derived(
  [selCat1, selCat2, selCat3, selCat4, selLogType, searchTokens],
  ([c1, c2, c3, c4, t, s]) =>
    !c1.length && !c2.length && !c3.length && !c4.length && t === null && s.length === 0
);

// Logs passing the log filters (AND logic): log type, categories (inherited
// from ancestor projects) and the log search.
export const matchingLogs = derived(
  [logs, liveProjects, selCat1, selCat2, selCat3, selCat4, selLogType, searchTokens],
  ([ls, ps, c1, c2, c3, c4, t, s]) => ls.filter(l =>
    (t === null || l.type_id === t) &&
    logMatchesSlot(l, 1, c1, ps) &&
    logMatchesSlot(l, 2, c2, ps) &&
    logMatchesSlot(l, 3, c3, ps) &&
    logMatchesSlot(l, 4, c4, ps) &&
    logMatchesSearch(l, s)
  )
);

// Which projects show (and which only as ancestor context), given the project
// filter and the matching logs.
export const projectVisibility = derived(
  [liveProjects, matchingLogs, selProject, filtersActive, noLogFilter],
  ([ps, ls, sel, active, noLog]) => computeVisibility(ps, ls, sel, active, noLog)
);

// Resets every filter (the "Clear all" button, revealing new items).
export function clearFilters() {
  selCat1.set([]); selCat2.set([]); selCat3.set([]); selCat4.set([]);
  selProject.set(null);
  selLogType.set(null);
  searchQuery.set('');
}

// Whether the filter drawer is open; lives here so the app shell in
// +layout.svelte can shift the page content aside instead of being overlaid.
export const filterDrawerOpen = writable(false);

export async function loadAll() {
  const [s, p, pr, l, pl] = await Promise.all([
    invoke<UserSettings>('get_settings'),
    invoke<PicklistValue[]>('get_all_picklists'),
    invoke<Project[]>('get_projects'),
    invoke<Log[]>('get_logs'),
    invoke<ProjectLink[]>('get_project_links'),
  ]);
  settings.set(s);
  picklists.set(p);
  projects.set(pr);
  logs.set(l);
  projectLinks.set(pl);
}

export async function createProject(project: Omit<Project, 'id'>) {
  const created = await invoke<Project>('create_project', { project: { ...project, id: 0, category1_ids: project.category1_ids ?? [], category2_ids: project.category2_ids ?? [], category3_ids: project.category3_ids ?? [], category4_ids: project.category4_ids ?? [] } });
  const fresh = await invoke<Project[]>('get_projects');
  projects.set(fresh);
  return created;
}

export async function updateProject(project: Project) {
  await invoke('update_project', { project });
  const fresh = await invoke<Project[]>('get_projects');
  projects.set(fresh);
}

export async function deleteProject(id: number) {
  await invoke('delete_project', { id });
  const [freshP, freshL] = await Promise.all([
    invoke<Project[]>('get_projects'),
    invoke<Log[]>('get_logs'),
  ]);
  projects.set(freshP);
  logs.set(freshL);
}

// Deep-copies a template project tree (logs, links, sub-projects) into a
// fully independent non-template project. Returns the new root project id.
export async function cloneProject(id: number, newTitle: string): Promise<number> {
  const newId = await invoke<number>('clone_project', { id, newTitle });
  const [freshP, freshL, freshLinks] = await Promise.all([
    invoke<Project[]>('get_projects'),
    invoke<Log[]>('get_logs'),
    invoke<ProjectLink[]>('get_project_links'),
  ]);
  projects.set(freshP);
  logs.set(freshL);
  projectLinks.set(freshLinks);
  return newId;
}

export async function saveSettings(s: UserSettings) {
  await invoke('save_settings', { settings: s });
  settings.set(s);
}

export async function createPicklistValue(picklist_type: string, label: string) {
  const val = await invoke<PicklistValue>('create_picklist_value', { picklistType: picklist_type, label });
  // A null/malformed value in the store would crash every picklist consumer.
  if (!val?.id) throw new Error(`create_picklist_value returned no value for "${label}"`);
  picklists.update(p => [...p, val]);
  return val;
}

export async function updatePicklistValue(id: number, label: string) {
  await invoke('update_picklist_value', { id, label });
  picklists.update(p => p.map(v => v.id === id ? { ...v, label } : v));
}

export async function deletePicklistValue(id: number) {
  await invoke('delete_picklist_value', { id });
  picklists.update(p => p.filter(v => v.id !== id));
}

export async function createProjectLink(link: Omit<ProjectLink, 'id'>) {
  const created = await invoke<ProjectLink>('create_project_link', { link: { ...link, id: 0 } });
  projectLinks.update(l => [...l, created]);
  return created;
}

export async function updateProjectLink(link: ProjectLink) {
  await invoke('update_project_link', { link });
  projectLinks.update(l => l.map(x => x.id === link.id ? link : x));
}

export async function deleteProjectLink(id: number) {
  await invoke('delete_project_link', { id });
  projectLinks.update(l => l.filter(x => x.id !== id));
}

export async function createLog(log: Omit<Log, 'id' | 'start_date' | 'closed_date'>) {
  const created = await invoke<Log>('create_log', { log: { ...log, id: 0, start_date: '', closed_date: null, project_id: log.project_id, category1_ids: log.category1_ids ?? [], category2_ids: log.category2_ids ?? [], category3_ids: log.category3_ids ?? [], category4_ids: log.category4_ids ?? [] } });
  const fresh = await invoke<Log[]>('get_logs');
  logs.set(fresh);
  return created;
}

export async function updateLog(log: Log) {
  await invoke<Log>('update_log', { log });
  const fresh = await invoke<Log[]>('get_logs');
  logs.set(fresh);
}

// Calendar rescheduling (drag & drop, right-click, "move old projects"):
// changes only the project dates. Optimistic, so projects land on their new
// day at once; the reload then confirms (or puts back whatever failed to save).
export async function moveProjectsDates(moves: { project: Project; date: string }[]) {
  const byId = new Map(moves.map(m => [m.project.id, m.date]));
  projects.update(ps => ps.map(p => (byId.has(p.id) ? { ...p, date: byId.get(p.id)! } : p)));
  try {
    for (const { project, date } of moves) {
      await invoke('update_project', { project: { ...project, date } });
    }
  } finally {
    projects.set(await invoke<Project[]>('get_projects'));
  }
}

export function moveProjectDate(project: Project, date: string) {
  return moveProjectsDates([{ project, date }]);
}

// Calendar right-click on a log: changes only its due date, optimistically
// like the project moves above.
export async function moveLogDueDate(log: Log, dueDate: string) {
  logs.update(ls => ls.map(l => (l.id === log.id ? { ...l, due_date: dueDate } : l)));
  try {
    await invoke<Log>('update_log', { log: { ...log, due_date: dueDate } });
  } finally {
    logs.set(await invoke<Log[]>('get_logs'));
  }
}

export async function deleteLog(id: number) {
  await invoke('delete_log', { id });
  const fresh = await invoke<Log[]>('get_logs');
  logs.set(fresh);
}
