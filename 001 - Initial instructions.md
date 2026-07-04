# Note Ledger - Business goal
The objective of Note Ledger is to be able to track and remember everything that happen at work, I call it a log:
* Decision taken
* Important dates (Go live, migration, etc.)
* Tasks
* Etc.

Everyone organize differently in their mind, so it's important to give each people a way to organize the way they wish by:
* Letting them customize the type of logs (Decision taken, Important dates, etc.)
* Letting them define 4 customizable categories that can be assigned to each log — **each log can hold multiple values per category (many-to-many)**

Logs can be grouped inside **Projects**. A project can be nested inside another project (no circular references allowed). Projects are a separate concept from logs.

**Projects can also have category values assigned to them.** A log inherits its project's categories (recursively up the ancestor chain), so filtering by a category value will surface all logs belonging to a project that carries that value — even if the individual log doesn't have it explicitly set.

# Development instruction
The application is developed using:
* **Rust** + **Tauri v2** for the backend and desktop shell
* **SQLite** via `rusqlite` (bundled feature) for the database
* **SvelteKit** for the user interface

**App name:** `Note Ledger` (productName in `tauri.conf.json`)
**App identifier:** `com.martinchalot.noteledger`
**Project location:** `note-ledger-tauri-new/`
**DB location:** `~/Library/Application Support/note-ledger/note_ledger.db` (macOS) / `%APPDATA%\note-ledger\note_ledger.db` (Windows)
**Dev command:** `npm run tauri dev` (port 1420)
**Window:** Opens maximized. Title is set in `src-tauri/tauri.conf.json`.
**Icon:** Custom NLedger book icon — dark indigo spine (left), lighter indigo body, pages strip (right), "NLedger" text rotated vertically on body. Source PNG at `src-tauri/icons/app-icon-source.png`.

## Security
- **CSP enabled** in `src-tauri/tauri.conf.json` (`app.security.csp` + `devCsp`): scripts/assets restricted to `'self'`, Tauri IPC allowed via `ipc:`/`http://ipc.localhost`, inline `style=` attributes allowed. Tauri auto-adds hashes for its own injected scripts in bundled assets.
- **HTML sanitization** (`src/lib/sanitize.ts`, DOMPurify): log/project descriptions are stored as rich-text HTML. Every path that turns that HTML into DOM goes through `sanitizeHtml()` — the three `{@html}` render sites (LogCard, ProjectCard, Table view), the RichTextEditor load (`editor.innerHTML`), its `syncValue()` save, and pasted `text/html`. Allowlist: formatting tags, lists, spans/font with `href`/`style`/`color` attributes only.
- **URL opening** (`openLink()` in `src/lib/types.ts`): only `http:`, `https:` and `mailto:` URLs are passed to the opener plugin; anything else (e.g. `file:`) is refused with a console warning. The capability file grants only `opener:allow-open-url` + default-URL scope.
- **SQL**: all queries use parameterized statements; the only interpolated identifiers are compile-time constant junction-table names.

## Testing & CI
- **Rust:** `cd src-tauri && cargo test --lib` — in-memory SQLite tests covering migrations and repository data-integrity rules (`src-tauri/src/db/tests.rs`).
- **Frontend:** `npm test` (vitest) — pure filter/visibility logic extracted to `src/lib/filters.ts` (`src/lib/filters.test.ts`).
- **Type-check:** `npm run check` (svelte-check) must report 0 errors.
- CI (`.github/workflows/build.yml`) runs all three in a `test` job that must pass before the Windows `build` job produces the installer artifact.

# Data Model

### DB Migration versioning
Schema is managed via `PRAGMA user_version` in `src-tauri/src/db/schema.rs`. Each migration block runs only once per database file. Current version: **10** (tracked by `schema::LATEST_VERSION`).

Before any pending migration runs, `db::open` copies the existing database file to `note_ledger.db.backup-v{N}-{timestamp}` (same folder as the DB) and **verifies** the copy (size match + backup opens as SQLite at the expected schema version). If the WAL flush, copy, or verification fails, **the app refuses to start and the migration does not run** — the database is left untouched, and a `STARTUP-ERROR.txt` explaining the reason is written next to the DB (a bundled app has no console). Skipped for a brand-new database or one already at the latest version.

### Table: `user_settings`
Stores application-wide preferences.

