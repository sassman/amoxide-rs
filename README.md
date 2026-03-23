# The Alias-Manager

> The alias-manager (`am`) is for the most lazy folks like me. It helps to manage your shell aliases either globally or profile or project specific.

Q: What does Globally mean?
A: It's as a regular shell alias right now works - always present.

Q: What is Profile specific then?
A: A Profile is simply a name like `node development` or `git stuff` under which aliases are collected - like a category of purpose for aliases.

Q: What is then project specific?
A: Really imagine a specific project, like you are working on this very rust backend - with project specific aliases.

Note: Profiles can be composed upon another. Like your node profile should leverage some git aliases, then `node development -> git stuff` would cause they are loaded upwards the dependency tree.

## Setup

Add one line to your shell config:

```fish
# ~/.config/fish/config.fish
am init fish | source
```

```zsh
# ~/.zshrc
eval "$(am init zsh)"
```

This does two things:
1. Loads aliases from your active profile into the current shell
2. Installs a cd hook that automatically loads/unloads project aliases (from `.aliases` files) when you change directories

## Usage by Example

### Adding and removing aliases

```shell
$ am add ll "ls -lha"
$ am add gs "git status"
$ am remove gs                 # remove from active profile
$ am r gs                      # short form
$ am remove -p rust ct         # remove from a specific profile
```

Short form works too:

```shell
$ am a l ls -lha
#    ^ ^ ^-----^
#    | |       |
#    | |       +---- this is alias command `ls -lha`
#    | +---- this is the alias name `l`
#    +---- this is the verb `add`
```

### Profiles

Profiles let you group aliases by context (e.g., `rust`, `node`, `git`):

```shell
# Add a profile
$ am profile add rust
$ am p a rust                  # short form

# Add a profile that inherits from another
$ am profile add rust --inherits git

# Set the active profile
$ am profile set rust
$ am p s rust                  # short form

# Add aliases to a specific profile
$ am add -p rust ct "cargo test"
$ am add -p rust cb "cargo build"

# List all profiles, aliases, and active project aliases
$ am profile                   # default action
$ am profile list              # explicit
$ am l                         # shortest form
```

The active profile's aliases are loaded on every shell start via `am init`.

Listing profiles shows a tree with inheritance:

```
○ git
│ gs → git status
│ gp → git push
├─○ node
│   nr → npm run
╰─● rust (active)
    ct → cargo test
    cb → cargo build
```

If you're inside a project with a `.aliases` file, the listing also shows those:

```
○ git
│ gs → git status
╰─● rust (active)
    ct → cargo test

📁 project aliases (.aliases)
  t → ./x.py test
  b → ./x.py build
```

### Project aliases (`.aliases` file)

You can add project-local aliases with the `-l`/`--local` flag:

```shell
$ cd ~/my-project
$ am add -l t "./x.py test"   # writes to .aliases in current directory
$ am add -l b "./x.py build"
```

If no `.aliases` file exists, one is created in the current directory. If a `.aliases` already exists further up the directory tree, you'll be asked whether you meant to add to that one instead.

You can also create or edit the `.aliases` file directly:

```toml
# /path/to/my-project/.aliases
[aliases]
t = "./x.py test"
b = "./x.py build"
```

These aliases are automatically loaded when you `cd` into the project (or any subdirectory) and unloaded when you leave. Works like direnv, but for aliases.

Under the hood, `am init` installs a cd hook that calls `am hook <shell>` on every directory change. The hook walks up from the current directory looking for a `.aliases` file (stopping before `$HOME`), unloads any previously active project aliases, and loads the new ones.
