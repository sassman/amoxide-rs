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

This loads your profile aliases and installs a cd hook for project aliases.

## Usage by Example

### Adding aliases

```shell
$ am add ll "ls -lha"
$ am add gs "git status"
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

Profiles let you group aliases by context (e.g., `rust`, `node`, `work`):

```shell
# Create and activate a profile
$ am profile rust

# Add aliases to a specific profile
$ am add -p rust ct "cargo test"
$ am add -p rust cb "cargo build"

# List all profiles and their aliases
$ am profiles
```

The active profile's aliases are loaded on every shell start via `am init`.

### Project aliases (`.aliases` file)

Create a `.aliases` file in any project directory:

```toml
# /path/to/my-project/.aliases
[aliases]
t = "./x.py test"
b = "./x.py build"
```

These aliases are automatically loaded when you `cd` into the project (or any subdirectory) and unloaded when you leave. Works like direnv, but for aliases.
