import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { UserSettings, PicklistValue, Log, Project } from './types';

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

export async function loadAll() {
  const [s, p, pr, l] = await Promise.all([
    invoke<UserSettings>('get_settings'),
    invoke<PicklistValue[]>('get_all_picklists'),
    invoke<Project[]>('get_projects'),
    invoke<Log[]>('get_logs'),
  ]);
  settings.set(s);
  picklists.set(p);
  projects.set(pr);
  logs.set(l);
}

export async function createProject(project: Omit<Project, 'id'>) {
  await invoke<Project>('create_project', { project: { ...project, id: 0, category1_ids: project.category1_ids ?? [], category2_ids: project.category2_ids ?? [], category3_ids: project.category3_ids ?? [], category4_ids: project.category4_ids ?? [] } });
  const fresh = await invoke<Project[]>('get_projects');
  projects.set(fresh);
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

export async function saveSettings(s: UserSettings) {
  await invoke('save_settings', { settings: s });
  settings.set(s);
}

export async function createPicklistValue(picklist_type: string, label: string) {
  const val = await invoke<PicklistValue>('create_picklist_value', { picklistType: picklist_type, label });
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

export async function createLog(log: Omit<Log, 'id' | 'start_date' | 'closed_date'>) {
  await invoke<Log>('create_log', { log: { ...log, id: 0, start_date: '', closed_date: null, project_id: log.project_id, category1_ids: log.category1_ids ?? [], category2_ids: log.category2_ids ?? [], category3_ids: log.category3_ids ?? [], category4_ids: log.category4_ids ?? [] } });
  const fresh = await invoke<Log[]>('get_logs');
  logs.set(fresh);
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
