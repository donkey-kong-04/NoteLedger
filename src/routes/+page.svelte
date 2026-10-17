<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import LogEditor from '$lib/components/LogEditor.svelte';
  import { settings, picklists, logs, loadAll, liveProjects, pendingProjectFocus, moveLogDueDate, moveLogsDueDates } from '$lib/store';
  import type { Log, Project, PicklistValue } from '$lib/types';
  import { computeVisibility, displayedProjectIds } from '$lib/filters';
  import { densityStyle } from '$lib/density';
  import { dateKey, parseDateKey, isWeekend, addDays, mondayOf, sameWeekdayInWeekOf, monthWeeks, shownWeeks, groupByDate, monthIndexOf, monthIndexOfKey, fromMonthIndex } from '$lib/calendar';

  $: isDark = $settings.dark_mode;
  $: pageDensityStyle = densityStyle($settings.density);

  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');
  const byLabel = (a: PicklistValue, b: PicklistValue) => a.label.localeCompare(b.label);
  $: cat1Vals = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: cat2Vals = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: cat3Vals = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: cat4Vals = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  // ── Logs ──────────────────────────────────────────────────────────────────
  // No filters on this page: every open log with a deadline, placed on its
  // due date — except in closed projects (hidden like in the unfiltered
  // Project view) and templates.
  $: shownProjectIds = displayedProjectIds($liveProjects, computeVisibility($liveProjects, $logs, null, false, true));
  $: calendarLogs = $logs
    .filter(l => !l.is_closed && l.due_date && shownProjectIds.has(Number(l.project_id)))
    .sort((a, b) => a.title.localeCompare(b.title));

  // Each day's logs grouped by project — sorted A-Z by project path, titles
  // A-Z within a project — so a project shows once with its logs below it.
  type ProjectGroup = { projectId: number; path: string; logs: Log[] };
  $: groupsByDate = new Map(
    [...groupByDate(calendarLogs, l => l.due_date ?? '')]
      .map(([key, dayLogs]) => [key, groupByProject(dayLogs, $liveProjects)])
  );

  function groupByProject(dayLogs: Log[], allProjects: Project[]): ProjectGroup[] {
    const groups = new Map<number, ProjectGroup>();
    for (const log of dayLogs) {
      const id = Number(log.project_id);
      let group = groups.get(id);
      if (!group) groups.set(id, group = { projectId: id, path: projectPath(id, allProjects), logs: [] });
      group.logs.push(log);
    }
    return [...groups.values()].sort((a, b) => a.path.localeCompare(b.path));
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
  // are only hidden while no displayed log falls on one: a log on a Saturday
  // or Sunday (often a wrong date) must never be out of sight.
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

  $: hasWeekendLogs = calendarLogs.some(l => isWeekend(parseDateKey(l.due_date ?? '')));
  $: showWeekends = showWeekendsPref || hasWeekendLogs;
  $: cols = showWeekends ? 7 : 5;

  // ── Months shown ──────────────────────────────────────────────────────────
  // All months are stacked in one scrolling list, from the current week to
  // next month (or the latest open deadline, if later). The past only shows
  // where open logs are: empty past weeks — even of the current month — and
  // empty past months are hidden.
  const now = new Date();
  const todayKey = dateKey(now);
  // The current week (Monday–Sunday) is highlighted across the grid.
  const weekStartKey = dateKey(mondayOf(now));
  const weekEndKey = dateKey(addDays(mondayOf(now), 6));
  const currentIndex = monthIndexOf(now);

  $: openDueKeys = new Set(calendarLogs.map(l => l.due_date ?? ''));
  $: dueIndexes = [...openDueKeys].map(monthIndexOfKey).filter(Number.isFinite);
  // The current week may start in the previous month.
  $: firstIndex = Math.min(monthIndexOf(mondayOf(now)), ...dueIndexes);
  $: lastIndex = Math.max(currentIndex + 1, ...dueIndexes);

  $: months = Array.from({ length: lastIndex - firstIndex + 1 }, (_, i) => {
    const index = firstIndex + i;
    const { year, month } = fromMonthIndex(index);
    return {
      index, month,
      label: new Date(year, month, 1).toLocaleDateString('en-US', { month: 'long', year: 'numeric' }),
      // Visible columns only; with weekends hidden, a week whose only days in
      // this month fall on the weekend has nothing to show and is dropped.
      weeks: shownWeeks(monthWeeks(year, month), month, weekStartKey, openDueKeys)
        .map(w => w.slice(0, cols))
        .filter(w => w.some(d => d.getMonth() === month)),
    };
  }).filter(m => m.weeks.length > 0);

  let mainEl: HTMLElement;
  let weekdayEl: HTMLElement;

  // Weeks/months appearing or disappearing above the viewport (logs moved,
  // added or deleted) would shift what's on screen: keep the first visible
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

  // ── Drag & drop (moves a log's due date) ──────────────────────────────────
  // Pointer events rather than HTML5 drag & drop, which the Tauri webview can
  // swallow (Windows). A press only becomes a drag past a few pixels, so a
  // plain click still opens the editor. Weekdays are the drop targets —
  // weekends (styled as read-only) and other months' blank cells are not.
  // Dragging near the top/bottom edge scrolls, to reach other months.
  const DRAG_START_PX = 5;
  const EDGE_PX = 56;

  type Drag = {
    log: Log;
    startX: number; startY: number;
    x: number; y: number;
    offsetX: number; offsetY: number; width: number;
    active: boolean; cancelled: boolean;
  };
  let drag: Drag | null = null;
  $: draggingId = drag?.active && !drag.cancelled ? drag.log.id : null;
  let dropKey: string | null = null;
  let suppressClick = false;
  let scrollRaf = 0;

  function onChipPointerDown(e: PointerEvent, log: Log) {
    if (e.button !== 0) return;
    // Pressed on a log row: the ghost takes the card's width, held where grabbed.
    const row = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const card = ((e.currentTarget as HTMLElement).closest('.log-chip') as HTMLElement).getBoundingClientRect();
    drag = {
      log, startX: e.clientX, startY: e.clientY, x: e.clientX, y: e.clientY,
      offsetX: e.clientX - card.left, offsetY: e.clientY - row.top, width: card.width,
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
    const { log, active, cancelled } = drag;
    const target = dropKey;
    endDrag();
    if (!active) return; // a plain click: the chip's click handler opens the editor
    // The click that follows this pointerup must not open the editor.
    suppressClick = true;
    setTimeout(() => { suppressClick = false; });
    if (!cancelled && target && target !== log.due_date) {
      moveLogDueDate(log, target).catch(err => console.error('moveLogDueDate failed:', err));
    }
  }

  function onWindowKeydown(e: KeyboardEvent) {
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

  function onChipClick(log: Log) {
    if (!suppressClick) openEdit(log);
  }

  onDestroy(() => {
    cancelAnimationFrame(scrollRaf);
    clearTimeout(flashTimer);
  });

  // ── Move old logs ─────────────────────────────────────────────────────────
  // "Old" = the open logs the calendar shows that are due before the current
  // week (any past week). Each moves to the same weekday this week (a Tuesday
  // stays a Tuesday), after a confirmation dialog listing every move.
  $: oldLogs = calendarLogs
    .filter(l => (l.due_date ?? '') < weekStartKey)
    .sort((a, b) => (a.due_date ?? '').localeCompare(b.due_date ?? '') || a.title.localeCompare(b.title));

  let moveSource: Log[] | null = null; // the confirmation is open while set
  let moving = false;
  let moveError = '';
  let moveConfirmBtn: HTMLButtonElement;

  $: moves = (moveSource ?? []).map(log => {
    const from = log.due_date ?? '';
    return { log, from, to: sameWeekdayInWeekOf(from, now) };
  });
  // Moved to a weekday this week that has already passed: still overdue.
  $: stillOverdue = moves.filter(m => m.to < todayKey).length;

  async function openMoveDialog() {
    // Snapshot, so the list can't change under the confirmation.
    moveSource = oldLogs;
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
      await moveLogsDueDates(moves.map(m => ({ log: m.log, dueDate: m.to })));
      moving = false;
      moveSource = null;
    } catch (e) {
      moveError = `Some logs could not be moved: ${e}`;
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

  // Clicking a weekday (outside its logs) creates a log due that day.
  // Weekends are read-only: no creation there.
  function openNewLog(key: string) {
    editorLog = { due_date: key, type_id: logTypes[0]?.id ?? 0, project_id: $liveProjects[0]?.id ?? 0 } as Log;
    showEditor = true;
  }

  function onDayClick(e: MouseEvent, key: string) {
    if (suppressClick || (e.target as HTMLElement).closest('.log-chip')) return;
    openNewLog(key);
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
      <!-- Only offered while past weeks still hold open logs. -->
      {#if oldLogs.length > 0}
        <button
          class="btn-secondary-sm move-old-btn" on:click={openMoveDialog}
          title="Move the open logs due before this week (before {fmtDay(weekStartKey)}) to the same weekday this week"
        >
          Move old logs
          <span class="count-pill">{oldLogs.length}</span>
        </button>
      {/if}
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
                <!-- Mouse: click anywhere outside a log to create one; keyboard: the + button. -->
                <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                <div
                  class="day" class:weekend class:current-week={currentWeek} class:today={key === todayKey}
                  class:flash={flashWeek && currentWeek}
                  class:drop-target={dropKey === key}
                  data-date={key}
                  data-drop={weekend ? null : key}
                  on:click={e => { if (!weekend) onDayClick(e, key); }}
                >
                  <span class="day-num">{day.getDate()}</span>
                  {#if !weekend}
                    <button
                      class="add-btn" title="New log due {fmtDay(key)}" aria-label="New log due {fmtDay(key)}"
                      on:click|stopPropagation={() => openNewLog(key)}
                    >+</button>
                  {/if}
                  {#each groupsByDate.get(key) ?? [] as group (group.projectId)}
                    <!-- One card per project and day: the project on top, then each log
                         as its own row (click to edit, drag to reschedule). The buttons
                         fill the card, so a click never falls through to the day. -->
                    <div
                      class="log-chip"
                      class:overdue={key < todayKey}
                      class:dragging={group.logs.length === 1 && group.logs[0].id === draggingId}
                    >
                      <button
                        class="chip-project" title="Open {group.path} in the Project view"
                        on:click|stopPropagation={() => openProjectView(group.projectId)}
                      >{group.path}</button>
                      {#each group.logs as log (log.id)}
                        <button
                          class="chip-title"
                          class:dragging={group.logs.length > 1 && log.id === draggingId}
                          title="{log.title}&#10;Click to edit · drag to reschedule"
                          on:pointerdown={e => onChipPointerDown(e, log)}
                          on:click|stopPropagation={() => onChipClick(log)}
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
    <div
      class="log-chip drag-ghost" class:overdue={(drag.log.due_date ?? '') < todayKey}
      style="left: {drag.x - drag.offsetX}px; top: {drag.y - drag.offsetY}px; width: {drag.width}px;"
    >
      <span class="ghost-project">{projectPath(Number(drag.log.project_id), $liveProjects)}</span>
      <span class="ghost-title">{drag.log.title}</span>
    </div>
  {/if}

  {#if moveSource}
    <div class="backdrop" on:click={closeMoveDialog} role="presentation"></div>
    <div class="move-dialog" role="dialog" aria-modal="true" aria-labelledby="move-dialog-title">
      <h2 id="move-dialog-title">Move old logs to this week?</h2>
      <p class="dialog-hint">
        Are you sure? {moves.length === 1 ? 'This open log is' : `These ${moves.length} open logs are`} due before
        this week (before {fmtDay(weekStartKey)}) and will move to the same weekday this week.
      </p>
      <ul class="move-list">
        {#each moves as mv (mv.log.id)}
          <li>
            <span class="move-title">{mv.log.title}</span>
            <span class="move-dates" class:overdue={mv.to < todayKey}>{fmtDay(mv.from)} → {fmtDay(mv.to)}</span>
          </li>
        {/each}
      </ul>
      {#if stillOverdue > 0}
        <p class="dialog-note">
          {stillOverdue} log{stillOverdue !== 1 ? 's' : ''} (in red) will land on a day of this week that has already
          passed, so {stillOverdue !== 1 ? 'they stay' : 'it stays'} overdue.
        </p>
      {/if}
      {#if moveError}
        <p class="dialog-error">{moveError}</p>
      {/if}
      <div class="dialog-actions">
        <button class="btn-secondary-sm" on:click={closeMoveDialog} disabled={moving}>Cancel</button>
        <button class="btn-primary" bind:this={moveConfirmBtn} on:click={confirmMove} disabled={moving}>
          {moving ? 'Moving…' : `Move ${moves.length} log${moves.length !== 1 ? 's' : ''}`}
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
    display: flex; flex-direction: column;
    height: 100%; overflow: hidden;
    background: var(--surface-2); color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .page.dark {
    --weekend-bg:   #0e1420;
    --weekend-text: #6b7280;
    --current-week-bg: rgba(99,102,241,0.15);
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
    position: relative; /* anchors the + button */
    background: var(--card-bg);
    border: 1px solid var(--border); border-radius: 6px;
    padding: 6px;
    display: flex; flex-direction: column; gap: 3px;
    min-width: 0;
    transition: background 0.1s, border-color 0.1s;
  }
  .day.blank { background: none; border-color: transparent; }
  .day[data-drop] { cursor: pointer; } /* weekdays: click to create a log */

  .add-btn {
    position: absolute; top: 4px; right: 4px;
    width: 20px; height: 20px;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent); color: #fff;
    border: none; border-radius: 6px;
    font-size: 15px; font-weight: 600; line-height: 1; font-family: inherit;
    cursor: pointer;
    opacity: 0; transition: opacity 0.15s;
  }
  .day:hover .add-btn, .add-btn:focus-visible { opacity: 1; }
  .page.dragging .add-btn { display: none; }
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

  /* One card per project and day: tinted, colored left bar; the project on
     top, then one row per log, separated by thin lines. Titles wrap over as
     many lines as needed (never truncated). */
  .log-chip {
    --chip-tint:  rgba(99,102,241,0.12);
    --chip-hover: rgba(99,102,241,0.14);
    --chip-line:  rgba(99,102,241,0.28);
    display: block; width: 100%; box-sizing: border-box;
    background: var(--chip-tint); color: var(--text);
    border-left: 3px solid var(--accent); border-radius: 4px;
    overflow: hidden; /* clips the row hover backgrounds */
    transition: opacity 0.15s;
  }
  .log-chip + .log-chip { margin-top: 3px; }
  /* All calendar logs are open, so a past deadline means overdue. */
  .log-chip.overdue {
    --chip-tint:  rgba(239,68,68,0.12);
    --chip-hover: rgba(239,68,68,0.14);
    --chip-line:  rgba(239,68,68,0.28);
    border-left-color: #ef4444;
  }

  .chip-project, .chip-title {
    display: block; width: 100%; box-sizing: border-box;
    background: none; border: none;
    text-align: left; font-family: inherit; color: inherit;
    white-space: normal; overflow-wrap: anywhere;
    cursor: pointer;
  }
  .chip-project {
    padding: 5px 8px 1px;
    font-size: 12px; font-weight: 600; line-height: 1.3;
    color: var(--text-muted);
  }
  .chip-project:hover { color: var(--accent); text-decoration: underline; }
  .chip-title {
    padding: 1px 8px 5px;
    font-size: 14px; line-height: 1.35;
    transition: background 0.15s, opacity 0.15s;
  }
  .chip-title + .chip-title { border-top: 1px solid var(--chip-line); padding-top: 5px; }
  .chip-title:hover { background: var(--chip-hover); }
  .log-chip.dragging, .chip-title.dragging { opacity: 0.35; }

  /* The dragged copy keeps its project for context. */
  .ghost-project {
    display: block; margin-bottom: 1px;
    font-size: 12px; font-weight: 600; line-height: 1.3;
    color: var(--text-muted);
  }
  .ghost-title { display: block; font-size: 14px; line-height: 1.35; overflow-wrap: anywhere; }

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
