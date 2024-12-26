zsm:
	cargo build -p sm-zsh-builtin
	cp target/debug/libsm_zsh_builtin.dylib target/debug/zsm.so
	$(zsh module_path=(/usr/lib/zsh/5.9 ${PWD}/target/debug))
	$(zsh zmodload zsm)