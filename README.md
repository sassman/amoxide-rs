# The Shell-Manager

> The shell-manager (`sm`) is only for the most laziest among you. It helps you to manage your aliases, paths and secret env variables on the shell, either globally or project (like directory) specific.

## An Example

to load everything into your current shell (zsh or bash), you can simply call this

```shell
# something you would put in your ~/.bashrc or ~/.zshrc
$ eval "$(sm env)"
```

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