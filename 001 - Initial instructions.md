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

# Data Model

### DB Migration versioning
Schema is managed via `PRAGMA user_version` in `src-tauri/src/db/schema.rs`. Each migration block runs only once per database file. Current version: **8**.

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
- A picklist value can only be deleted if no log references it (enforced server-side).
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
| start_date | TEXT | Yes | NULL | Optional start date (ISO 8601, no logic enforced) |
| end_date | TEXT | Yes | NULL | Optional end date (ISO 8601, no logic enforced) |

#### Notes
- Application logic prevents circular parent references.
- Deleting a project is blocked if it has sub-projects or logs.
- Closed projects are hidden by default; shown when "Show closed" toggle is on.
- Projects can have category values assigned (see junction tables below).

### Tables: `project_category_1`, `project_category_2`, `project_category_3`, `project_category_4`
Junction tables for many-to-many category assignments on projects.

| Field | Type | Description |
|---|---|---|
| project_id | INTEGER | FK → projects.id (ON DELETE CASCADE) |
| value_id | INTEGER | FK → user_customizable_input.id |

Categories assigned to a project are inherited by all logs in that project and all descendant sub-projects (recursive ancestor traversal at filter time).

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

## User Interface

### Layout Overview

The main screen is divided into four zones:

- **Menu bar (top)**: split into two groups.
  - **Left group**: Book icon logo, "Show closed" toggle, project filter dropdown, "✕ Clear filters" button — all grouped tightly together (not spread across the bar).
  - **Right group** (`nav`): "+ New Project" button, settings gear (⚙️). No dark mode toggle here — it lives only in Settings.
- **Top area (main column)**: Category 3 and Category 4 filter badges, side by side.
- **Left sidebar (230px)**: Category 1 and Category 2 filter badges, arranged horizontally with wrapping. Category 1 is at the top, directly below the menu bar.
- **Main area**: Project cards in a vertical tree list (full width).

All spacing in the menu bar, sidebar, top category bar, main grid, log/project cards, settings modal, and log/project editors is controlled by the **Layout density** setting (see Settings Panel).

### Show Closed Toggle

A toggle switch in the top-left of the menu bar (next to the logo) labelled **"Show closed"**:
- **Off (default)**: closed **projects** are hidden. Closed **logs** are always shown inside visible projects (greyed out at 60% opacity).
- **On**: closed projects are also visible (rendered at 55% opacity). Closed logs remain visible in all cases.

### Project Filter Dropdown

A `<select>` in the menu bar listing all projects (indented to show hierarchy). Selecting a project:
- Shows the selected project and all its descendants.
- Ancestor projects (parent chain up to root) are shown as context-only — their header is visible but their log table is hidden.
- When no category filter is active, **all descendants are shown** including closed ones, regardless of whether they have logs.
- When category filters are active, descendants are only shown if their subtree contains matching logs (closed ones still respect the "Show closed" toggle).
- Selecting "All projects" resets the filter.

### Clear Filters Button

A **"✕ Clear filters"** button sits in the menu bar's left group, next to the project filter dropdown. It is **always visible** (not conditional) — clicking it resets all filters (categories, project, show-closed) at once. Always-visible by design: users tend to click it by default just to reassure themselves nothing is filtered.

### Settings Panel (⚙️)

A **centered modal** (80vw × 80vh, rounded corners) with a left-side section nav and content area on the right — not a slide-in panel. Sections:
1. **Appearance** — dark/light mode toggle; **Layout density** picklist (Compact / Normal / Comfortable) controlling spacing app-wide
2. **Category Labels** — rename the 4 category labels
3. **Log Types** — add, rename, delete log types (delete blocked if logs use it)
4. **Projects** — add, rename, delete projects with parent picker

Each section has a title and a short description above its controls. Settings rows use a consistent `label + description` on the left, control on the right pattern.

### Category Filters

All four categories are always visible, acting as quick filters.

