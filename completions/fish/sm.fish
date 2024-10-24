# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_sm_global_optspecs
	string join \n current-shell= h/help V/version
end

function __fish_sm_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_sm_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_sm_using_subcommand
	set -l cmd (__fish_sm_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c sm -n "__fish_sm_needs_command" -l current-shell -d 'The current shell sm runing in' -r
complete -c sm -n "__fish_sm_needs_command" -s h -l help -d 'Print help'
complete -c sm -n "__fish_sm_needs_command" -s V -l version -d 'Print version'
complete -c sm -n "__fish_sm_needs_command" -f -a "add" -d 'Add a new alias, path, or secret'
complete -c sm -n "__fish_sm_needs_command" -f -a "env" -d 'Load environment variables into the current shell'
complete -c sm -n "__fish_sm_needs_command" -f -a "init" -d 'Initialize the shell-manager for your shell, usually put `eval "$(sm init)"` in your shell rc file'
complete -c sm -n "__fish_sm_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c sm -n "__fish_sm_using_subcommand add; and not __fish_seen_subcommand_from alias path secret help" -s h -l help -d 'Print help'
complete -c sm -n "__fish_sm_using_subcommand add; and not __fish_seen_subcommand_from alias path secret help" -s V -l version -d 'Print version'
complete -c sm -n "__fish_sm_using_subcommand add; and not __fish_seen_subcommand_from alias path secret help" -f -a "alias" -d 'Add a new alias'
complete -c sm -n "__fish_sm_using_subcommand add; and not __fish_seen_subcommand_from alias path secret help" -f -a "path" -d 'Add a new path'
complete -c sm -n "__fish_sm_using_subcommand add; and not __fish_seen_subcommand_from alias path secret help" -f -a "secret" -d 'Add a new secret'
complete -c sm -n "__fish_sm_using_subcommand add; and not __fish_seen_subcommand_from alias path secret help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from alias" -s d -l directory -d 'Directory-specific flag'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from alias" -s l -l long -d 'Long alias flag'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from alias" -s h -l help -d 'Print help'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from alias" -s V -l version -d 'Print version'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from path" -s d -l directory -d 'Directory-specific flag'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from path" -s h -l help -d 'Print help'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from path" -s V -l version -d 'Print version'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from secret" -s d -l directory -d 'Directory-specific flag'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from secret" -s h -l help -d 'Print help'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from secret" -s V -l version -d 'Print version'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "alias" -d 'Add a new alias'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "path" -d 'Add a new path'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "secret" -d 'Add a new secret'
complete -c sm -n "__fish_sm_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c sm -n "__fish_sm_using_subcommand env" -s h -l help -d 'Print help'
complete -c sm -n "__fish_sm_using_subcommand env" -s V -l version -d 'Print version'
complete -c sm -n "__fish_sm_using_subcommand init" -s h -l help -d 'Print help'
complete -c sm -n "__fish_sm_using_subcommand init" -s V -l version -d 'Print version'
complete -c sm -n "__fish_sm_using_subcommand help; and not __fish_seen_subcommand_from add env init help" -f -a "add" -d 'Add a new alias, path, or secret'
complete -c sm -n "__fish_sm_using_subcommand help; and not __fish_seen_subcommand_from add env init help" -f -a "env" -d 'Load environment variables into the current shell'
complete -c sm -n "__fish_sm_using_subcommand help; and not __fish_seen_subcommand_from add env init help" -f -a "init" -d 'Initialize the shell-manager for your shell, usually put `eval "$(sm init)"` in your shell rc file'
complete -c sm -n "__fish_sm_using_subcommand help; and not __fish_seen_subcommand_from add env init help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c sm -n "__fish_sm_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "alias" -d 'Add a new alias'
complete -c sm -n "__fish_sm_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "path" -d 'Add a new path'
complete -c sm -n "__fish_sm_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "secret" -d 'Add a new secret'
