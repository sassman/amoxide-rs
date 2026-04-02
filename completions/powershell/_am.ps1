
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'am' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'am'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'am' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an alias')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'List all profiles and project aliases')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check if the shell is set up correctly')
            [CompletionResult]::new('profile', 'profile', [CompletionResultType]::ParameterValue, 'Manage profiles (defaults to listing when no subcommand given)')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Print shell init code')
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Guided setup — adds amoxide to your shell profile')
            [CompletionResult]::new('tui', 'tui', [CompletionResultType]::ParameterValue, 'Launch the interactive TUI for managing aliases and profiles')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export aliases to stdout as TOML')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import aliases from stdin or a URL')
            [CompletionResult]::new('share', 'share', [CompletionResultType]::ParameterValue, 'Generate a share command for posting aliases to a pastebin service')
            [CompletionResult]::new('hook', 'hook', [CompletionResultType]::ParameterValue, 'Internal: called by the cd hook to load/unload project aliases')
            [CompletionResult]::new('reload', 'reload', [CompletionResultType]::ParameterValue, 'Internal: called by the am wrapper to reload profile aliases after switching')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;add' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Profile to add the alias to (defaults to active profile)')
            [CompletionResult]::new('--profile', '--profile', [CompletionResultType]::ParameterName, 'Profile to add the alias to (defaults to active profile)')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Add to the project''s .aliases file instead of a profile')
            [CompletionResult]::new('--local', '--local', [CompletionResultType]::ParameterName, 'Add to the project''s .aliases file instead of a profile')
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Add as a global alias (always loaded, independent of profile)')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Add as a global alias (always loaded, independent of profile)')
            [CompletionResult]::new('--raw', '--raw', [CompletionResultType]::ParameterName, 'Disable {{N}} template detection (treat command as literal)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;remove' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Profile to remove the alias from (defaults to active profile)')
            [CompletionResult]::new('--profile', '--profile', [CompletionResultType]::ParameterName, 'Profile to remove the alias from (defaults to active profile)')
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Remove a global alias')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Remove a global alias')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;ls' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;status' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;profile' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new profile')
            [CompletionResult]::new('use', 'use', [CompletionResultType]::ParameterValue, 'Toggle a profile as active/inactive, optionally at a specific priority')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;profile;add' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;profile;use' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Activate at specific priority position (1-based). Repositions if already active')
            [CompletionResult]::new('--priority', '--priority', [CompletionResultType]::ParameterName, 'Activate at specific priority position (1-based). Repositions if already active')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;profile;remove' {
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Skip confirmation prompt')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Skip confirmation prompt')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;profile;list' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;profile;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new profile')
            [CompletionResult]::new('use', 'use', [CompletionResultType]::ParameterValue, 'Toggle a profile as active/inactive, optionally at a specific priority')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;profile;help;add' {
            break
        }
        'am;profile;help;use' {
            break
        }
        'am;profile;help;remove' {
            break
        }
        'am;profile;help;list' {
            break
        }
        'am;profile;help;help' {
            break
        }
        'am;init' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;setup' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;tui' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;export' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Operate on specific profile(s) — can be repeated')
            [CompletionResult]::new('--profile', '--profile', [CompletionResultType]::ParameterName, 'Operate on specific profile(s) — can be repeated')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Operate on project-local aliases')
            [CompletionResult]::new('--local', '--local', [CompletionResultType]::ParameterName, 'Operate on project-local aliases')
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Operate on global aliases')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Operate on global aliases')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Operate on everything (global + all profiles + local)')
            [CompletionResult]::new('-b', '-b', [CompletionResultType]::ParameterName, 'Encode output as base64')
            [CompletionResult]::new('--base64', '--base64', [CompletionResultType]::ParameterName, 'Encode output as base64')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;import' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Operate on specific profile(s) — can be repeated')
            [CompletionResult]::new('--profile', '--profile', [CompletionResultType]::ParameterName, 'Operate on specific profile(s) — can be repeated')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Operate on project-local aliases')
            [CompletionResult]::new('--local', '--local', [CompletionResultType]::ParameterName, 'Operate on project-local aliases')
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Operate on global aliases')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Operate on global aliases')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Operate on everything (global + all profiles + local)')
            [CompletionResult]::new('-b', '-b', [CompletionResultType]::ParameterName, 'Decode base64 input before parsing')
            [CompletionResult]::new('--base64', '--base64', [CompletionResultType]::ParameterName, 'Decode base64 input before parsing')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Skip all confirmation prompts')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Skip all confirmation prompts')
            [CompletionResult]::new('--trust', '--trust', [CompletionResultType]::ParameterName, 'DANGER: Skip safety checks for suspicious content (escape sequences). Only use for your own exports. Never trust external input blindly — it can carry invisible escape sequences that hide malicious commands')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;share' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Operate on specific profile(s) — can be repeated')
            [CompletionResult]::new('--profile', '--profile', [CompletionResultType]::ParameterName, 'Operate on specific profile(s) — can be repeated')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Operate on project-local aliases')
            [CompletionResult]::new('--local', '--local', [CompletionResultType]::ParameterName, 'Operate on project-local aliases')
            [CompletionResult]::new('-g', '-g', [CompletionResultType]::ParameterName, 'Operate on global aliases')
            [CompletionResult]::new('--global', '--global', [CompletionResultType]::ParameterName, 'Operate on global aliases')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Operate on everything (global + all profiles + local)')
            [CompletionResult]::new('--termbin', '--termbin', [CompletionResultType]::ParameterName, 'Generate command for termbin.com (netcat)')
            [CompletionResult]::new('--paste-rs', '--paste-rs', [CompletionResultType]::ParameterName, 'Generate command for paste.rs (curl)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;hook' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;reload' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove an alias')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'List all profiles and project aliases')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Check if the shell is set up correctly')
            [CompletionResult]::new('profile', 'profile', [CompletionResultType]::ParameterValue, 'Manage profiles (defaults to listing when no subcommand given)')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Print shell init code')
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Guided setup — adds amoxide to your shell profile')
            [CompletionResult]::new('tui', 'tui', [CompletionResultType]::ParameterValue, 'Launch the interactive TUI for managing aliases and profiles')
            [CompletionResult]::new('export', 'export', [CompletionResultType]::ParameterValue, 'Export aliases to stdout as TOML')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import aliases from stdin or a URL')
            [CompletionResult]::new('share', 'share', [CompletionResultType]::ParameterValue, 'Generate a share command for posting aliases to a pastebin service')
            [CompletionResult]::new('hook', 'hook', [CompletionResultType]::ParameterValue, 'Internal: called by the cd hook to load/unload project aliases')
            [CompletionResult]::new('reload', 'reload', [CompletionResultType]::ParameterValue, 'Internal: called by the am wrapper to reload profile aliases after switching')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;help;add' {
            break
        }
        'am;help;remove' {
            break
        }
        'am;help;ls' {
            break
        }
        'am;help;status' {
            break
        }
        'am;help;profile' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new profile')
            [CompletionResult]::new('use', 'use', [CompletionResultType]::ParameterValue, 'Toggle a profile as active/inactive, optionally at a specific priority')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a profile')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all profiles')
            break
        }
        'am;help;profile;add' {
            break
        }
        'am;help;profile;use' {
            break
        }
        'am;help;profile;remove' {
            break
        }
        'am;help;profile;list' {
            break
        }
        'am;help;init' {
            break
        }
        'am;help;setup' {
            break
        }
        'am;help;tui' {
            break
        }
        'am;help;export' {
            break
        }
        'am;help;import' {
            break
        }
        'am;help;share' {
            break
        }
        'am;help;hook' {
            break
        }
        'am;help;reload' {
            break
        }
        'am;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