| Field | Type | Nullable | Default | Description |
|---|---|---|---|---|
| category1_label | TEXT | No | — | Display label for Category 1 |
| category2_label | TEXT | No | — | Display label for Category 2 |
| category3_label | TEXT | No | — | Display label for Category 3 |
| category4_label | TEXT | No | — | Display label for Category 4 |
| dark_mode | BOOLEAN | No | FALSE | Whether the UI is in dark mode |
| density | TEXT | No | 'normal' | Layout density: `compact`, `normal`, or `comfortable` (picklist only — no free-form values) |

### Table: `user_customizable_input`
Picklist values for each user-customizable input (four categories + log type).

| Field | Type | Nullable | Default | Description |
|---|---|---|---|---|
| id | INTEGER | No | autoincrement | Primary key |
| type | TEXT | No | — | One of: category_1, category_2, category_3, category_4, log_type |
| label | TEXT | No | — | Display label |

#### Notes
- A picklist value (log type or category value) can only be deleted if **nothing references it** — no log uses it as a type or category, and no project uses it as a category (enforced server-side). The error reports how many items still use it.
- Values are always displayed in alphabetical order.

### Table: `projects`
A project groups logs together and can be nested under a parent project.

| Field | Type | Nullable | Default | Description |
|---|---|---|---|---|
| id | INTEGER | No | autoincrement | Primary key |
| title | TEXT | No | — | Project name |
| description | TEXT | Yes | NULL | Optional description |
| parent_id | INTEGER | Yes | NULL | FK → projects.id (self-referencing, for nesting) |
| is_closed | BOOLEAN | No | FALSE | Whether the project is closed |
| is_template | BOOLEAN | No | FALSE | Template projects live on the Templates page only (v10) |
| start_date | TEXT | Yes | NULL | Optional start date (ISO 8601, no logic enforced) |
| end_date | TEXT | Yes | NULL | Optional end date (ISO 8601, no logic enforced) |

#### Notes
- Application logic prevents circular parent references.
- Deleting a project is blocked if it has sub-projects or logs.
- Closed projects are hidden by default; shown when "Show closed" toggle is on.
- Projects can have category values assigned (see junction tables below).
- `is_template` is fixed at creation time (not updatable). A sub-project always inherits its parent's flag at insert (enforced in `project_repo::insert`), so a template tree stays consistently marked. Template projects and their logs are excluded from the Project view and Table view pages.

### Tables: `project_category_1`, `project_category_2`, `project_category_3`, `project_category_4`
Junction tables for many-to-many category assignments on projects.

| Field | Type | Description |
|---|---|---|
| project_id | INTEGER | FK → projects.id (ON DELETE CASCADE) |
| value_id | INTEGER | FK → user_customizable_input.id |

Categories assigned to a project are inherited by all logs in that project and all descendant sub-projects (recursive ancestor traversal at filter time).

### Table: `project_links`
Labelled URLs attached to a project.

| Field | Type | Nullable | Default | Description |
|---|---|---|---|---|
| id | INTEGER | No | autoincrement | Primary key |
| project_id | INTEGER | No | — | FK → projects.id (ON DELETE CASCADE) |
| label | TEXT | No | — | Display label shown on the link card |
| url | TEXT | No | — | Full URL (e.g. `https://…`) |

Links are displayed in the project card's **Links tab** (see project card body tabs). Clicking opens the URL in the system default browser.

### Table: `logs`
A single log entry.

| Field | Type | Nullable | Default | Description |
|---|---|---|---|---|
| id | INTEGER | No | autoincrement | Primary key |
| type_id | INTEGER | No | — | FK → user_customizable_input.id (log_type) |
| title | TEXT(255) | No | — | Title of the log |
| description | TEXT | Yes | NULL | Rich text description (HTML from contenteditable) |
| start_date | DATE | No | current date | Set automatically at creation |
| due_date | DATE | Yes | NULL | Optional due date |
| is_closed | BOOLEAN | No | FALSE | Whether the log is closed |
| closed_date | DATE | Yes | NULL | Auto-set when closed; reset to NULL when re-opened |
| project_id | INTEGER | No | — | FK → projects.id (required — every log must belong to a project) |

#### Notes
- A log must always be assigned to a project. An "Others" project is created automatically (migration v6) to hold any pre-existing orphan logs.

### Tables: `log_category_1`, `log_category_2`, `log_category_3`, `log_category_4`
Junction tables for many-to-many category assignments on logs.

| Field | Type | Description |
|---|---|---|
| log_id | INTEGER | FK → logs.id |
| value_id | INTEGER | FK → user_customizable_input.id |

Each log can have **multiple values** assigned per category slot.

