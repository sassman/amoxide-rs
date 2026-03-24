# `am tui` — Interactive Alias Manager TUI

## Overview

A minimalistic, borderless TUI for managing aliases and profiles interactively. Built with ratatui + crossterm, following the existing Elm-architecture pattern. Entered via `am tui`.

The TUI provides a unified tree view of all profiles and project aliases, allowing users to navigate, select, move, create, and delete aliases and profiles — like a midnight commander for alias management.

The codebase has three alias scopes: **global** (in `config.toml`, always loaded), **profile** (in `profiles.toml`, loaded per active profile), and **project** (in `.aliases` files, loaded per directory). The TUI covers all three.

## Visual Design

### Aesthetic

- No borders, sparse layout, generous whitespace
- Multi-row alias items (name on first line, command dimmed on second line)
- Inspired by gh-dash: spacious, clean, well-designed
- Does not fill all terminal space — content is left-aligned and breathes

### Single Column (Normal Mode)

One unified tree showing global aliases, project aliases, and all profiles with their aliases:

```
  am tui                                                q quit  ␣ select  m move  n new  x delete  s activate

  🌐 global
  │
  │  ll
  │  ls -lha
  │
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

- `🌐` icon for global aliases node
- `📁` icon for project aliases node
- `●` for active profile, `○` for inactive profiles
- Tree connectors (`│`, `├─`, `╰─`) link everything into one continuous tree
- Profile inheritance shown via nesting (child profiles indented under parent)
- All profiles expanded by default
- Order: global → project → profiles (tree)

### Two Column (Moving Mode)

Appears when aliases are selected and user presses `m`:

```
  am tui                                                q quit  ␣ select  m move  n new  x delete  s activate

  🌐 global
  │
  │  ll
  │  ls -lha
  │
  📁 project (.aliases)
  │
  │  t
  │  ./x.py test
  │
  │  b
  │  ./x.py build
  │
  ● rust (active)                                       🌐 global
  │                                                     │
  │  ▸ gs                          ──────────────────►  📁 project (.aliases)
  │    git status                                       │
  │                                                     ● rust (active)
  │  ■ ct                                               │
  │    cargo test                                       ├─○ node
  │                                                     │
  ├─○ node                                              ╰─○ git
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

### Help Bar

The help bar at the top is **mode-aware**:

- **Normal mode:** `q quit  ␣ select  m move  n new  x delete  s activate`
- **Moving mode:** `Esc cancel  ↑↓ navigate  Enter move here  Tab switch column`
- **TextInput mode:** `Esc cancel  Enter confirm`

### Symbols

| Symbol | Meaning |
|--------|---------|
| `▸` | Cursor position |
| `■` | Selected alias |
| `●` | Active profile |
| `○` | Inactive profile |
| `🌐` | Global aliases node |
| `📁` | Project aliases node |

## Interaction Model

### Keybindings

**Navigation (both columns):**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down (next navigable node) |
| `k` / `↑` | Move cursor up (previous navigable node) |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |

**Selection & Moving (Normal and Moving mode):**

| Key | Action |
|-----|--------|
| `space` | Toggle select alias under cursor (alias items only, no-op on headers) |
| `m` | Enter Moving mode — right column appears, focus moves right (no-op if nothing selected) |
| `Enter` | Execute move to destination under cursor (Moving mode, right column only) |
| `Esc` | Cancel move / clear selection, return to Normal mode |
| `Tab` | Switch focus between columns (Moving mode only, no-op in Normal mode) |

**Management (Normal mode only):**

| Key | Action |
|-----|--------|
| `n` | Create new profile (inline text input at bottom). Disabled in Moving mode. |
| `x` | Delete item under cursor — alias (any) or profile (header only). Shows confirmation for profiles. |
| `s` | Set profile under cursor as active (profile headers only) |

**General:**

| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit |

### Cursor Behavior

Two concepts govern cursor interaction with tree nodes:

- **Navigable:** cursor can land on this node. Applies to alias items and profile/project headers. Whitespace gaps and tree connector lines are skipped.
- **Selectable:** `space` can mark this node for a move. Applies to alias items only — headers are navigable but not selectable.

The cursor always jumps to the next/previous navigable node, skipping decorative rows.

After a move or delete: tree rebuilds, cursor repositions to the nearest navigable node to where it was.

