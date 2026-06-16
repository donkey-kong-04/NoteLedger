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
Schema is managed via `PRAGMA user_version` in `src-tauri/src/db/schema.rs`. Each migration block runs only once per database file. Current version: **5**.

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

#### Notes
- Application logic prevents circular parent references.
- Deleting a project is blocked if it has sub-projects.
- Deleting a project unlinks its logs (sets their `project_id` to NULL).
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
| project_id | INTEGER | Yes | NULL | FK → projects.id |

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
  - **Right group** (`nav`): "+ New Project" button, per-log-type "New" buttons (e.g. "+ Task", "+ Decision"), settings gear (⚙️). No dark mode toggle here — it lives only in Settings.
- **Top area (main column)**: Category 3 and Category 4 filter badges, side by side.
- **Left sidebar (230px)**: Category 1 and Category 2 filter badges, arranged horizontally with wrapping. Category 1 is at the top, directly below the menu bar.
- **Main area**: Log and project cards in a 3-column grid.

All spacing in the menu bar, sidebar, top category bar, main grid, log/project cards, settings modal, and log/project editors is controlled by the **Layout density** setting (see Settings Panel).

### Closed Logs Toggle

A toggle switch in the top-left of the menu bar (next to the logo) labelled **"Show closed"**:
- **Off (default)**: closed logs are hidden everywhere — in the main view and inside project cards.
- **On**: closed logs are visible. Project card count badge shows `open / total` (e.g. `3 / 5`). When all logs are open, just shows the count.

### Project Filter Dropdown

A `<select>` in the menu bar listing all projects (indented to show hierarchy). Selecting a project:
- Shows only logs belonging to that project or any of its sub-projects (recursive).
- Shows all sub-projects under the selected project's root, even if empty.
- Selecting "All projects" resets the filter.

### Clear Filters Button

A **"✕ Clear filters"** button sits in the menu bar's left group, next to the project filter dropdown. It is **always visible** (not conditional) — clicking it resets all filters (categories, project, show-closed) at once. Always-visible by design: users tend to click it by default just to reassure themselves nothing is filtered.

### New Log Buttons

When at least one log type is configured, the generic "+ New Log" button is replaced by individual buttons for each log type (e.g. "+ Task", "+ Decision", "+ Event").

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
- **Quick-add input**: "+ Add" button after the last badge; pressing Enter creates and selects the new value inline.

**Filter logic:**
- Selecting multiple values within the **same category** → **AND** (log+project combined must cover all selected values)
- Selecting values across **different categories** → **AND** (log must match all active category filters)
- A log's **effective category set** = its own categories **∪** all ancestor project categories (recursive)
- When filters are active, project and sub-project cards that have no matching logs are **hidden automatically** (recursive check through the full subtree)
- Sub-projects are only shown without logs when a specific project is selected in the dropdown **and** no category/close filters are active

| Category | Position | Arrangement | Color |
|---|---|---|---|
| Category 1 | Left sidebar, top | Values horizontal, wrapping | Indigo `#6366f1` |
| Category 2 | Left sidebar, bottom | Values horizontal, wrapping | Emerald `#10b981` |
| Category 3 | Top bar, left half | Values horizontal, wrapping | Rose `#e11d48` |
| Category 4 | Top bar, right half | Values horizontal, wrapping | Cyan `#06b6d4` |

Badge style:
- **Unselected**: colored text + border, background matches theme
- **Selected**: colored background, text auto-contrasts (black or white)

### Log Card (View Mode)

Each log card shows, top to bottom:
- **Title** (dimmed if closed)
- **Description** (rendered as rich text, clamped to 3 lines by default)
- **Deadline date** and **Log type** label on the same line, packed together on the left (date first, then type — not spread across the row). Deadline color: 🔴 past, 🟡 within 7 days, 🟢 beyond 7 days
- **Category badges** at the bottom (all assigned values across all 4 categories)
- **CLOSED** badge if the log is closed
- **On hover**: card expands in place to show the full description (smooth transition, card grows downward only via `align-self: start`)
- **Links in description**: clickable — opens in the system default browser via Tauri opener plugin

### Project Cards

Projects appear as collapsible cards spanning the full width of the main grid. Inside each project card:
- **Header**: collapse toggle, project title (clickable → opens Project Editor), **assigned category badges** (shown inline next to the title, wrapping to next line if needed), log count badge, **+** button to add a log to this project
- Own logs are shown first in a 3-column grid
- Sub-projects are shown below own logs, each spanning full width (recursive)
- A count badge shows open logs (or `open / total` when closed logs exist and are shown)
- When filters are active, projects and sub-projects with no matching logs are hidden

### Log Editor (slide-in panel, 780px wide)

Fields:
- Title (required)
- Log Type
- Description (rich text — see Rich Text Editor section)
- Due Date
- Status (open/closed toggle)
- Project (dropdown — assigns the log to a project)
- Categories — 2×2 grid of badge toggles, one group per category; multiple values selectable per category; values sorted alphabetically; **"+ Add" button in each group** to create new category values on the fly (auto-selected after creation, race-condition safe via `pendingAdd` promise tracking)

### Project Editor (slide-in panel, 480px wide)

Fields:
- Title, Description
- Parent Project (dropdown with cycle detection — cannot create circular nesting)
- **Default Categories** — 2×2 grid of badge toggles (same as Log Editor); categories assigned here are inherited by all logs in this project and descendant sub-projects; includes "**+ Add**" inline creation per category
- Delete (blocked if sub-projects exist; unlinks logs on delete)

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
