# Example of a zsh builtin

## Activate the builtin

```zsh
$ cargo build -p sm_zsh_builtin
$ cp target/debug/libsm_zsh_builtin.dylib target/debug/rgreeter.so
$ module_path=($module_path $PWD/target/debug)
$ zmodload rgreeter
```

Then you can use it like this:

```zsh
$ greet
Hello, world!
```
