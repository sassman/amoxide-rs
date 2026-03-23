
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
            [CompletionResult]::new('profile', 'profile', [CompletionResultType]::ParameterValue, 'Add or activate a profile')
            [CompletionResult]::new('profiles', 'profiles', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Print shell init code (eval in your shell rc file)')
            [CompletionResult]::new('hook', 'hook', [CompletionResultType]::ParameterValue, 'Internal: called by the cd hook to load/unload project aliases')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;add' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Profile to add the alias to (defaults to active profile)')
            [CompletionResult]::new('--profile', '--profile', [CompletionResultType]::ParameterName, 'Profile to add the alias to (defaults to active profile)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;profile' {
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'Base profile to inherit from')
            [CompletionResult]::new('--inherits', '--inherits', [CompletionResultType]::ParameterName, 'Base profile to inherit from')
            [CompletionResult]::new('--list', '--list', [CompletionResultType]::ParameterName, 'List all profiles')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;profiles' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;init' {
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
        'am;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('profile', 'profile', [CompletionResultType]::ParameterValue, 'Add or activate a profile')
            [CompletionResult]::new('profiles', 'profiles', [CompletionResultType]::ParameterValue, 'List all profiles')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Print shell init code (eval in your shell rc file)')
            [CompletionResult]::new('hook', 'hook', [CompletionResultType]::ParameterValue, 'Internal: called by the cd hook to load/unload project aliases')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;help;add' {
            break
        }
        'am;help;profile' {
            break
        }
        'am;help;profiles' {
            break
        }
        'am;help;init' {
            break
        }
        'am;help;hook' {
            break
        }
        'am;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
