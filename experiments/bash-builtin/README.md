# Example for a bash builtin

## Activate the builtin

```shell
$ cargo build -p sm_bash_builtin
$ enable -f target/debug/libsm_bash_builtin.so counter

$ counter
0

$ counter
1
```

## Deactivate the builtin

```shell
$ enable -d counter
```