# The Alias-Manager

> The alias-manager (`am`) is only for the most laziest among you. It helps you to manage your aliases, paths and secret env variables on the shell, either globally or project (like directory) specific.

## Setup

after installation you can add the following line to your `~/.bashrc` or `~/.zshrc` to have the alias-manager loaded on every shell start

```shell
# puth this in your ~/.zshrc
eval "$(am init zsh)"
```

## Usage by Example

Once this is done, you can get really lazy so instead of editing your `~/.bashrc` or `~/.zshrc` to add a new alias you can simple call this command

```shell
$ am add ll "ls -lha"
$ am add gs "git status"
```

Ok, so far so good. But now let's assume you are working on a project like the rust compiler, and you want to have a new alias that is **only** available when working on this very project.

```shell
# t is an alias for test, just like `cargo test` but in the rustc context
$ am add -d t "./x.py test"
```

The `-d` or `--directory` will ensure this very alias shows only up when in this or any sub directories. You could say it's project or directory bound.

If you want to add simply the last executed command as an alias you can do so by not naming the alias command

```shell
# example command
$ echo "Hello World!"

# add the last command above as an alias h
$ am add h
```

Also you can be lazy about the verb and nound like so:

```shell
$ am a l ls -lha
#    ^ ^ ^-----^
#    | |       |
#    | |       +---- this is alias command `ls -lha`
#    | +---- this is the alias name `l`
#    +---- this is the verb `add`
```

This does the same as `am a l ls -lha`

## Future features (WIP)

It's also possible to add a "longer alias" that you would usually put in a shell function, just like this:

```shell
$ am add -l gp
# this is now the code for the alias
git fetch origin
git stash
git pull --rebase
git push --force-with-lease origin HEAD
git stash pop
```

There is more to discover, you can also be lazy about paths (`$PATH`) and even load secrets into env variables only in specific directories without the risk of having them laying around readable files.