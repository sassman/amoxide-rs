# Architecture Assessment

## Current State

The CLI uses an Elm/TEA-inspired message-passing architecture:

```
bin/am.rs (parse CLI → construct Message)
    → update.rs (match Message → mutate AppModel → return Option<Message>)
    → loop until None
```

### Module Responsibilities

| Module | Role |
|---|---|
| `cli.rs` | Clap-based CLI definition |
| `messages.rs` | `Message` enum + `AddAliasProfile` helper type |
| `update.rs` | `AppModel` struct + `update()` function that dispatches messages |
| `bin/am.rs` | Entry point: parses CLI, constructs messages, runs the update loop |
| `config.rs` | Active profile name persistence (`config.toml`) |
| `profile.rs` | `ProfileConfig` + `Profile` — TOML-based profile storage |
| `project.rs` | `.aliases` file parsing with directory walk-up |
| `init.rs` | Shell init code generation (fish/zsh) — pure function, returns String |
| `hook.rs` | cd hook output generation — pure function, returns String |
| `display.rs` | Profile tree rendering — pure function, returns String |
| `shell/` | `Shell` trait + implementations (Fish, Zsh, NixShell) |
| `dirs.rs` | Path helpers (`config_dir`, `home_dir`, `relative_path`) |
| `alias.rs` | `AliasName`, `AliasSet`, `TomlAlias` data types |

### What Works Well

- **Pure rendering functions** — `init.rs`, `hook.rs`, `display.rs` all return Strings. Easy to test, no side effects.
- **AppModel is simple** — just `Config` + `ProfileConfig`, no bloat.
- **Shell trait abstraction** — clean separation between shell-specific output and business logic.

### What Could Be Improved

#### 1. Message loop adds complexity without value

This is a CLI that runs one command and exits. There is no event loop, no UI re-rendering, no async. The `Option<Message>` return + `while let` loop is used only for chaining `SaveProfiles`/`SaveConfig` after mutations — that's just two function calls pretending to be an event system.

In `bin/am.rs`, `update()` is already called multiple times manually before entering the loop (e.g., `CreateOrUpdateProfile` → `SaveProfiles` → `SaveConfig`). The "loop" rarely iterates more than once.

#### 2. SaveProfiles and SaveConfig are leaked implementation details

They're not real user-facing commands — they're persistence side effects. The caller has to know to chain them after every mutation. A handler returning `Some(Message::SaveProfiles)` is just a deferred `model.profile_config().save()` call.

#### 3. Duplicated profile resolution

The `AddAliasProfile → &mut Profile` resolution is copy-pasted between `AddAlias` and `RemoveAlias` handlers. Every new alias mutation will need the same boilerplate. A shared `resolve_profile_mut(Option<&str>)` helper would eliminate this.

#### 4. DoNothing is a code smell

If a code path has nothing to do, it shouldn't need to construct a message to say so.

#### 5. println! inside update() couples to stdout

`ListProfiles`, `InitShell`, and `Hook` all print directly. This makes the update function untestable for integration tests. The pure rendering functions already return Strings — the `print!` call should happen in `bin/am.rs`.

#### 6. Adding a command touches 3 files instead of 2

Currently: `messages.rs` (add variant) + `update.rs` (add handler) + `bin/am.rs` (add CLI→message mapping). With direct methods on `AppModel`, it would be: `update.rs` (add method) + `bin/am.rs` (add match arm).

### Proposed Refactoring Direction

Replace the message enum with direct methods on `AppModel`:

```rust
impl AppModel {
    // Mutations
    pub fn add_alias(&mut self, name: String, cmd: String, profile: Option<&str>) -> Result<()>
    pub fn remove_alias(&mut self, name: &str, profile: Option<&str>) -> Result<()>
    pub fn create_or_update_profile(&mut self, name: String, inherits: Option<String>) -> Result<bool>
    pub fn activate_profile(&mut self, name: &str) -> Result<()>

    // Queries (return String, don't print)
    pub fn list_profiles(&self) -> String
    pub fn init_shell(&self, shell: &Shells) -> String
    pub fn hook(&self, shell: &Shells) -> Result<String>

    // Persistence
    pub fn save(&self) -> Result<()>           // both config + profiles
    pub fn save_profiles(&self) -> Result<()>
    pub fn save_config(&self) -> Result<()>
}
```

Then `bin/am.rs` becomes flat match arms with no message loop:

```rust
Commands::Add { .. } => { model.add_alias(..)?; model.save_profiles()?; }
Commands::Ls => println!("{}", model.list_profiles()),
Commands::Init { shell } => print!("{}", model.init_shell(&shell)),
```

**Impact:** Delete `messages.rs`, rewrite `update.rs` and `bin/am.rs`. Zero test changes needed — no existing tests reference `Message` or `AddAliasProfile`.

**Trade-off:** If a TUI is ever added, the Elm architecture would be useful. But that's hypothetical and can be introduced then.
