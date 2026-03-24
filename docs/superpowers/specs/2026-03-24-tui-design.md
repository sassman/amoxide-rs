# `am tui` — Interactive Alias Manager TUI

## Overview

A minimalistic, borderless TUI for managing aliases and profiles interactively. Built with ratatui + crossterm, following the existing Elm-architecture pattern. Entered via `am tui`.

The TUI provides a unified tree view of all profiles and project aliases, allowing users to navigate, select, move, create, and delete aliases and profiles — like a midnight commander for alias management.

## Visual Design

### Aesthetic

- No borders, sparse layout, generous whitespace
- Multi-row alias items (name on first line, command dimmed on second line)
- Inspired by gh-dash: spacious, clean, well-designed
- Does not fill all terminal space — content is left-aligned and breathes

### Single Column (Normal Mode)

One unified tree showing project aliases and all profiles with their aliases:

```
  am tui                                                q quit  ␣ select  m move  n new  x delete  s activate

  📁 project (.aliases)
  │
  │  t
  │  ./x.py test
  │
  │  b
  │  ./x.py build
  │
  ● rust (active)
  │
  │  gs
  │  git status
  │
  │  ct
  │  cargo test
  │
  ├─○ node
  │
  │     nr
  │     npm run
  │
  ╰─○ git

       ga
       git add
```

- `📁` icon for project aliases node
- `●` for active profile, `○` for inactive profiles
- Tree connectors (`│`, `├─`, `╰─`) link everything into one continuous tree
- Profile inheritance shown via nesting (child profiles indented under parent)
- All profiles expanded by default

### Two Column (Moving Mode)

Appears when aliases are selected and user presses `m`:

```
  am tui                                                q quit  ␣ select  m move  n new  x delete  s activate

  📁 project (.aliases)
  │
  │  t
  │  ./x.py test
  │
  │  b
  │  ./x.py build
  │
  ● rust (active)                                       📁 project (.aliases)
  │                                                     │
  │  ▸ gs                          ──────────────────►  ● rust (active)
  │    git status                                       │
  │                                                     ├─○ node
  │  ■ ct                                               │
  │    cargo test                                       ╰─○ git
  │
  ├─○ node
  │
  │     nr
  │     npm run
  │
  ╰─○ git

       ga
       git add
```

- Right column shows only profile/project **headers** (no aliases), same order as left
- `▸` = cursor, `■` = selected alias
- Arrow between columns indicates move direction
- Right column is an exact structural mirror of the left tree (same order, same nodes)

### Symbols

| Symbol | Meaning |
|--------|---------|
| `▸` | Cursor position |
| `■` | Selected alias |
| `●` | Active profile |
| `○` | Inactive profile |
| `📁` | Project aliases node |

## Interaction Model

### Keybindings

**Navigation (both columns):**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down |
| `k` / `↑` | Move cursor up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |

**Selection & Moving:**

| Key | Action |
|-----|--------|
| `space` | Toggle select alias under cursor (alias items only) |
| `m` | Enter Moving mode — right column appears, focus moves right (no-op if nothing selected) |
| `Enter` | Execute move to destination under cursor (right column only) |
| `Esc` | Cancel move / clear selection, return to Normal mode |
| `Tab` | Switch focus between columns (Moving mode only) |

**Management:**

| Key | Action |
|-----|--------|
| `n` | Create new profile (inline text input at bottom) |
| `x` | Delete item under cursor — alias (any) or profile (header only) |
| `s` | Set profile under cursor as active (profile headers only) |

**General:**

| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit |

### Cursor Behavior

- Cursor skips non-interactive gaps (empty lines, tree connectors) — jumps between alias items and profile/project headers only
- After a move: selection clears, right column disappears, tree rebuilds, cursor stays near its previous position

### Mode Transitions

```
Normal ──(space to select, then m)──► Moving ──(Enter)──► Normal
  ▲                                      │
  └──────────(Esc)───────────────────────┘
```

## Operations

### Move Alias

