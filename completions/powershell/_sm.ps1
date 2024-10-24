
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'sm' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'sm'
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
        'sm' {
            [CompletionResult]::new('--current-shell', '--current-shell', [CompletionResultType]::ParameterName, 'The current shell sm runing in')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new alias, path, or secret')
            [CompletionResult]::new('env', 'env', [CompletionResultType]::ParameterValue, 'Load environment variables into the current shell')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize the shell-manager for your shell, usually put `eval "$(sm init)"` in your shell rc file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'sm;add' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Add a new path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'Add a new secret')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'sm;add;alias' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('--directory', '--directory', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Long alias flag')
            [CompletionResult]::new('--long', '--long', [CompletionResultType]::ParameterName, 'Long alias flag')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'sm;add;path' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('--directory', '--directory', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'sm;add;secret' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('--directory', '--directory', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'sm;add;help' {
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Add a new path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'Add a new secret')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'sm;add;help;alias' {
            break
        }
        'sm;add;help;path' {
            break
        }
        'sm;add;help;secret' {
            break
        }
        'sm;add;help;help' {
            break
        }
        'sm;env' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'sm;init' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'sm;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new alias, path, or secret')
            [CompletionResult]::new('env', 'env', [CompletionResultType]::ParameterValue, 'Load environment variables into the current shell')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize the shell-manager for your shell, usually put `eval "$(sm init)"` in your shell rc file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'sm;help;add' {
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Add a new path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'Add a new secret')
            break
        }
        'sm;help;add;alias' {
            break
        }
        'sm;help;add;path' {
            break
        }
        'sm;help;add;secret' {
            break
        }
        'sm;help;env' {
            break
        }
        'sm;help;init' {
            break
        }
        'sm;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