- **Label**: editable in place — clicking allows the user to rename the category.
- **Values**: shown as clickable badges arranged **horizontally with wrapping**; clicking filters the main view. Values are sorted alphabetically.
- **Edit/Delete buttons**: visible only when a badge is **selected** (not on hover) — ✎ edit (inline rename) and × delete (blocked if any log uses the value).
- **Add or search**: an **"Add or search"** button appears **before** the badges. Clicking it opens an inline input that simultaneously filters visible badges (case-insensitive substring match as you type) and allows creating new values. Pressing Enter with an exact match toggles that value; with no match, creates a new value and auto-selects it. Escape or clicking away closes the input. The same control appears in the Log Editor and Project Editor category sections.

**Filter logic:**
- Selecting multiple values within the **same category** → **AND** (the log+project combined must carry ALL selected values)
- Selecting values across **different categories** → **AND** (log must match all active category filters)
- A log's **effective category set** = its own categories **∪** all ancestor project categories (recursive)
- When category filters are active, projects whose entire subtree has no matching logs are **hidden automatically**
- When "Show closed" is off, closed projects are hidden regardless of other filters
- When a project is selected in the dropdown with no category filter active, all descendant projects are shown (including closed ones)

| Category | Position | Arrangement | Color |
|---|---|---|---|
| Category 1 | Left sidebar, top | Values horizontal, wrapping | Indigo `#6366f1` |
| Category 2 | Left sidebar, bottom | Values horizontal, wrapping | Emerald `#10b981` |
| Category 3 | Top bar, left half | Values horizontal, wrapping | Rose `#e11d48` |
| Category 4 | Top bar, right half | Values horizontal, wrapping | Cyan `#06b6d4` |

Badge style:
- **Unselected**: colored text + border, background matches theme
- **Selected**: colored background, text auto-contrasts (black or white)

### Project Tree (Main Area)

Projects are displayed as a vertical tree list (not a grid). Each level of nesting is indented 24px to the right relative to its parent, giving a visual hierarchy without drawing tree lines.

Each **project card** contains:
- **Header row**: collapse chevron | project title (clickable → opens Project Editor) | assigned category badges (inline, wrapping) | "Closed" pill (if closed) | open/total log count badge | **＋ Sub-project** button | **+** button
- **Log table** (if logs exist): bordered table with columns **Title**, **Deadline**, **Description** — sorted by deadline descending (no deadline last). Closed log rows are greyed out. Hovering a row expands the Description cell to show the full text.
- Sub-projects rendered immediately below the log table, at the next indent level.

The **＋ Sub-project** button opens the Project Editor pre-filled with the current project as the parent. The **+** button on each project opens a **type picker** dropdown (purple background, white text). Selecting a log type opens the Log Editor pre-filled with that type and the project. The dropdown is dismissed by clicking outside it.

The log count badge always shows `open / total` (e.g. `2 / 5`). It is always visible next to the `+` button.

Closed projects render at 55% opacity. The "Closed" pill appears in the header.

When a project has no logs to display:
- **No filters active**: shows *"No logs yet — click + to add one."*
- **Filters active and the project has logs but none match**: shows *"No logs matching the filters."*

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
- Delete (blocked if sub-projects exist)

### Rich Text Editor

Built on `contenteditable` with `document.execCommand`. Toolbar:
- **B** / *I* toggle buttons (highlights when active)
- **Bullet list** / **Numbered list** toggle buttons (highlight when cursor is inside a list; click again to remove)
- Font size selector (Small / Normal / Large / Huge)
- 8 color swatches

Additional behaviors:
- **Pasting a bare URL** (`https://...`) automatically converts it to a clickable `<a>` link
- **Clicking a link** (Ctrl/Cmd+click in the editor, or click in the log card description) opens it in the system default browser

### Links

Links in the rich text editor and log card descriptions open in the **system default browser** via `@tauri-apps/plugin-opener` (`openUrl`). Permission: `opener:allow-open-url` in `src-tauri/capabilities/default.json`.
