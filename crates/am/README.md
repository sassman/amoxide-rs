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
aman add t "npm test"
aman add d "npm run dev"
aman add b "npm run build"
```

- In a Python project:

```sh
aman add t "pytest"
aman add d "python manage.py runserver"
aman add b "python setup.py build"
```

- In a Docker project:

```sh
aman add up "docker-compose up -d"
aman add down "docker-compose down"
aman add logs "docker-compose logs -f"
```

2. Profiles for groups of aliases

- Profile for JS Development,
  - with a profile activation command, here we use `fnm` to switch between node versions

```sh
aman profile js --activate-with 'eval $(fnm env) && fnm use lts'
aman add t "npm test"
aman add d "npm run dev"
aman add b "npm run build"
```

- Profile for Python Development,
  - with a profile activation command, here we use `pyenv` to switch between python versions

```sh
aman profile py --activate-with 'eval "$(pyenv init -)" && pyenv shell 3.8.2'
aman add t "pytest"
aman add d "python manage.py runserver"
aman add b "python setup.py build"
```

Now we can switch between the profiles with `aman profile js` and `aman profile py`. The activation of a profile will also deactivate the previous profile, so no alias conflicts will happen.

Profiles can also inherit from other profiles, so you can have a base profile for all your projects and then add technology specific aliases on top of that. A good example is git aliases, that you might want to have in every project. You can create a base profile with those aliases and then inherit from that profile in your project specific profiles.

- Base Profile with git aliases

```sh
aman profile base
aman add g "git"
aman add gs "git status"
aman add ga "git add"
aman add gc "git commit -s -m"

aman profile js --inherits bas
aman add t "npm test"
aman add d "npm run dev"
aman add b "npm run build"
```