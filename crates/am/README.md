# The Alias Manager

## Concetps

- **Alias**: A short name for a command or a sequence of commands.
- **Profile**: A group of aliases that can be activated together. Useful for global but specific workflows. E.g. doing JS development, Python development but without project specific aliases.
  - **Activation Command**: A command that is executed when a profile is activated.
  - **Deactivation Command**: A command that is executed when a profile is deactivated.
- **Directory Specific Alias**: An alias that is only available in a specific directory or it's subdirectories. Useful for project specific aliases that can be also stored in git and shared with team mates.

To avoid confusion between Profiles and Directory Specific Aliases, we will refer to the former as **Profile Context** and the latter as **Project Context**. Also there is a difference on where those are stored. Profile Context is stored in the user's home directory and Project Context is stored in the project's directory.

## Usage Examples

1. Project Context Examples

- In a Node.js project:

```sh
am add t "npm test"
am add d "npm run dev"
am add b "npm run build"
```

- In a Python project:

```sh
am add t "pytest"
am add d "python manage.py runserver"
am add b "python setup.py build"
```

- In a Docker project:

```sh
am add up "docker-compose up -d"
am add down "docker-compose down"
am add logs "docker-compose logs -f"
```

2. Profiles for groups of aliases

- Profile for JS Development,
  - with a profile activation command, here we use `fnm` to switch between node versions

```sh
am profile js --activate-with 'eval $(fnm env) && fnm use lts'
am add t "npm test"
am add d "npm run dev"
am add b "npm run build"
```

- Profile for Python Development,
  - with a profile activation command, here we use `pyenv` to switch between python versions

```sh
am profile py --activate-with 'eval "$(pyenv init -)" && pyenv shell 3.8.2'
am add t "pytest"
am add d "python manage.py runserver"
am add b "python setup.py build"
```

Now we can switch between the profiles with `am profile js` and `am profile py`. The activation of a profile will also deactivate the previous profile, so no alias conflicts will happen.

Profiles can also inherit from other profiles, so you can have a base profile for all your projects and then add technology specific aliases on top of that. A good example is git aliases, that you might want to have in every project. You can create a base profile with those aliases and then inherit from that profile in your project specific profiles.

- Base Profile with git aliases

```sh
am profile base
am add g "git"
am add gs "git status"
am add ga "git add"
am add gc "git commit -s -m"

am profile js --inherits bas
am add t "npm test"
am add d "npm run dev"
am add b "npm run build"
```

## Implementation Details

### Fish Shell

Fish has an autoload feature that allows you to define functions in separate files and load them when they are called. am uses this feature and persists aliases as functions in separate files in the user's home directory.

When `am env fish` is called those files are written to the fish configuration directory and autoloaded. This is subject of change in the future, as it might be better to write the function that correspons to an alias to the fish configuration directory when the alias is added.