### Deletion rules (history preservation)

To keep the record intact, the structural entities are protected from deletion server-side; only logs are freely deletable. Closing (`is_closed`) is the non-destructive alternative for logs and projects.

| Entity | Can be deleted when… | Blocked when… |
|---|---|---|
| Log | always | — |
| Project | it has no sub-projects **and** no logs | it has sub-projects, or it still contains logs |
| Log type | no log uses it | any log is of that type |
| Category value | no log **and** no project uses it | any log or project has it assigned |

All blocks are enforced in the repository layer and surfaced as inline error text in the relevant editor (Project Editor / Settings page).

## User Interface

### Layout Overview

The app has a global **navigation bar** at the very top (shared by every page via the layout): a "Note Ledger" text wordmark sits at the left edge (bold, theme text color — dark in light mode, light in dark mode; no icon), and a centered segmented tab control (`NavTabs.svelte`) switches between the views — **Project view | Table view | Templates | Settings**. The active tab is highlighted as a solid accent-colored pill with white text; the strip background is a darker surface shade for contrast. New pages are added by appending to the `tabs` array in `NavTabs.svelte`.

Below the navigation bar, the main screen (**Project view**, the `/` route — historically "homepage") is divided into three zones:

- **Menu bar (top)**: split into two groups.
  - **Left group**: the **filter funnel icon** and the active-filter **chips** (see Filter Drawer below).
  - **Right group** (`nav`): **"Fold all / Unfold all"** toggle button, "+ New Project" button. No dark mode toggle here — it lives only in Settings.
- **Main area**: Project cards in a vertical tree list (full width — there is no permanent sidebar anymore).
- **Filter drawer**: slides in from the left when the funnel icon is clicked.

All spacing in the menu bar, main grid, log/project cards, settings page, and log/project editors is controlled by the **Layout density** setting (see Settings Page).

### Filter Drawer

All filters live in a single component, `FilterPanel.svelte`, used by both the Project view and the Table view. It renders three things:

1. **Funnel icon button** (left side of the page header). Clicking toggles the drawer. When any filter is active, the icon is accent-colored and carries a **count badge** with the number of active filters.
2. **Active-filter chips**, shown next to the icon at all times (even with the drawer closed) so active filters are never hidden state. One chip per active filter — selected project, log type, each selected category value (tinted with its category color), and "Closed shown". Each chip has an **×** that clears just that filter.
3. **The drawer** itself: a non-modal panel (440px) that slides in from the left edge — **no backdrop**, so the main view stays visible and updates live as filters are toggled. Closed via the funnel icon, the ✕ button, or **Escape**. Header row: "Filters" title, a red **✕ Clear all** button (visible only while filters are active), and the close button. Body sections:
   - **Project filters** — project lookup (search input) and the "Show closed" toggle, side by side.
   - **Log filters** — the log type single-select.
   - **Category filters** — the four category filter groups in a **2×2 grid** (1/2 on the first row, 3/4 on the second). Category labels remain editable in place here, and the "Add or search" badge management works exactly as before.

The filter *behaviors* below are unchanged from the previous top-bar/sidebar design — only their location moved into the drawer.

### Show Closed Toggle

A toggle switch in the drawer's **Project filters** section labelled **"Show closed"**:
- **Off (default)**: closed **projects** are hidden. Closed **logs** are always shown inside visible projects (greyed out at 60% opacity).
- **On**: closed projects are also visible (rendered at 55% opacity). Closed logs remain visible in all cases.

### Project Lookup

A search-style lookup in the drawer's **Project filters** section (replaces the old `<select>` dropdown). Behavior:
- **Click** the input → clears any active project filter and shows all projects in a floating dropdown, sorted A-Z with sub-projects indented (4 spaces per depth level) immediately below their parent.
- **Type to filter** — token search, case-insensitive, order-independent (e.g. "api back" matches "Backend API"). The dropdown narrows in real time.
- **Closed projects** are shown dimmed with a "Closed" pill so their status is immediately visible.
- **Selecting a project** filters the main view to that project and its descendants. Pressing **Enter** selects the first option in the dropdown; **Escape** closes it without selecting.
- To clear the project filter, click the input again, dismiss the project chip in the header, or use the drawer's **✕ Clear all** button.

Filter logic once a project is selected:
- Shows the selected project and all its descendants.
- Ancestor projects (parent chain up to root) are shown as context-only — their header is visible but their log table is hidden.
- When no category filter is active, **all descendants are shown** including closed ones, regardless of whether they have logs.
- When category filters are active, descendants are only shown if their subtree contains matching logs (closed ones still respect the "Show closed" toggle).

