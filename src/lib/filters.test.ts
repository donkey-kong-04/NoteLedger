import { describe, it, expect } from 'vitest';
import {
  getDescendantIds,
  getAncestorIds,
  ancestorCatIds,
  logMatchesSlot,
  computeVisibility,
} from './filters';
import type { Log, Project } from './types';

function project(id: number, parent_id: number | null, extra: Partial<Project> = {}): Project {
  return {
    id,
    title: `P${id}`,
    description: '',
    parent_id,
    is_closed: false,
    start_date: null,
    end_date: null,
    category1_ids: [],
    category2_ids: [],
    category3_ids: [],
    category4_ids: [],
    ...extra,
  };
}

function log(id: number, project_id: number, extra: Partial<Log> = {}): Log {
  return {
    id,
    type_id: 1,
    title: `L${id}`,
    description: '',
    start_date: '2026-01-01',
    due_date: null,
    is_closed: false,
    closed_date: null,
    project_id,
    category1_ids: [],
    category2_ids: [],
    category3_ids: [],
    category4_ids: [],
    ...extra,
  };
}

// Tree: 1 ─ 2 ─ 3,  1 ─ 4
const tree: Project[] = [
  project(1, null),
  project(2, 1),
  project(3, 2),
  project(4, 1),
];

describe('tree traversal', () => {
  it('collects the full descendant subtree', () => {
    expect([...getDescendantIds(tree, 1)].sort()).toEqual([1, 2, 3, 4]);
    expect([...getDescendantIds(tree, 2)].sort()).toEqual([2, 3]);
    expect([...getDescendantIds(tree, 3)].sort()).toEqual([3]);
  });

  it('walks ancestors up to the root', () => {
    expect([...getAncestorIds(tree, 3)].sort()).toEqual([1, 2, 3]);
    expect([...getAncestorIds(tree, 1)].sort()).toEqual([1]);
  });
});

describe('category inheritance', () => {
  const projects = [
    project(1, null, { category1_ids: [10] }),
    project(2, 1, { category1_ids: [20] }),
  ];

  it('inherits ancestor category ids', () => {
    expect(ancestorCatIds(projects, 2, 1).sort()).toEqual([10, 20]);
  });

  it('matches a log when the value is inherited from its project', () => {
    const l = log(1, 2); // log on project 2, no own categories
    expect(logMatchesSlot(l, 1, [10], projects)).toBe(true); // 10 inherited from project 1
    expect(logMatchesSlot(l, 1, [99], projects)).toBe(false);
  });

  it('uses AND logic across selected values', () => {
    const l = log(1, 2, { category1_ids: [30] });
    expect(logMatchesSlot(l, 1, [20, 30], projects)).toBe(true); // 20 inherited, 30 own
    expect(logMatchesSlot(l, 1, [20, 40], projects)).toBe(false); // 40 absent
  });

  it('matches everything when nothing is selected', () => {
    expect(logMatchesSlot(log(1, 2), 1, [], projects)).toBe(true);
  });
});

describe('computeVisibility', () => {
  it('hides closed projects unless showClosed is set', () => {
    const projects = [project(1, null), project(2, null, { is_closed: true })];
    const open = computeVisibility(projects, [], null, false, true);
    expect(open.visible.has(2)).toBe(false);
    const withClosed = computeVisibility(projects, [], null, true, true);
    expect(withClosed.visible.has(2)).toBe(true);
  });

  it('shows ancestors of a selected project as context-only', () => {
    const { visible, ancestorOnly } = computeVisibility(tree, [], 3, false, true);
    expect(visible.has(3)).toBe(true);
    expect(ancestorOnly.has(1)).toBe(true);
    expect(ancestorOnly.has(2)).toBe(true);
    expect(ancestorOnly.has(3)).toBe(false);
  });

  it('with a category filter, only shows projects whose subtree has matching logs', () => {
    const logs = [log(1, 3)]; // a log deep under project 1
    const { visible } = computeVisibility(tree, logs, null, false, false);
    expect(visible.has(1)).toBe(true); // subtree contains the log
    expect(visible.has(4)).toBe(false); // empty branch hidden
  });
});
