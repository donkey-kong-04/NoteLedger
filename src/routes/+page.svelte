<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import LogEditor from '$lib/components/LogEditor.svelte';
  import { settings, picklists, logs, loadAll, liveProjects, pendingProjectFocus, moveProjectDate, moveProjectsDates, moveLogDueDate } from '$lib/store';
  import type { Log, Project, PicklistValue } from '$lib/types';
  import { compareLogs, deadlineLevel } from '$lib/types';
  import { computeVisibility, displayedProjectIds } from '$lib/filters';
  import { densityStyle } from '$lib/density';
  import { dateKey, parseDateKey, isWeekend, addDays, mondayOf, sameWeekdayInWeekOf, addWeeks, monthWeeks, shownWeeks, groupByDate, monthIndexOf, monthIndexOfKey, fromMonthIndex } from '$lib/calendar';

  $: isDark = $settings.dark_mode;
  $: pageDensityStyle = densityStyle($settings.density);

  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');
  const byLabel = (a: PicklistValue, b: PicklistValue) => a.label.localeCompare(b.label);
  $: cat1Vals = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: cat2Vals = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: cat3Vals = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: cat4Vals = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  // ── Projects ──────────────────────────────────────────────────────────────
  // No filters on this page: every open project with at least one open log of
  // its own, placed on its date — except closed projects (or under a closed
  // ancestor — hidden like in the unfiltered Project view) and templates.
  // Each project's card lists its own open logs, whatever their due date
  // (sub-projects have their own card).
  $: shownProjectIds = displayedProjectIds($liveProjects, computeVisibility($liveProjects, $logs, null, false, true));
  $: openLogsByProject = openLogsOf($logs);
  $: calendarProjects = $liveProjects.filter(p => p.date && shownProjectIds.has(p.id) && openLogsByProject.has(p.id));

  // Each day's project cards, sorted A-Z by project path.
  type ProjectCard = { project: Project; path: string; logs: Log[] };
  $: cardsByDate = new Map(
    [...groupByDate(calendarProjects, p => p.date)]
      .map(([key, dayProjects]) => [key, projectCards(dayProjects, $liveProjects, openLogsByProject)] as const)
  );

  // Open logs per project id: by due date (no date last), titles A-Z on ties.
  function openLogsOf(allLogs: Log[]): Map<number, Log[]> {
    const byProject = new Map<number, Log[]>();
    const open = allLogs
      .filter(l => !l.is_closed)
      .sort((a, b) => a.title.localeCompare(b.title))
      .sort(compareLogs);
    for (const log of open) {
      const id = Number(log.project_id);
      const list = byProject.get(id);
      if (list) list.push(log);
      else byProject.set(id, [log]);
    }
    return byProject;
  }

  function projectCards(dayProjects: Project[], allProjects: Project[], logsByProject: Map<number, Log[]>): ProjectCard[] {
    return dayProjects
      .map(p => ({ project: p, path: projectPath(p.id, allProjects), logs: logsByProject.get(p.id) ?? [] }))
      .sort((a, b) => a.path.localeCompare(b.path));
  }

  // Log titles are colored by due date, like the Project view's deadline
  // pills: red = this week or earlier, yellow = next week, green = later.
  function dueClass(log: Log): string {
    return log.due_date ? `due-${deadlineLevel(log.due_date)}` : 'due-none';
  }

  function logTooltip(log: Log): string {
    return `${log.title}\n${log.due_date ? `Due ${fmtDay(log.due_date)}` : 'No due date'} · click to edit · right-click to move the log by weeks`;
  }

  // "Parent › Child" — takes the store snapshot so callers re-render on updates.
  function projectPath(projectId: number, allProjects: Project[]): string {
    const parts: string[] = [];
    let cur = allProjects.find(p => p.id === projectId);
    while (cur) {
      parts.unshift(cur.title);
      const parentId = cur.parent_id;
      cur = parentId != null ? allProjects.find(p => p.id === Number(parentId)) : undefined;
    }
    return parts.join(' › ');
  }

  // ── Weekends ──────────────────────────────────────────────────────────────
  // "Show weekends" is off by default and remembered per machine. Weekends
  // are only hidden while no displayed project falls on one: a project on a
  // Saturday or Sunday (often a wrong date) must never be out of sight.
  const WEEKDAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
  const WEEKENDS_PREF_KEY = 'noteledger.calendar.showWeekends';

  function readWeekendsPref(): boolean {
    try { return localStorage.getItem(WEEKENDS_PREF_KEY) === '1'; } catch { return false; }
  }
  function saveWeekendsPref(on: boolean) {
    try { localStorage.setItem(WEEKENDS_PREF_KEY, on ? '1' : '0'); } catch { /* convenience only */ }
  }

  let showWeekendsPref = readWeekendsPref();
  $: saveWeekendsPref(showWeekendsPref);

  $: hasWeekendProjects = calendarProjects.some(p => isWeekend(parseDateKey(p.date)));
  $: showWeekends = showWeekendsPref || hasWeekendProjects;
  $: cols = showWeekends ? 7 : 5;

  // ── Months shown ──────────────────────────────────────────────────────────
  // All months are stacked in one scrolling list, from the current week to
  // next month (or the latest project date, if later). The past only shows
  // where projects are: empty past weeks — even of the current month — and
  // empty past months are hidden.
  const now = new Date();
  const todayKey = dateKey(now);
  // The current week (Monday–Sunday) is highlighted across the grid.
  const weekStartKey = dateKey(mondayOf(now));
  const weekEndKey = dateKey(addDays(mondayOf(now), 6));
  const currentIndex = monthIndexOf(now);

  $: datedKeys = new Set(calendarProjects.map(p => p.date));
  $: datedIndexes = [...datedKeys].map(monthIndexOfKey).filter(Number.isFinite);
  // The current week may start in the previous month.
  $: firstIndex = Math.min(monthIndexOf(mondayOf(now)), ...datedIndexes);
  $: lastIndex = Math.max(currentIndex + 1, ...datedIndexes);

  $: months = Array.from({ length: lastIndex - firstIndex + 1 }, (_, i) => {
    const index = firstIndex + i;
    const { year, month } = fromMonthIndex(index);
    return {
      index, month,
      label: new Date(year, month, 1).toLocaleDateString('en-US', { month: 'long', year: 'numeric' }),
      // Visible columns only; with weekends hidden, a week whose only days in
      // this month fall on the weekend has nothing to show and is dropped.
      weeks: shownWeeks(monthWeeks(year, month), month, weekStartKey, datedKeys)
        .map(w => w.slice(0, cols))
        .filter(w => w.some(d => d.getMonth() === month)),
    };
  }).filter(m => m.weeks.length > 0);

  let mainEl: HTMLElement;
  let weekdayEl: HTMLElement;

  // Weeks/months appearing or disappearing above the viewport (projects
  // moved, added or closed) would shift what's on screen: keep the first visible
  // day where it was. Not before the initial scroll, which sets the position.
  let initialScrollDone = false;
  $: layoutKey = months.map(m => `${m.index}:${m.weeks.map(w => dateKey(w[0])).join('|')}`).join(',');
  $: keepScrollPosition(layoutKey);
  function keepScrollPosition(_layout: string) {
    if (!initialScrollDone) return;
    const top = mainEl.scrollTop + weekdayEl.offsetHeight;
    const anchor = [...mainEl.querySelectorAll<HTMLElement>('.day[data-date]')]
      .find(c => c.offsetTop + c.offsetHeight > top);
    if (!anchor) return;
    const key = anchor.dataset.date;
    const offset = anchor.offsetTop - mainEl.scrollTop;
    tick().then(() => {
      const moved = mainEl.querySelector<HTMLElement>(`.day[data-date="${key}"]`);
      if (moved) mainEl.scrollTop = moved.offsetTop - offset;
    });
  }

  // Scrolls so the month starts right under the sticky weekday header.
  function scrollToMonth(index: number, behavior: ScrollBehavior) {
    const section = mainEl?.querySelector<HTMLElement>(`section[data-month="${index}"]`);
    if (section) mainEl.scrollTo({ top: section.offsetTop - weekdayEl.offsetHeight, behavior });
  }

  // "This week" button: scrolls the current week to the middle of the view
  // and flashes its days so it's easy to spot, from anywhere in the list.
  let flashWeek = false;
  let flashTimer: ReturnType<typeof setTimeout>;
  function goToCurrentWeek() {
    // Today's card when visible (a hidden weekend day isn't), else the week's first one.
    const cell = mainEl.querySelector<HTMLElement>('.day.today')
      ?? mainEl.querySelector<HTMLElement>('.day.current-week');
    if (!cell) {
      scrollToMonth(currentIndex, 'smooth');
      return;
    }
    const top = cell.offsetTop - (mainEl.clientHeight - cell.offsetHeight) / 2;
    mainEl.scrollTo({ top: Math.max(0, top), behavior: 'smooth' });
    flashWeek = false;
    clearTimeout(flashTimer);
    tick().then(() => {
      flashWeek = true;
      flashTimer = setTimeout(() => { flashWeek = false; }, 1600);
    });
  }

  onMount(async () => {
    await loadAll();
    await tick();
    scrollToMonth(currentIndex, 'auto');
    initialScrollDone = true;
  });

  // ── Drag & drop (moves a project's date) ──────────────────────────────────
  // Pointer events rather than HTML5 drag & drop, which the Tauri webview can
  // swallow (Windows). A press anywhere on a project card (but its + button)
  // only becomes a drag past a few pixels, so a plain click still opens the
  // log editor or the Project view. Weekdays are the drop targets — weekends
  // (styled as read-only) and other months' blank cells are not. Dragging
  // near the top/bottom edge scrolls, to reach other months.
  const DRAG_START_PX = 5;
  const EDGE_PX = 56;

  type Drag = {
    project: Project;
    startX: number; startY: number;
    x: number; y: number;
    offsetX: number; offsetY: number; width: number;
    active: boolean; cancelled: boolean;
  };
  let drag: Drag | null = null;
  $: draggingId = drag?.active && !drag.cancelled ? drag.project.id : null;
  let dropKey: string | null = null;
  let suppressClick = false;
  let scrollRaf = 0;

  function onCardPointerDown(e: PointerEvent, project: Project) {
    if (e.button !== 0) return;
    // The ghost takes the card's width, held where grabbed.
    const card = (e.currentTarget as HTMLElement).getBoundingClientRect();
    drag = {
      project, startX: e.clientX, startY: e.clientY, x: e.clientX, y: e.clientY,
      offsetX: e.clientX - card.left, offsetY: e.clientY - card.top, width: card.width,
      active: false, cancelled: false,
    };
  }

  function onPointerMove(e: PointerEvent) {
    if (!drag || drag.cancelled) return;
    const moved = Math.hypot(e.clientX - drag.startX, e.clientY - drag.startY);
    if (!drag.active && moved < DRAG_START_PX) return;
    const starting = !drag.active;
    drag = { ...drag, x: e.clientX, y: e.clientY, active: true };
    if (starting) {
      window.getSelection()?.removeAllRanges();
      scrollRaf = requestAnimationFrame(autoScroll);
    }
    updateDropTarget();
  }

  function updateDropTarget() {
    if (!drag?.active || drag.cancelled) { dropKey = null; return; }
    const cell = document.elementFromPoint(drag.x, drag.y)?.closest<HTMLElement>('[data-drop]');
    dropKey = cell?.dataset.drop ?? null;
  }

  function autoScroll() {
    if (!drag?.active || drag.cancelled) return;
    const r = mainEl.getBoundingClientRect();
    const top = r.top + weekdayEl.offsetHeight + EDGE_PX; // below the sticky headers
    const bottom = r.bottom - EDGE_PX;
    let dy = 0;
    if (drag.y < top) dy = -Math.min(24, (top - drag.y) / 3 + 2);
    else if (drag.y > bottom) dy = Math.min(24, (drag.y - bottom) / 3 + 2);
    if (dy) {
      mainEl.scrollTop += dy;
      updateDropTarget();
    }
    scrollRaf = requestAnimationFrame(autoScroll);
  }

  function endDrag() {
    cancelAnimationFrame(scrollRaf);
    drag = null;
    dropKey = null;
  }

  function onPointerUp() {
    if (!drag) return;
    const { project, active, cancelled } = drag;
    const target = dropKey;
    endDrag();
    if (!active) return; // a plain click: the card's buttons handle it
    // The click that follows this pointerup must not open anything.
    suppressClick = true;
    setTimeout(() => { suppressClick = false; });
    if (!cancelled && target && target !== project.date) {
      moveProjectDate(project, target).catch(err => console.error('moveProjectDate failed:', err));
    }
  }

  function onWindowKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && cardMenu) {
      cardMenu = null;
      return;
    }
    if (e.key === 'Escape' && moveSource) {
      closeMoveDialog();
      return;
    }
    if (e.key === 'Escape' && drag?.active) {
      cancelAnimationFrame(scrollRaf);
      drag = { ...drag, cancelled: true };
      dropKey = null;
    }
  }

  function onLogClick(log: Log) {
    if (!suppressClick) openEdit(log);
  }

  // ── Right-click menu ──────────────────────────────────────────────────────
  // Moves what was right-clicked 1 or 2 weeks later (same weekday): a log row
  // → that log's due date (from today when it has none); the rest of a card
  // → the project's date. A transparent layer under the menu catches the
  // dismissing click, so it doesn't also open the editor or the Project view.
  const WEEK_MOVES = [
    { weeks: 1, label: 'Move to next week' },
    { weeks: 2, label: 'Move in 2 weeks' },
  ];
  type MenuTarget = { kind: 'project'; project: Project } | { kind: 'log'; log: Log };
  let cardMenu: { target: MenuTarget; x: number; y: number } | null = null;
  let cardMenuEl: HTMLDivElement;

  function menuBaseDate(target: MenuTarget): string {
    return target.kind === 'project' ? target.project.date : (target.log.due_date || todayKey);
  }

  function menuTitle(target: MenuTarget): string {
    return target.kind === 'project'
      ? `Project: ${projectPath(target.project.id, $liveProjects)}`
      : `Log: ${target.log.title}`;
  }

  async function openCardMenu(e: MouseEvent, target: MenuTarget) {
    if (drag?.active) return;
    cardMenu = { target, x: e.clientX, y: e.clientY };
    await tick();
    if (!cardMenu || !cardMenuEl) return;
    // Near the window's right/bottom edge, shift it back inside.
    const r = cardMenuEl.getBoundingClientRect();
    cardMenu = {
      ...cardMenu,
      x: Math.max(4, Math.min(cardMenu.x, window.innerWidth - r.width - 4)),
      y: Math.max(4, Math.min(cardMenu.y, window.innerHeight - r.height - 4)),
    };
    cardMenuEl.querySelector('button')?.focus();
  }

  function moveByWeeks(weeks: number) {
    if (!cardMenu) return;
    const { target } = cardMenu;
    cardMenu = null;
    const to = addWeeks(menuBaseDate(target), weeks);
    if (target.kind === 'project') {
      moveProjectDate(target.project, to).catch(err => console.error('moveProjectDate failed:', err));
    } else {
      moveLogDueDate(target.log, to).catch(err => console.error('moveLogDueDate failed:', err));
    }
  }

  onDestroy(() => {
    cancelAnimationFrame(scrollRaf);
    clearTimeout(flashTimer);
  });

  // ── Move old projects ─────────────────────────────────────────────────────
  // "Old" = the projects the calendar shows dated before the current week
  // (any past week). Each moves to the same weekday this week (a Tuesday
  // stays a Tuesday), after a confirmation dialog listing every move. Their
  // logs keep their due dates.
  type OldProject = { project: Project; path: string };
  $: oldProjects = calendarProjects
    .filter(p => p.date < weekStartKey)
    .map(p => ({ project: p, path: projectPath(p.id, $liveProjects) }))
    .sort((a, b) => a.project.date.localeCompare(b.project.date) || a.path.localeCompare(b.path));

  let moveSource: OldProject[] | null = null; // the confirmation is open while set
  let moving = false;
  let moveError = '';
  let moveConfirmBtn: HTMLButtonElement;

  $: moves = (moveSource ?? []).map(({ project, path }) =>
    ({ project, path, from: project.date, to: sameWeekdayInWeekOf(project.date, now) }));
  // Moved to a weekday this week that has already passed: still in the past.
  $: stillOverdue = moves.filter(m => m.to < todayKey).length;

  async function openMoveDialog() {
    // Snapshot, so the list can't change under the confirmation.
    moveSource = oldProjects;
    moveError = '';
    await tick();
    moveConfirmBtn?.focus();
  }

  function closeMoveDialog() {
    if (!moving) moveSource = null;
  }

  async function confirmMove() {
    if (!moveSource || moving) return;
    moving = true;
    moveError = '';
    try {
      await moveProjectsDates(moves.map(m => ({ project: m.project, date: m.to })));
      moving = false;
      moveSource = null;
    } catch (e) {
      moveError = `Some projects could not be moved: ${e}`;
      moving = false;
    }
  }

  function fmtDay(key: string): string {
    return parseDateKey(key).toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' });
  }

  // ── Editor ────────────────────────────────────────────────────────────────
  let editorLog: Log | null = null;
  let showEditor = false;
  function openEdit(log: Log) { editorLog = log; showEditor = true; }
  function closeEditor() { showEditor = false; editorLog = null; }

  // The + next to a project's name creates a log in that project. No due date
  // is preset: the editor asks for one before Create.
  function openNewLog(projectId: number) {
    editorLog = { due_date: null, type_id: logTypes[0]?.id ?? 0, project_id: projectId } as Log;
    showEditor = true;
  }

  // Clicking a project name: the Project view, filtered on that project.
  function openProjectView(projectId: number) {
    if (suppressClick) return;
    pendingProjectFocus.set(projectId);
    goto('/projects');
  }