### Log Type Filter

A **basic single-select dropdown** in the drawer's **Log filters** section, defaulting to **"All types"** and listing every configured log type. Selecting a type narrows the view to logs of that type only; it highlights in the accent color when active. Unlike the category filters (multi-select badges), this is a plain picklist — one type at a time. It composes (AND) with the category and project filters, and like them, projects whose subtree has no matching logs are hidden while it is active. Reset via "All types", the chip's ×, or the drawer's **✕ Clear all** button.

### Clearing Filters

Two ways to clear:
- **Per filter**: every active filter has a chip in the page header with an × — one click removes just that filter, without opening the drawer.
- **All at once**: the **✕ Clear all** button in the drawer header resets everything (categories, project, log type, show-closed). It appears only while at least one filter is active — with no filters there is nothing to clear, and the funnel badge/chips already make the "nothing is filtered" state obvious.

### Settings Page

A **full page** at `/settings`, reached via the **Settings** tab in the navigation bar (no longer a modal or gear button). Same design as before the conversion: a page header, a left-side section nav, and a content area on the right. Sections:
1. **Appearance** — dark/light mode toggle; **Layout density** picklist (Compact / Normal / Comfortable) controlling spacing app-wide
2. **Category Labels** — rename the 4 category labels
3. **Log Types** — add, rename, delete log types (delete blocked if logs use it)
4. **Projects** — add, rename, delete projects with parent picker

Each section has a title and a short description above its controls. Settings rows use a consistent `label + description` on the left, control on the right pattern.

### Category Filters

All four categories sit in the drawer's **Category filters** section (2×2 grid), acting as quick filters.

- **Label**: editable in place — clicking allows the user to rename the category.
- **Values**: shown as clickable badges arranged **horizontally with wrapping**; clicking filters the main view. Values are sorted alphabetically.
- **Edit/Delete buttons**: visible only when a badge is **selected** (not on hover) — ✎ edit (inline rename) and × delete (blocked if any log or project uses the value).
- **Add or search**: an **"Add or search"** button appears **before** the badges. Clicking it opens an inline search input (border color matches the category color) that simultaneously filters visible badges (case-insensitive substring match as you type) and allows creating new values. Pressing Enter with an exact match toggles that value; with no match, creates a new value and auto-selects it. Clicking a badge while the search is open selects/deselects it and clears the search text. Escape or clicking away closes the input. The same control appears in the Log Editor and Project Editor category sections.

**Filter logic:**
- Selecting multiple values within the **same category** → **AND** (the log+project combined must carry ALL selected values)
- Selecting values across **different categories** → **AND** (log must match all active category filters)
- A log's **effective category set** = its own categories **∪** all ancestor project categories (recursive)
- The **log type filter** (menu-bar picklist) further restricts to a single log type, AND-ed with the category filters
- When any log-narrowing filter (category or log type) is active, projects whose entire subtree has no matching logs are **hidden automatically**
- When "Show closed" is off, closed projects are hidden regardless of other filters
- When a project is selected in the dropdown with no category filter active, all descendant projects are shown (including closed ones)

**Filters after creation:** creating a new project or log auto-adjusts the filters so the new item is immediately visible (instead of vanishing behind active filters): all category filters and the log type filter are cleared, and the project filter is set to the new project (or, for a log, the project it belongs to). This happens only on creation — editing an existing item leaves the filters untouched.

| Category | Position (drawer grid) | Arrangement | Color |
|---|---|---|---|
| Category 1 | Row 1, left | Values horizontal, wrapping | Indigo `#6366f1` |
| Category 2 | Row 1, right | Values horizontal, wrapping | Emerald `#10b981` |
| Category 3 | Row 2, left | Values horizontal, wrapping | Rose `#e11d48` |
| Category 4 | Row 2, right | Values horizontal, wrapping | Cyan `#06b6d4` |

Badge style:
- **Unselected**: colored text + border, background matches theme
- **Selected**: colored background, text auto-contrasts (black or white)

### Project Tree (Main Area)

Projects are displayed as a vertical tree list (not a grid). Each level of nesting is indented 24px to the right relative to its parent, giving a visual hierarchy without drawing tree lines.

