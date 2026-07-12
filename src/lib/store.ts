import { writable } from 'svelte/store';
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

// Set before navigating to the homepage to have it clear its filters and
// focus the project filter on this project (used after cloning a template).
export const pendingProjectFocus = writable<number | null>(null);

// Filter state, shared between the Home and Table views so switching pages
// keeps the same filters applied.
export const showClosed = writable(false);
// UI-only: expand every log's "Closed Points" section instead of per-log clicks.
export const expandClosedPoints = writable(false);
export const selCat1 = writable<number[]>([]);
export const selCat2 = writable<number[]>([]);
export const selCat3 = writable<number[]>([]);
export const selCat4 = writable<number[]>([]);
export const selProject = writable<number | null>(null);
export const selLogType = writable<number | null>(null);

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

export async function deleteLog(id: number) {
  await invoke('delete_log', { id });
  const fresh = await invoke<Log[]>('get_logs');
  logs.set(fresh);
}