### Scrolling

If the tree is taller than the terminal viewport, the view scrolls to keep the cursor visible. A `scroll_offset` in the model tracks the top visible line. The viewport follows the cursor — when the cursor moves past the visible area, the viewport shifts to keep it centered or near-edge.

### Mode Transitions

```
Normal ──(space to select, then m)──► Moving ──(Enter)──► Normal
  ▲                                      │       │
  └──────────(Esc)───────────────────────┘       │
                                                  ▼
Normal ◄──(y/n)── Confirm ◄──(name collision)────┘

Normal ──(x on profile)──► Confirm ──(y)──► Normal
  ▲                            │
  └─────────(n)────────────────┘

Normal ──(n)──► TextInput ──(Enter)──► Normal
  ▲                  │
  └────(Esc)─────────┘
```

## Operations

### Move Alias

1. Select one or more aliases from anywhere in the tree (multi-select across global, profiles, and project)
2. Press `m` — right column appears with destination tree
3. Navigate to destination header in right column (global, any profile, or project)
4. Press `Enter` — aliases are removed from source and added to destination
5. **Name collision:** If destination already has an alias with the same name, show a confirmation prompt before overwriting
6. **Same source/destination:** No-op, silent — no error message
7. Changes persist to disk immediately
8. Tree rebuilds, selection clears, right column disappears

### Delete Alias

- `x` on an alias item removes it from its profile/project
- Persists immediately
- No confirmation prompt (single alias, easily re-added)

### Delete Profile

- `x` on a profile header shows a confirmation prompt ("Delete profile 'name' and all its aliases? y/n")
- If confirmed: deletes the profile and all its aliases
- Dependents (child profiles) are re-parented to the deleted profile's parent
- Deleting the currently active profile clears the active marker (no active profile — only global and project aliases remain)
- No special-cased profiles — any profile can be deleted

### Create Profile

- `n` triggers an inline text input at the bottom of the screen
- Enter a name, press `Enter` to create, `Esc` to cancel
- New profile appears in the tree immediately
- Created as a root-level profile (no inheritance)

### Set Active Profile

- `s` on a profile header sets it as active
- `●`/`○` markers update immediately
- Persists to disk

## Architecture

### Crate Structure

The TUI lives in a **separate crate** (`am-tui`) to keep the `am` binary lean. The `am` binary runs on every `cd` (hook) and shell startup (init), so startup latency is critical — ratatui/crossterm must not be linked into it.

```
crates/
├── am/                 # existing — lib.rs (core types) + bin/am.rs (CLI)
└── am-tui/             # new — separate binary, depends on am (lib)
    ├── Cargo.toml      # depends on am, ratatui, crossterm
    └── src/
        ├── main.rs     # entry point: loads AppModel, calls run()
        ├── app.rs      // pub fn run(model: AppModel) -> Result<()>
        ├── model.rs    // TuiModel, TreeNode, NodeKind, Mode, Column
        ├── update.rs   // TuiMessage handling, state transitions
        ├── view.rs     // ratatui rendering (tree, columns, help bar)
        ├── tree.rs     // build/rebuild flattened tree from AppModel
        └── input.rs    // crossterm keypress → TuiMessage mapping
```

Users install both: `cargo install am am-tui`

The `am` CLI does **not** depend on `am-tui` at compile time. No feature flags, no cycle.

### Data Model

```rust
/// Identifies an alias by its scope and name — survives tree rebuilds.
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
enum AliasId {
    Global { alias_name: String },
    Profile { profile_name: String, alias_name: String },
    Project { alias_name: String },
}

struct TuiModel {
    app_model: AppModel,           // existing profiles, config, project aliases
    tree: Vec<TreeNode>,           // flattened tree for rendering/navigation
    cursor: usize,                 // raw index into Vec<TreeNode>; navigation skips non-navigable nodes
    selected: BTreeSet<AliasId>,   // multi-select set (identity-based, survives rebuilds)
    mode: Mode,                    // Normal | Moving | TextInput | Confirm
    dest_tree: Vec<TreeNode>,      // right column tree (headers only)
    dest_cursor: usize,            // cursor in destination column
    active_column: Column,         // Left | Right
    scroll_offset: usize,          // viewport top line for scrolling
}

struct TreeNode {
    kind: NodeKind,                // GlobalHeader | ProfileHeader | AliasItem | ProjectHeader
    depth: u16,                    // indentation level
    alias_id: Option<AliasId>,     // for AliasItem nodes — full identity for move/delete operations
    alias_command: Option<String>, // for AliasItem — displayed as dimmed second line
    is_active: bool,               // active profile marker (ProfileHeader only)
    label: String,                 // display text: profile name, alias name, or "project (.aliases)"
}

enum NodeKind {
    GlobalHeader,    // navigable, not selectable
    ProfileHeader,   // navigable, not selectable
    AliasItem,       // navigable and selectable
    ProjectHeader,   // navigable, not selectable
}

enum Mode {
    Normal,
    Moving,
    TextInput(String),             // new profile name being typed
    Confirm(ConfirmAction),        // awaiting y/n
}

enum ConfirmAction {
    DeleteProfile(String),
    OverwriteAliases(Vec<AliasId>, String),  // aliases to move, destination
}

enum Column {
    Left,
    Right,
}
```