The **"Fold all / Unfold all"** button in the menu bar collapses every project card (including nested sub-projects) to its header in one click, then toggles to expand them all again. Individual cards can still be folded/unfolded via their chevron in between — the button re-broadcasts the global state on each press. While "Fold all" is active, expanding a project via its chevron reveals only that project's logs and its immediate sub-project headers — the sub-projects themselves stay folded (they inherit the global fold state when revealed) and must be expanded individually.

Each **project card** contains:
- **Header row**: collapse chevron | project title (clickable → opens Project Editor) | assigned category badges (inline, wrapping) | "Closed" pill (if closed) | open/total log count badge | **＋ Sub-project** button | **＋ Link** button | **+** button
- **Body** (shown when the project has logs or links): a full-width **Logs | Links** tab bar in a folder-tab style: the two tabs sit flush against each other with no gap, rounded only on the bar's outer top corners (square at the seam between tabs and along the bottom) so the active tab (solid accent, white text) visually encapsulates the panel below; each tab shows a count pill. **Logs** is the default selection. The tab choice is per-card and resets to Logs when the card is re-rendered (e.g. after folding). Sub-projects are NOT part of the tabs — they render below the card regardless of the selected tab.
  - **Logs tab**: bordered table with columns **Title**, **Deadline**, **Description** — sorted: open logs with due date ASC, then open logs without due date, then closed logs with due date ASC. Closed log rows are greyed out. Descriptions are always fully visible (no hover needed). The **Title** cell shows the log title, then a muted meta line below it reading `{log type} · Open since {N} days` (open logs only — "Open since today" / "1 day" / "N days", computed from the log's creation/start date; closed logs show just the log type), then any category badges. **Clicking anywhere on a log row opens the Log Editor** — with one exception: clicking a link inside the description opens that link in the browser instead.
  - **Links tab**: project links displayed as Confluence-style cards (chain icon + label), flowing horizontally with wrapping across the full width. Clicking a card opens the URL in the system default browser. A ✎ button opens the Link Editor.
  - Each tab has an empty state ("No logs." / "No links — use ＋ Link to add one.").
- Sub-projects rendered nested below the body: each sub-project is connected to its parent by **tree connector lines** (a vertical trunk plus an elbow into each child's header, colored like the project headers), making the hierarchy path visually explicit.

The **＋ Sub-project** button opens the Project Editor pre-filled with the current project as the parent. The **＋ Link** button opens the **Link Editor** (slide-in panel, 400px wide) to add a labelled URL to the project. The **+** button opens a **type picker** dropdown (purple background, white text). Selecting a log type opens the Log Editor pre-filled with that type and the project.

The log count badge always shows `open / total` (e.g. `2 / 5`). It is always visible next to the `+` button.

Closed projects render at 55% opacity. The "Closed" pill appears in the header.

Project header bars use a dedicated `--project-header` color (a clearly darker blue-grey in light mode, a lighter grey in dark mode) so they stand out from the page background; the tree connector lines use the same color.

When a project has no logs to display:
- **No filters active**: shows *"No logs yet — click + to add one."*
- **Filters active and the project has logs but none match**: shows *"No logs matching the filters."*

### Link Editor (slide-in panel, 400px wide)

Fields:
- Label (required) — display name for the link
- URL (required) — the full URL (e.g. `https://…`)

The project name is shown as a subtitle below the panel title. Supports create, edit, and delete. Links open in the system default browser via `opener:allow-default-urls`.

### Table View Page

A flat table display of logs across all projects (formerly called "Deadlines" — renamed because deadlines are also visible on the homepage; what distinguishes this page is the flat table layout). Accessible via the **"Table view"** tab in the navigation bar, at the `/table` route. Shows all **open logs** (closed logs excluded), with columns:

| Column | Notes |
|---|---|
| Project | Full project path (ancestors joined with `›`); category badges assigned to the project shown below the path |
| Log | Log title; a muted `{log type} · Open since {N} days` meta line below it (all table-view logs are open); category badges assigned to the log shown below that |
| Deadline | Color-coded pill (see color rules below); blank if no due date |
| Description | Full rich text, not truncated |

Sorted by due date ASC (no due date → end of list); closed logs sort after open ones and render greyed out with no deadline color. Descriptions are always fully expanded — no hover needed.

**Filters**: the Table view embeds the same `FilterPanel` component as the Project view — funnel icon + chips in the page header, and the same filter drawer — with identical matching semantics (AND logic, categories inherited from ancestor projects, project filter covering the selected subtree). One log-centric adaptation: here "Show closed" reveals closed **logs** (hidden by default). **Filter state is shared** between the Project view and the Table view via Svelte stores in `store.ts` (`showClosed`, `selCat1–4`, `selProject`, `selLogType`) — switching pages keeps the same filters applied. Filters reset on app restart (in-memory only).

**Row click behavior** (same Log/Project editors as the main view, rendered on the page):
- Clicking the **Project** column opens the **Project Editor** for the log's project.
- Clicking **anywhere else on the row** opens the **Log Editor** for that log.
- Exception: clicking a link inside the description opens the link in the browser instead of the editor.
- Edits are reflected in the table immediately after save; closing a log from the editor removes it from the list (only open logs are shown).

**Deadline color rules** (applied in both the project card log table and the Table view page; closed logs are never color-coded):

| Color | Condition |
|---|---|
| 🔴 Red | Due date is this calendar week (up to and including this Sunday) or earlier |
| 🟡 Yellow | Due date falls in next calendar week (Mon–Sun) |
| 🟢 Green | Due date is beyond next week |
| — | No due date, or log is closed |

### Templates Page

At `/templates`, via the **Templates** tab. Behaves like the homepage — same project cards (Logs/Links tabs, fold all, ＋ Sub-project / ＋ Link / + log buttons, editors), "+ New Template" button — **except there are no filters** (no filter drawer/funnel icon, no project lookup, no log type filter, no show-closed toggle; closed template content is always visible). Only **template projects** (`is_template = 1`) are shown here, and they appear nowhere else in the app.

**Cloning**: top-level template cards have an accent **⧉ Clone** button. It opens a small dialog asking for the new project's title (pre-filled with the template's title). Confirming deep-copies the entire tree — logs, links, sub-projects, and their logs/links recursively (`clone_project` command → `project_repo::clone_tree`). The copy:
- is a **regular project** (never a template) that appears on the homepage;
- is **fully independent** — later edits to the template never affect projects cloned from it, and vice versa;
- after cloning, the app navigates to the homepage with all filters cleared and the project filter set to the new project (handed off via the `pendingProjectFocus` store).

**Date re-anchoring on clone** (template dates are typically in the past):
- **Log due dates** shift as a group: the earliest due date anywhere in the template tree lands on **today**, all others keep their relative distance (e.g. today 04/07: logs due 01/06, 04/06, 07/06 become 04/07, 07/07, 10/07). Logs without a due date stay empty.
- **Log start dates** ("Open since") reset to today.
- **Project start dates** become today; **end dates** shift by the same per-project amount so planned duration is preserved. An end date without a start date falls back to the global log shift.
- Closed state / closed dates are copied unchanged; the template's own dates are untouched.

### Log Editor (slide-in panel, 780px wide)

Fields:
- Title (required)
- Log Type
- Description (rich text — see Rich Text Editor section)
- Due Date
- Status (open/closed toggle)
- Project (dropdown — required; every log must belong to a project)
- Categories — 2×2 grid of badge toggles, one group per category; multiple values selectable per category; values sorted alphabetically; **"+ Add" button in each group** to create new category values on the fly (auto-selected after creation, race-condition safe via `pendingAdd` promise tracking)

### Project Editor (slide-in panel, 480px wide)

Fields:
- Title (required), Description
- Start Date, End Date (optional, no logic enforced — same row)
- Parent Project + Closed checkbox (same row; parent takes 3× width)
- **Default Categories** — 2×2 grid of badge toggles; categories assigned here are inherited by all logs in this project and descendant sub-projects; includes "**+ Add**" inline creation per category
- Delete (blocked if sub-projects or logs exist — move or delete the logs first)

### Rich Text Editor

Built on `contenteditable` with `document.execCommand`. Toolbar:
- **B** / *I* toggle buttons (highlights when active)
- **Bullet list** / **Numbered list** toggle buttons (highlight when cursor is inside a list; click again to remove)
- Font size selector (Small / Normal / Large / Huge)
- 8 color swatches

Additional behaviors:
- **Pasting a bare URL** (`https://...`) automatically converts it to a clickable `<a>` link
- **Clicking a link** (Ctrl/Cmd+click in the editor, or click in the log card description) opens it in the system default browser via `opener:allow-default-urls`
- **Tab** inside a list item indents it (creates a nested sub-list); **Shift+Tab** outdents it. Outside a list, Tab inserts 4 spaces.

### Links

Links in the rich text editor and log card descriptions open in the **system default browser** via `@tauri-apps/plugin-opener` (`openUrl`). Permission: `opener:allow-open-url` in `src-tauri/capabilities/default.json`.