</script>

<svelte:window
  on:pointermove={onPointerMove}
  on:pointerup={onPointerUp}
  on:pointercancel={endDrag}
  on:blur={endDrag}
  on:keydown={onWindowKeydown}
/>

<div class="page" class:dark={isDark} class:dragging={drag?.active && !drag.cancelled} style={pageDensityStyle}>
  <header class="page-header">
    <div class="header-left">
      <!-- Only offered while past weeks still hold projects. -->
      {#if oldProjects.length > 0}
        <button
          class="btn-secondary-sm move-old-btn" on:click={openMoveDialog}
          title="Move the projects dated before this week (before {fmtDay(weekStartKey)}) to the same weekday this week"
        >
          Move old projects
          <span class="count-pill">{oldProjects.length}</span>
        </button>
      {/if}
      <span class="legend" title="Log titles are colored by their due date">
        <span class="legend-label">Logs due:</span>
        <span class="due-red">this week</span>
        <span class="due-yellow">next week</span>
        <span class="due-green">later</span>
        <span class="due-none">no date</span>
      </span>
    </div>
    <div class="header-right">
      <label class="weekend-toggle" title="Saturdays and Sundays are always shown while a log falls on one, so nothing on a wrong date is missed">
        <input type="checkbox" bind:checked={showWeekendsPref} />
        Show weekends
      </label>
      <button class="btn-secondary-sm" on:click={goToCurrentWeek} title="Jump to the current week">This week</button>
    </div>
  </header>

  <main class="main" bind:this={mainEl} style="--cols: {cols}">
    <div class="weekdays" bind:this={weekdayEl}>
      {#each WEEKDAYS.slice(0, cols) as wd, i}
        <div class="weekday" class:weekend={i >= 5}>{wd}</div>
      {/each}
    </div>

    {#each months as m (m.index)}
      <section class="month" data-month={m.index}>
        <h2 class="month-title" class:current={m.index === currentIndex}>{m.label}</h2>
        <div class="month-grid">
          {#each m.weeks as week}
            {#each week as day}
              <!-- Days of other months stay blank: they belong to their own month block. -->
              {#if day.getMonth() === m.month}
                {@const key = dateKey(day)}
                {@const weekend = isWeekend(day)}
                {@const currentWeek = key >= weekStartKey && key <= weekEndKey}
                <div
                  class="day" class:weekend class:current-week={currentWeek} class:today={key === todayKey}
                  class:flash={flashWeek && currentWeek}
                  class:drop-target={dropKey === key}
                  data-date={key}
                  data-drop={weekend ? null : key}
                >
                  <span class="day-num">{day.getDate()}</span>
                  {#each cardsByDate.get(key) ?? [] as card (card.project.id)}
                    <!-- One card per project: its path on top (click: Project view; +: new
                         log in it), then its open logs, whatever their due date (click to
                         edit; titles colored by due date). Drag the card to move the
                         project. Right-click moves by weeks: a log row → that log, the
                         rest of the card → the project. -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div
                      class="project-card"
                      class:overdue={key < todayKey}
                      class:dragging={card.project.id === draggingId}
                      on:pointerdown={e => onCardPointerDown(e, card.project)}
                      on:contextmenu|preventDefault|stopPropagation={e => openCardMenu(e, { kind: 'project', project: card.project })}
                    >
                      <div class="card-head">
                        <button
                          class="card-project"
                          title="Open {card.path} in the Project view&#10;Drag to move the project · right-click to move it by weeks"
                          on:click={() => openProjectView(card.project.id)}
                        >{card.path}</button>
                        <button
                          class="card-add" title="New log in {card.path}" aria-label="New log in {card.path}"
                          on:pointerdown={e => e.stopPropagation()}
                          on:click={() => openNewLog(card.project.id)}
                        >+</button>
                      </div>
                      {#each card.logs as log (log.id)}
                        <button
                          class="card-log {dueClass(log)}"
                          title={logTooltip(log)}
                          on:contextmenu|preventDefault|stopPropagation={e => openCardMenu(e, { kind: 'log', log })}
                          on:click={() => onLogClick(log)}
                        >{log.title}</button>
                      {/each}
                    </div>
                  {/each}
                </div>
              {:else}
                <div class="day blank"></div>
              {/if}
            {/each}
          {/each}
        </div>
      </section>
    {/each}
  </main>

  {#if drag?.active && !drag.cancelled}
    {@const logCount = openLogsByProject.get(drag.project.id)?.length ?? 0}
    <div
      class="project-card drag-ghost" class:overdue={drag.project.date < todayKey}
      style="left: {drag.x - drag.offsetX}px; top: {drag.y - drag.offsetY}px; width: {drag.width}px;"
    >
      <span class="ghost-project">{projectPath(drag.project.id, $liveProjects)}</span>
      <span class="ghost-count">{logCount} open log{logCount !== 1 ? 's' : ''}</span>
    </div>
  {/if}

  {#if cardMenu}
    <div
      class="menu-layer" role="presentation"
      on:pointerdown={() => cardMenu = null}
      on:contextmenu|preventDefault={() => cardMenu = null}
      on:wheel={() => cardMenu = null}
    ></div>
    <div class="card-menu" role="menu" bind:this={cardMenuEl} style="left: {cardMenu.x}px; top: {cardMenu.y}px;">
      <div class="menu-title">{menuTitle(cardMenu.target)}</div>
      {#each WEEK_MOVES as mv}
        <button role="menuitem" on:click={() => moveByWeeks(mv.weeks)}>
          {mv.label}
          <span class="menu-date">{fmtDay(addWeeks(menuBaseDate(cardMenu.target), mv.weeks))}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if moveSource}
    <div class="backdrop" on:click={closeMoveDialog} role="presentation"></div>
    <div class="move-dialog" role="dialog" aria-modal="true" aria-labelledby="move-dialog-title">
      <h2 id="move-dialog-title">Move old projects to this week?</h2>
      <p class="dialog-hint">
        Are you sure? Projects dated before this week (before {fmtDay(weekStartKey)}) will move to the same
        weekday this week. Their logs keep their due dates.
      </p>
      <ul class="move-list">
        {#each moves as mv (mv.project.id)}
          <li>
            <span class="move-title">{mv.path}</span>
            <span class="move-dates" class:overdue={mv.to < todayKey}>{fmtDay(mv.from)} → {fmtDay(mv.to)}</span>
          </li>
        {/each}
      </ul>
      {#if stillOverdue > 0}
        <p class="dialog-note">
          {stillOverdue} project{stillOverdue !== 1 ? 's' : ''} (in red) will land on a day of this week that has already
          passed, so {stillOverdue !== 1 ? 'they stay' : 'it stays'} in the past.
        </p>
      {/if}
      {#if moveError}
        <p class="dialog-error">{moveError}</p>
      {/if}
      <div class="dialog-actions">
        <button class="btn-secondary-sm" on:click={closeMoveDialog} disabled={moving}>Cancel</button>
        <button class="btn-primary" bind:this={moveConfirmBtn} on:click={confirmMove} disabled={moving}>
          {moving ? 'Moving…' : `Move ${moves.length} project${moves.length !== 1 ? 's' : ''}`}
        </button>
      </div>
    </div>
  {/if}

  {#if showEditor}
    <LogEditor
      log={editorLog}
      {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      cat1Label={$settings.category1_label} cat2Label={$settings.category2_label}
      cat3Label={$settings.category3_label} cat4Label={$settings.category4_label}
      allProjects={$liveProjects}
      on:close={closeEditor}
    />
  {/if}
</div>

<style>
  .page {
    /* Weekends look like a read-only input: darker, muted, not a drop target. */
    --weekend-bg:   #dde1e7;
    --weekend-text: #9ca3af;
    --current-week-bg: rgba(99,102,241,0.08);
    /* Log due-date text colors: the Project view's deadline red/yellow/green,
       darkened (light) or lightened (dark) so titles stay readable. */
    --due-red:    #dc2626;
    --due-yellow: #b45309;
    --due-green:  #15803d;
    display: flex; flex-direction: column;
    height: 100%; overflow: hidden;
    background: var(--surface-2); color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .page.dark {
    --weekend-bg:   #0e1420;
    --weekend-text: #6b7280;
    --current-week-bg: rgba(99,102,241,0.15);
    --due-red:    #f87171;
    --due-yellow: #fbbf24;
    --due-green:  #4ade80;
  }

  .page-header {
    display: flex; align-items: center; justify-content: space-between; gap: 16px;
    padding: 8px 20px; min-height: 52px;
    background: var(--surface); border-bottom: 1px solid var(--border);
    flex-shrink: 0; z-index: 10;
  }
  .header-left { display: flex; align-items: center; gap: 14px; min-width: 0; flex: 1; }
  .header-right { display: flex; align-items: center; gap: 14px; flex-shrink: 0; }

  .weekend-toggle {
    display: inline-flex; align-items: center; gap: 6px;
    font-size: 13px; font-weight: 600; color: var(--text-muted);
    cursor: pointer; user-select: none;
  }
  .weekend-toggle input { accent-color: var(--accent); cursor: pointer; margin: 0; }

  .legend {
    display: inline-flex; align-items: baseline; gap: 10px;
    font-size: 12px; font-weight: 600; white-space: nowrap;
  }
  .legend-label { color: var(--text-muted); font-weight: 500; }


  .btn-secondary-sm {
    background: none; color: var(--text-muted);
    border: 1px solid var(--border); border-radius: 10px;
    padding: 8px 14px; font-size: 14px; font-weight: 600; cursor: pointer;
    font-family: inherit; transition: background 0.15s, color 0.15s;
  }
  .btn-secondary-sm:hover { background: var(--surface-2); color: var(--text); }
  .btn-secondary-sm:disabled { opacity: 0.5; cursor: default; }
  .btn-secondary-sm:disabled:hover { background: none; color: var(--text-muted); }

  .move-old-btn { display: inline-flex; align-items: center; gap: 6px; white-space: nowrap; }
  .count-pill {
    min-width: 18px; padding: 0 5px; box-sizing: border-box;
    background: var(--accent); color: #fff;
    border-radius: 999px;
    font-size: 11px; font-weight: 700; line-height: 18px; text-align: center;
  }

  /* Project right-click menu. */
  .menu-layer { position: fixed; inset: 0; z-index: 200; }
  .card-menu {
    position: fixed; z-index: 201;
    min-width: 220px;
    display: flex; flex-direction: column;
    padding: 4px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 6px 18px rgba(0,0,0,0.25);
  }
  .card-menu button {
    display: flex; align-items: baseline; justify-content: space-between; gap: 16px;
    padding: 6px 10px;
    background: none; border: none; border-radius: 5px;
    font-family: inherit; font-size: 13px; color: var(--text);
    text-align: left; cursor: pointer;
  }
  .card-menu button:hover, .card-menu button:focus-visible { background: var(--surface-2); outline: none; }
  .menu-date { font-size: 12px; color: var(--text-muted); white-space: nowrap; }
  /* Says what the moves apply to: the right-clicked log or project. */
  .menu-title {
    max-width: 280px;
    padding: 5px 10px 6px; margin-bottom: 2px;
    border-bottom: 1px solid var(--border);
    font-size: 12px; font-weight: 600; color: var(--text-muted);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  /* Confirmation dialog — same look as the Templates clone dialog. */
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(2px);
    z-index: 100;
  }
  .move-dialog {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    width: min(480px, 90vw);
    max-height: 80vh;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 22px 24px;
    z-index: 101;
    display: flex; flex-direction: column; gap: 10px;
    box-shadow: 0 20px 60px rgba(0,0,0,0.25);
  }
  .move-dialog h2 { margin: 0; font-size: 16px; font-weight: 700; color: var(--text); }
  .dialog-hint { margin: 0; font-size: 12px; color: var(--text-muted); line-height: 1.5; }
  .move-list {
    list-style: none; margin: 0; padding: 0;
    min-height: 0; overflow-y: auto;
    border: 1px solid var(--border); border-radius: 8px;
  }
  .move-list li {
    display: flex; align-items: baseline; justify-content: space-between; gap: 12px;
    padding: 7px 12px;
    font-size: 13px;
  }
  .move-list li + li { border-top: 1px solid var(--border); }
  .move-title { color: var(--text); overflow-wrap: anywhere; }
  .move-dates { flex-shrink: 0; font-size: 12px; color: var(--text-muted); white-space: nowrap; }
  .move-dates.overdue { color: #ef4444; }

  .dialog-note {
    margin: 0; font-size: 12px; line-height: 1.5; color: #b45309;
    background: rgba(245,158,11,0.12); border-radius: 6px; padding: 6px 10px;
  }

  .dialog-error {
    margin: 0; font-size: 12px; color: #ef4444;
    background: rgba(239,68,68,0.08); border-radius: 6px; padding: 6px 10px;
  }
  .dialog-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  .btn-primary {
    background: var(--accent); color: #fff; border: none; border-radius: 10px;
    padding: 8px 16px; font-size: 14px; font-weight: 600; cursor: pointer;
    font-family: inherit; transition: opacity 0.15s;
  }
  .btn-primary:hover { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.5; cursor: default; }

  /* No top padding: the sticky weekday header sits flush with the top edge. */
  .main {
    position: relative; /* offsetTop of month sections is measured from here */
    flex: 1; min-height: 0;
    overflow-y: auto;
    padding: 0 var(--sp-main-pad) var(--sp-main-pad);
    -webkit-user-select: none; user-select: none; /* drags must not select text */
  }

  /* Weekday header (sticky) and month grids share the same columns:
     Mon–Fri, or Mon–Sun when weekends are shown. */
  .weekdays, .month-grid {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
    gap: 4px;
  }

  .weekdays {
    position: sticky; top: 0; z-index: 3;
    box-sizing: border-box; height: 32px; align-items: center;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }
  .weekday {
    padding: 0 8px;
    font-size: 11px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--text-muted);
  }
  .weekday.weekend { color: var(--weekend-text); }

  .month { padding-bottom: 12px; }

  /* Sticks under the weekday header until the next month pushes it away. */
  .month-title {
    position: sticky; top: 32px; z-index: 2;
    margin: 0; padding: 12px 2px 8px;
    background: var(--surface-2);
    font-size: 15px; font-weight: 700; color: var(--text);
  }
  .month-title.current { color: var(--accent); }

  .month-grid { grid-auto-rows: minmax(96px, auto); }

  .day {
    background: var(--card-bg);
    border: 1px solid var(--border); border-radius: 6px;
    padding: 6px;
    display: flex; flex-direction: column; gap: 3px;
    min-width: 0;
    transition: background 0.1s, border-color 0.1s;
  }
  .day.blank { background: none; border-color: transparent; }
  .day.weekend { background: var(--weekend-bg); }
  .day.weekend .day-num { color: var(--weekend-text); }
  /* The current week stands out: accent tint + border on all its days
     (weekend days keep their read-only shade under the tint); today gets
     the strongest outline. */
  .day.current-week {
    border-color: rgba(99,102,241,0.6);
    background: linear-gradient(var(--current-week-bg), var(--current-week-bg)), var(--card-bg);
  }
  .day.current-week.weekend {
    background: linear-gradient(var(--current-week-bg), var(--current-week-bg)), var(--weekend-bg);
  }
  .day.today { border-color: var(--accent); box-shadow: inset 0 0 0 1px var(--accent); }
  .day.flash { animation: flash-week 1.6s ease-out; }
  @keyframes flash-week {
    0%, 30% { box-shadow: 0 0 0 4px rgba(99,102,241,0.55); }
    100% { box-shadow: 0 0 0 4px rgba(99,102,241,0); }
  }
  .day.drop-target {
    border-color: var(--accent);
    background: rgba(99,102,241,0.10);
    box-shadow: inset 0 0 0 1px var(--accent);
  }

  .day-num {
    align-self: flex-start;
    font-size: 12px; font-weight: 600; color: var(--text-muted);
    padding: 1px 4px; margin-bottom: 2px;
  }
  .day.today .day-num {
    background: var(--accent); color: #fff;
    border-radius: 999px; padding: 1px 7px;
  }

  /* One card per project: tinted, colored left bar; the project path and its
     + on top, then one row per open log, separated by thin lines. Titles wrap
     over as many lines as needed (never truncated). */
  .project-card {
    --chip-tint:  rgba(99,102,241,0.12);
    --chip-hover: rgba(99,102,241,0.14);
    --chip-line:  rgba(99,102,241,0.28);
    display: block; width: 100%; box-sizing: border-box;
    background: var(--chip-tint); color: var(--text);
    border-left: 3px solid var(--accent); border-radius: 4px;
    overflow: hidden; /* clips the row hover backgrounds */
    transition: opacity 0.15s;
  }
  .project-card + .project-card { margin-top: 3px; }
  /* A project dated before today: tinted red until it's moved. */
  .project-card.overdue {
    --chip-tint:  rgba(239,68,68,0.12);
    --chip-hover: rgba(239,68,68,0.14);
    --chip-line:  rgba(239,68,68,0.28);
    border-left-color: #ef4444;
  }
  .project-card.dragging { opacity: 0.35; }

  .card-head { display: flex; align-items: flex-start; }
  .card-project, .card-log {
    display: block; width: 100%; box-sizing: border-box;
    background: none; border: none;
    text-align: left; font-family: inherit; color: inherit;
    white-space: normal; overflow-wrap: anywhere;
    cursor: pointer;
  }
  .card-project {
    flex: 1; min-width: 0;
    padding: 5px 4px 2px 8px;
    font-size: 12px; font-weight: 600; line-height: 1.3;
    color: var(--text-muted);
  }
  .card-project:hover { color: var(--accent); text-decoration: underline; }
  .card-add {
    flex-shrink: 0;
    width: 20px; height: 20px; margin: 3px 3px 0 0;
    display: flex; align-items: center; justify-content: center;
    background: none; color: var(--text-muted);
    border: 1px solid var(--chip-line); border-radius: 5px;
    font-size: 14px; font-weight: 600; line-height: 1; font-family: inherit;
    cursor: pointer;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
  }
  .card-add:hover, .card-add:focus-visible {
    background: var(--accent); border-color: var(--accent); color: #fff; outline: none;
  }
  .page.dragging .card-add { visibility: hidden; }
  .card-log {
    padding: 2px 8px 5px;
    font-size: 14px; line-height: 1.35;
    transition: background 0.15s;
  }
  .card-log + .card-log { border-top: 1px solid var(--chip-line); padding-top: 5px; }
  .card-log:hover { background: var(--chip-hover); }

  /* Due-date colors (log titles and the header legend). */
  .due-red    { color: var(--due-red); }
  .due-yellow { color: var(--due-yellow); }
  .due-green  { color: var(--due-green); }
  .due-none   { color: var(--text-muted); }

  /* The dragged copy shows the project and how many logs come along. */
  .ghost-project {
    display: block; margin-bottom: 1px;
    font-size: 12px; font-weight: 600; line-height: 1.3;
    color: var(--text-muted);
  }
  .ghost-count { display: block; font-size: 13px; line-height: 1.35; }

  /* Follows the pointer while dragging; opaque so it reads over any cell. */
  .drag-ghost {
    position: fixed; z-index: 1000;
    pointer-events: none;
    padding: 5px 8px;
    background: var(--card-bg);
    box-shadow: 0 6px 18px rgba(0,0,0,0.25);
    transform: rotate(-1.5deg);
  }

  .page.dragging, .page.dragging :global(*) { cursor: grabbing !important; }
  .page.dragging .day.weekend,
  .page.dragging .day.weekend :global(*),
  .page.dragging .day.blank { cursor: not-allowed !important; }
</style>
