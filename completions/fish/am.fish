# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_am_global_optspecs
	string join \n h/help V/version
end

function __fish_am_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_am_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_am_using_subcommand
	set -l cmd (__fish_am_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c am -n "__fish_am_needs_command" -s h -l help -d 'Print help'
complete -c am -n "__fish_am_needs_command" -s V -l version -d 'Print version'
complete -c am -n "__fish_am_needs_command" -a "add" -d 'Add a new alias'
complete -c am -n "__fish_am_needs_command" -a "profile" -d 'Add or activate a profile'
complete -c am -n "__fish_am_needs_command" -a "profiles" -d 'List all profiles'
complete -c am -n "__fish_am_needs_command" -a "env" -d 'Print and set up required environment variables for am'
complete -c am -n "__fish_am_needs_command" -a "init"
complete -c am -n "__fish_am_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c am -n "__fish_am_using_subcommand add" -s p -l profile -d 'The name of the profile to add the alias to, if not provided, the active profile will be used. If no profile is active, the default profile will be used' -r
complete -c am -n "__fish_am_using_subcommand add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c am -n "__fish_am_using_subcommand add" -s V -l version -d 'Print version'
complete -c am -n "__fish_am_using_subcommand profile" -s i -l inherits -d 'The optional base profile to inherit from' -r
complete -c am -n "__fish_am_using_subcommand profile" -l on-activate -d 'Execute this on activation of the profile' -r
complete -c am -n "__fish_am_using_subcommand profile" -l list
complete -c am -n "__fish_am_using_subcommand profile" -l print-full-init
complete -c am -n "__fish_am_using_subcommand profile" -s h -l help -d 'Print help'
complete -c am -n "__fish_am_using_subcommand profile" -s V -l version -d 'Print version'
complete -c am -n "__fish_am_using_subcommand profiles" -s h -l help -d 'Print help'
complete -c am -n "__fish_am_using_subcommand profiles" -s V -l version -d 'Print version'
complete -c am -n "__fish_am_using_subcommand env" -s h -l help -d 'Print help'
complete -c am -n "__fish_am_using_subcommand env" -s V -l version -d 'Print version'
complete -c am -n "__fish_am_using_subcommand init" -s h -l help -d 'Print help'
complete -c am -n "__fish_am_using_subcommand init" -s V -l version -d 'Print version'
complete -c am -n "__fish_am_using_subcommand help; and not __fish_seen_subcommand_from add profile profiles env init help" -f -a "add" -d 'Add a new alias'
complete -c am -n "__fish_am_using_subcommand help; and not __fish_seen_subcommand_from add profile profiles env init help" -f -a "profile" -d 'Add or activate a profile'
complete -c am -n "__fish_am_using_subcommand help; and not __fish_seen_subcommand_from add profile profiles env init help" -f -a "profiles" -d 'List all profiles'
complete -c am -n "__fish_am_using_subcommand help; and not __fish_seen_subcommand_from add profile profiles env init help" -f -a "env" -d 'Print and set up required environment variables for am'
complete -c am -n "__fish_am_using_subcommand help; and not __fish_seen_subcommand_from add profile profiles env init help" -f -a "init"
complete -c am -n "__fish_am_using_subcommand help; and not __fish_seen_subcommand_from add profile profiles env init help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