### TuiMessage Enum

```rust
enum TuiMessage {
    // Navigation
    CursorUp,
    CursorDown,
    JumpTop,
    JumpBottom,

    // Selection
    ToggleSelect,          // space — toggle alias under cursor
    EnterMoveMode,         // m — show right column
    ExecuteMove,           // Enter — move selected to destination
    CancelMove,            // Esc — back to Normal

    // Column
    SwitchColumn,          // Tab

    // Management
    StartCreateProfile,    // n — enter TextInput mode
    DeleteItem,            // x — delete alias or profile under cursor
    SetActive,             // s — set profile as active

    // TextInput
    TextInputChar(char),
    TextInputConfirm,      // Enter in TextInput mode
    TextInputCancel,       // Esc in TextInput mode

    // Confirm
    ConfirmYes,
    ConfirmNo,

    // System
    Quit,
    Resize(u16, u16),      // terminal resized — re-check minimum size
}
```

### Data Flow (per frame)

1. **input.rs** — crossterm event → `TuiMessage`
2. **update.rs** — `TuiMessage` + `&mut TuiModel` → state mutation, optionally calls `AppModel` methods (move/delete/create) and persists to disk
3. **view.rs** — `&TuiModel` → ratatui `Frame` rendering

### Integration with Existing Code

- `TuiModel` wraps `AppModel` — reuses all existing profile/alias/project CRUD operations from the `am` library crate
- Move = `remove_alias()` from source + `add_alias()` to destination + `save()`. For global aliases, this uses `Config::add_alias`/`remove_alias` + `Config::save`. For profile aliases, `Profile::add_alias`/`remove_alias` + `ProfileConfig::save`. For project aliases, `ProjectAliases` methods.
- The `am` CLI gains no new code or dependencies for the TUI. Users run `am-tui` directly.

### Dependencies (am-tui crate only)

- `am` — core library (workspace dependency)
- `ratatui` — terminal UI rendering
- `crossterm` — terminal backend and input events

### Binary Startup Latency

The `am` binary's startup time must not regress. As part of the implementation:

1. **Baseline measurement** before adding `am-tui` — measure `am hook fish` and `am init fish` with `hyperfine`
2. **Post-implementation measurement** — confirm `am` binary is unchanged (no new dependencies linked)
3. Target: `am` commands should complete in under **5ms** for hook/init paths

## Constraints

### Terminal Size

- Minimum terminal size: 60 columns, 15 rows
- If the terminal is below minimum at startup, exit immediately with an error message asking for more space
- If the terminal is resized below minimum during use, exit with the same error message
- No cramped fallback layout

### Node Visibility

- **Global node (`🌐`):** Hidden when `Config.aliases` is empty. Appears once a global alias exists (e.g. moved there from another scope).
- **Project node (`📁`):** Hidden when no `.aliases` file exists in the cwd ancestry. Moving an alias to the project node when no file exists creates it in the **current working directory**.
- **Profile nodes:** Always visible, even when empty (they can receive aliases via move).
- Project alias discovery follows existing behavior: walks up from cwd, stops before `$HOME`.

## Out of Scope (V1)

- **Creating aliases** from within the TUI. Use `am add` from the CLI. The TUI focuses on reorganization (move, delete) and profile management (create, delete, activate).
- **Renaming aliases** within the TUI.
- **Editing alias commands** within the TUI.
