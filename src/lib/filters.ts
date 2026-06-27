// Pure filtering/visibility helpers for the project + log tree.
// Kept free of Svelte/store imports so they can be unit-tested in isolation.
import type { Log, Project } from './types';

export type CatSlot = 1 | 2 | 3 | 4;

/** The project's own id plus every ancestor up the parent chain. */
export function getAncestorIds(projects: Project[], projectId: number): Set<number> {
  const ids = new Set<number>([projectId]);
  const p = projects.find((p) => p.id === projectId);
  if (p?.parent_id != null)
    for (const id of getAncestorIds(projects, Number(p.parent_id))) ids.add(id);
  return ids;
}

/** The project's own id plus every descendant in the subtree. */
export function getDescendantIds(projects: Project[], projectId: number): Set<number> {
  const ids = new Set<number>([projectId]);
  projects
    .filter((p) => p.parent_id != null && Number(p.parent_id) === projectId)
    .forEach((child) => getDescendantIds(projects, child.id).forEach((id) => ids.add(id)));
  return ids;
}

/** Category ids for a slot on a project, inherited down from its ancestors. */
export function ancestorCatIds(projects: Project[], projectId: number, slot: CatSlot): number[] {
  const p = projects.find((p) => p.id === projectId);
  if (!p) return [];
  const own = p[`category${slot}_ids`] as number[];
  if (p.parent_id != null)
    return [...own, ...ancestorCatIds(projects, Number(p.parent_id), slot)];
  return own;
}

/** A log matches a slot when every selected value is present on the log or inherited from its project. */
export function logMatchesSlot(
  l: Log,
  slot: CatSlot,
  sel: number[],
  projects: Project[]
): boolean {
  if (!sel.length) return true;
  const available = new Set([
    ...(l[`category${slot}_ids`] as number[]),
    ...ancestorCatIds(projects, Number(l.project_id), slot),
  ]);
  return sel.every((id) => available.has(id));
}

/** Whether any log lives anywhere in a project's subtree. */
export function subtreeHasLogs(projects: Project[], logs: Log[], projectId: number): boolean {
  const subtree = getDescendantIds(projects, projectId);
  return logs.some((l) => subtree.has(Number(l.project_id)));
}

export interface Visibility {
  visible: Set<number>;
  ancestorOnly: Set<number>;
}

/** Which projects are shown, and which are shown only as ancestor context (no logs). */
export function computeVisibility(
  projects: Project[],
  logs: Log[],
  selProject: number | null,
  showClosed: boolean,
  noCatFilter: boolean
): Visibility {
  const visible = new Set<number>();
  const ancestorOnly = new Set<number>();

  if (selProject !== null) {
    // Ancestor projects: shown as context without logs
    for (const aid of getAncestorIds(projects, selProject)) {
      if (aid !== selProject) {
        visible.add(aid);
        ancestorOnly.add(aid);
      }
    }
    // Selected project: always shown (ignore show-closed for itself)
    visible.add(selProject);
    // Descendants of selected project
    for (const did of getDescendantIds(projects, selProject)) {
      if (did === selProject) continue;
      const p = projects.find((p) => p.id === did);
      if (!p) continue;
      // No category filter: show all descendants (including closed) — user explicitly chose this project
      if (noCatFilter) {
        visible.add(did);
        continue;
      }
      // Category filter active: respect show-closed, only show if subtree has matching logs
      if (p.is_closed && !showClosed) continue;
      if (subtreeHasLogs(projects, logs, did)) visible.add(did);
    }
  } else {
    for (const p of projects) {
      if (p.is_closed && !showClosed) continue;
      if (noCatFilter || subtreeHasLogs(projects, logs, p.id)) visible.add(p.id);
    }
  }

  return { visible, ancestorOnly };
}
