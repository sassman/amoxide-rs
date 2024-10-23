# The Shell-Manager

> The shell-manager (`sm`) is only for the most laziest among you. It helps you to manage your aliases, paths and secret env variables on the shell, either globally or project (like directory) specific.

## Setup

after installation you can add the following line to your `~/.bashrc` or `~/.zshrc` to have the shell-manager loaded on every shell start

```shell
chpwd() {
  eval "$(sm env)"
}
```

Or just run this snipped that adds the line to your `~/.zshrc`

```shell
echo 'chpwd() { eval "$(sm env)"; }' >> ~/.zshrc
```

## Usage by Example

Once this is done, you can get really lazy so instead of editing your `~/.bashrc` or `~/.zshrc` to add a new alias you can simple call this command

```shell
$ sm add alias ll "ls -lha"
$ sm add alias gs "git status"
```

Ok, so far so good. But now let's assume you are working on a project like the rust compiler, and you want to have a new alias that is **only** available when working on this very project.

```shell
# t is an alias for test, just like `cargo test` but in the rustc context
$ sm add alias -d t "./x.py test"
```

The `-d` or `--directory` will ensure this very alias shows only up when in this or any sub directories. You could say it's project or directory bound.

If you want to add simply the last executed command as an alias you can do so by not naming the alias command

```shell
# example command
$ echo "Hello World!"

# add the last command above as an alias h
$ sm add alias h
```

Also you can be lazy about the verb and nound like so:

```shell
$ sm a a l 'ls -lha'
#    ^ ^ ^
#    | | |
#    | | +---- this is the alais `l`
#    | +---- this is the noun `alias`
#    +---- this is the verb `add`
```

This does the same as `sm add alias l 'ls -lha'`

## Future features (WIP)

It's also possible to add a "longer alias" that you would usually put in a shell function, just like this:

```shell
$ sm add alias -l gp
# this is now the code for the alias
git fetch origin
git stash
git pull --rebase
git push --force-with-lease origin HEAD
git stash pop
```

There is more to discover, you can also be lazy about paths (`$PATH`) and even load secrets into env variables only in specific directories without the risk of having them laying around readable files.