1. Select one or more aliases from anywhere in the tree (multi-select across profiles and project)
2. Press `m` — right column appears with destination tree
3. Navigate to destination profile/project header in right column
4. Press `Enter` — aliases are removed from source and added to destination
5. **Name collision:** If destination already has an alias with the same name, show a confirmation prompt before overwriting
6. **Same source/destination:** No-op, silent — no error message
7. Changes persist to disk immediately
8. Tree rebuilds, selection clears, right column disappears

### Delete Alias

- `x` on an alias item removes it from its profile/project
- Persists immediately

### Delete Profile

- `x` on a profile header deletes the profile and all its aliases
- Dependents (child profiles) are re-parented to the deleted profile's parent
- Deleting the currently active profile clears the active marker (no active profile — only global/project aliases remain)
- No special-cased "default" profile — any profile can be deleted

### Create Profile

- `n` triggers an inline text input at the bottom of the screen
- Enter a name, press `Enter` to create
- New profile appears in the tree immediately
- Created as a root-level profile (no inheritance)

### Set Active Profile

- `s` on a profile header sets it as active
- `●`/`○` markers update immediately
- Persists to disk

## Architecture

### Module Structure

```
crates/am/src/
├── tui/
│   ├── mod.rs          // pub fn run() -> Result<()>
│   ├── model.rs        // TuiModel, TreeNode, NodeKind, Mode, Column
│   ├── update.rs       // TuiMessage handling, state transitions
│   ├── view.rs         // ratatui rendering (tree, columns, help bar)
│   ├── tree.rs         // build/rebuild flattened tree from AppModel
│   └── input.rs        // crossterm keypress → TuiMessage mapping
```

### Data Model

```rust
struct TuiModel {
    app_model: AppModel,          // existing profiles, config, project aliases
    tree: Vec<TreeNode>,          // flattened tree for rendering/navigation
    cursor: usize,                // index into flattened tree
    selected: BTreeSet<usize>,    // multi-select set (alias indices)
    mode: Mode,                   // Normal | Moving | TextInput
    dest_tree: Vec<TreeNode>,     // right column tree (headers only)
    dest_cursor: usize,           // cursor in destination column
    active_column: Column,        // Left | Right
}

struct TreeNode {
    kind: NodeKind,               // ProfileHeader | AliasItem | ProjectHeader
    depth: u16,                   // indentation level
    profile_name: Option<String>, // owning profile
    alias_name: Option<String>,   // for AliasItem
    alias_command: Option<String>,// for AliasItem
    is_active: bool,              // active profile marker
    selectable: bool,             // only alias items
}

enum NodeKind {
    ProfileHeader,
    AliasItem,
    ProjectHeader,
}

enum Mode {
    Normal,
    Moving,
    TextInput(String),            // for new profile name
}

enum Column {
    Left,
    Right,
}
```

### Data Flow (per frame)

1. **input.rs** — crossterm event → `TuiMessage`
2. **update.rs** — `TuiMessage` + `&mut TuiModel` → state mutation, optionally calls `AppModel` methods (move/delete/create) and persists to disk
3. **view.rs** — `&TuiModel` → ratatui `Frame` rendering

### Integration with Existing Code

- `TuiModel` wraps `AppModel` — reuses all existing profile/alias/project CRUD operations
- Move = `remove_alias()` from source + `add_alias()` to destination + `save()`
- `Commands` enum gets a new `Tui` variant
- `messages.rs` gets `Message::LaunchTui` that calls `tui::run()`

### New Dependencies

- `ratatui` — terminal UI rendering
- `crossterm` — terminal backend and input events

## Constraints

### Terminal Size

If the terminal is too narrow (below ~60 columns) or too short, exit immediately with an error message asking for more terminal space. No cramped fallback layout.

### Project Aliases

- If no `.aliases` file exists in the cwd ancestry, the `📁 project` node does not appear in the tree
- Moving an alias to the project node when no `.aliases` file exists creates the file automatically
- Project alias discovery follows existing behavior: walks up from cwd, stops before `$HOME`
