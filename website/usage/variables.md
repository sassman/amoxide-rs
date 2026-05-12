# Variables <VersionBadge v="0.9.0" />

<div v-pre>

Named placeholders in alias commands. Set once with `am var`, and the value is baked into the rendered shell output. They complement positional arguments — `{{1}}`, `{{@}}` — described in [Parameterized Aliases](/advanced/parameterized-aliases).

Use variables when a fragment recurs across several aliases — a Kubernetes namespace, a registry prefix, an AWS profile name, a target triple. Set it in one place; every alias that references it picks up the value the moment you change it.

## Quick Example

Three project-local aliases that all share a Kubernetes namespace:

```sh
am var set -l ns monitoring
am add -l klogs "kubectl -n {{ns}} logs -f {{1}}"
am add -l kget  "kubectl -n {{ns}} get pods"
am add -l kdesc "kubectl -n {{ns}} describe {{1}}"
```

Switch the namespace later:

```sh
am var set -l ns staging
```

`klogs`, `kget`, and `kdesc` all update on the spot — no shell reload, no resync step.

## Setting Variables

Each row shows the long form and its two short equivalents — all three lines mean the same thing.

| Command | Description |
|---------|-------------|
| `am var set <name> <value>`<br>`am var s   <name> <value>`<br>`am v   s   <name> <value>` | Set or replace a value |
| `am var unset <name>`<br>`am var u     <name>`<br>`am v   u     <name>` | Remove a variable |
| `am var get <name>`<br>`am var g   <name>`<br>`am v   g   <name>` | Print value to stdout |
| `am var list`<br>`am var l`<br>`am v   l` | List variables |

All four commands take an optional scope flag: `-g` for global, `-p <profile>` for a profile, `-l` for the project's `.aliases` file.

**Default scope.** Without a scope flag, mutations land in the active profile if one is active, otherwise global. `-l` is never implicit — writing to a project's `.aliases` always requires the explicit flag (and a trusted project).

**Names.** Letters, digits, `_`, and `-`. Must start with a letter or `_`. `am var set` errors with the specific rule violated if a name is rejected.

## Scope-Local Resolution

An alias defined at scope *S* substitutes `{{name}}` references against the variable set at scope *S* — nothing else. There is no fall-through across scopes and no merging.

This is deliberate. Two AWS profiles, each with their own `aws-prof`:

```sh
am profile add aws-prod
am profile add aws-staging

am var set -p aws-prod    aws-prof prod-readonly
am var set -p aws-staging aws-prof staging-admin

am add -p aws-prod    awsls 'aws --profile {{aws-prof}} s3 ls'
am add -p aws-staging awsls 'aws --profile {{aws-prof}} s3 ls'
```

`am use aws-prod` → `awsls` resolves with `prod-readonly`. Switch to `aws-staging` → the same alias name expands with `staging-admin`. Neither profile sees the other's value.

The trade-off: if you want the same value in both global and a profile, set it in both. The small redundancy is the price of predictable resolution — a `cd` into a project directory will never silently rewrite a global alias.

## Within a Scope

Inside one scope, variables are a shared namespace. Every alias at that scope can reference every variable at that scope — there is no per-alias privacy.

Values are inserted as **literal text**. Quotes and shell metacharacters pass through verbatim; amoxide does not escape them. Whatever quoting you write around `{{name}}` is what ends up in the rendered alias:

```sh
am var set -g msg "hello world"
am add -g greet 'echo "{{msg}}"'
# greet → echo "hello world"
```

::: danger 🚨 Variables are not secrets
Variable values land in `am init` output, in `am export` bundles, in shell history when you set them, and in process arguments during sync. **Never store API tokens, passwords, SSH keys, or any credential as a variable.** Use variables for paths, flags, namespaces, endpoints, target triples — values you would happily commit to a checked-in script.

For secrets, reach for your shell's existing tooling: a password manager CLI, `pass`, `1password-cli`, `op run`, or `direnv` with a gitignored `.envrc`.
:::

## Undefined Variables

If an alias references `{{name}}` and the variable is not defined at its scope, the alias is skipped on that sync: a warning goes to stderr, the alias is unloaded from the shell if it had been loaded previously, and unrelated aliases continue to sync normally.

Define the missing variable (or remove the reference) and the alias returns on the next sync.

</div>
