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

# Development instruction
The application is developed using:
* **Rust** + **Tauri v2** for the backend and desktop shell
* **SQLite** via `rusqlite` (bundled feature) for the database
* **SvelteKit** for the user interface

**Project location:** `note-ledger-tauri-new/`
**DB location:** `~/Library/Application Support/note-ledger/note_ledger.db`
**Dev command:** `npm run tauri dev` (port 1420)
**Window:** Opens maximized. Title is set in `src-tauri/tauri.conf.json` (not in `app.html`)

# Data Model

### Table: `user_settings`
Stores application-wide preferences.

| Field | Type | Nullable | Default | Description |
|---|---|---|---|---|
| category1_label | TEXT | No | — | Display label for Category 1 |
| category2_label | TEXT | No | — | Display label for Category 2 |
| category3_label | TEXT | No | — | Display label for Category 3 |
| category4_label | TEXT | No | — | Display label for Category 4 |
| dark_mode | BOOLEAN | No | FALSE | Whether the UI is in dark mode |

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
Junction tables for many-to-many category assignments.

| Field | Type | Description |
|---|---|---|
| log_id | INTEGER | FK → logs.id |
| value_id | INTEGER | FK → user_customizable_input.id |

Each log can have **multiple values** assigned per category slot.

## User Interface

### Layout Overview

The main screen is divided into four zones:

- **Menu bar (top)**: Book icon logo, "Show closed" toggle, "+ New Project" button, per-log-type "New" buttons (e.g. "+ Task", "+ Decision"), dark mode toggle, settings gear (⚙️).
- **Top area (main column)**: Category 3 and Category 4 filter badges, side by side.
- **Left sidebar**: Category 1 and Category 2 filter badges, stacked vertically.
- **Main area**: Log and project cards in a 3-column grid.

### Closed Logs Toggle

A toggle switch in the top-left of the menu bar (next to the logo) labelled **"Show closed"**:
- **Off (default)**: closed logs are hidden everywhere — in the main view and inside project cards.
- **On**: closed logs are visible. Project card count badge shows `open / total` (e.g. `3 / 5`). When all logs are open, just shows the count.

### New Log Buttons

When at least one log type is configured, the generic "+ New Log" button is replaced by individual buttons for each log type (e.g. "+ Task", "+ Decision", "+ Event").

### Settings Panel (⚙️)

Accessible from the menu bar. Contains four sections:
1. **Appearance** — dark/light mode toggle
2. **Category Labels** — rename the 4 category labels
3. **Log Types** — add, rename, delete log types (delete blocked if logs use it)
4. **Projects** — add, rename, delete projects with parent picker

### Category Filters

All four categories are always visible, acting as quick filters.

- **Label**: editable in place — clicking allows the user to rename the category.
- **Values**: shown as clickable badges; clicking filters the main view. Values are sorted alphabetically.
- On hover, each badge reveals an **✎ edit** button (inline rename) and **× delete** button (blocked if any log uses the value).
- Quick-add input after the last badge to create new values inline.

**Filter logic:**
- Selecting multiple values within the **same category** → **AND** (log must have all selected values)
- Selecting values across **different categories** → **AND** (log must match all active category filters)
- When filters are active, project and sub-project cards that have no matching logs are **hidden automatically** (recursive)

| Category | Position | Arrangement | Color |
|---|---|---|---|
| Category 1 | Left sidebar, top | Values stacked vertically | Indigo `#6366f1` |
| Category 2 | Left sidebar, bottom | Values stacked vertically | Emerald `#10b981` |
| Category 3 | Top bar, left half | Values side by side (wrapping) | Rose `#e11d48` |
| Category 4 | Top bar, right half | Values side by side (wrapping) | Cyan `#06b6d4` |

Badge style:
- **Unselected**: colored text + border, background matches theme
- **Selected**: colored background, text auto-contrasts (black or white)

### Log Card (View Mode)

Each log card shows:
- **Title** (dimmed if closed)
- **Log type** label
- **Description** (rendered as rich text, clamped to 3 lines)
- **Deadline date** with urgency color: 🔴 past, 🟡 within 7 days, 🟢 beyond 7 days
- **Category badges** at the bottom (all assigned values across all 4 categories)
- **CLOSED** badge if the log is closed

### Project Cards

Projects appear as collapsible cards spanning the full width of the main grid. Inside each project card:
- Own logs are shown first in a 3-column grid
- Sub-projects are shown below own logs, each spanning full width (recursive)
- A **+** button on the project header opens a new log pre-assigned to that project
- A count badge shows open logs (or `open / total` when closed logs exist and are shown)
- Clicking the project title opens the Project Editor
- When filters are active, projects with no matching logs are hidden

### Log Editor (slide-in panel, 780px wide)

Fields:
- Title (required)
- Log Type
- Description (rich text: bold, italic, font size, 8 color swatches)
- Due Date
- Status (open/closed toggle)
- Project (dropdown — assigns the log to a project)
- Categories — 2×2 grid of badge toggles, one group per category; multiple values selectable per category; values sorted alphabetically

### Project Editor (slide-in panel, 420px wide)

Fields:
- Title, Description
- Parent Project (dropdown with cycle detection — cannot create circular nesting)
- Delete (blocked if sub-projects exist; unlinks logs on delete)

### Rich Text Editor

Built on `contenteditable` with `document.execCommand`. Toolbar:
- **B** / *I* toggle buttons
- Font size selector (Small / Normal / Large / Huge)
- 8 color swatches
