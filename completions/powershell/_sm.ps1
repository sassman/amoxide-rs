
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
            [CompletionResult]::new('--current-shell', '--current-shell', [CompletionResultType]::ParameterName, 'The current shell am runing in')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new alias, path, or secret')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Imports all alias provided via stdin, e.g. `alias -L | am import alias`')
            [CompletionResult]::new('env', 'env', [CompletionResultType]::ParameterValue, 'Load environment variables into the current shell')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize the alias-manager for your shell, usually put `eval "$(am init)"` in your shell rc file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;add' {
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
        'am;add;alias' {
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
        'am;add;path' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('--directory', '--directory', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;add;secret' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('--directory', '--directory', [CompletionResultType]::ParameterName, 'Directory-specific flag')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;add;help' {
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Add a new path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'Add a new secret')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;add;help;alias' {
            break
        }
        'am;add;help;path' {
            break
        }
        'am;add;help;secret' {
            break
        }
        'am;add;help;help' {
            break
        }
        'am;import' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'secret')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;import;alias' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;import;path' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;import;secret' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'am;import;help' {
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'secret')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;import;help;alias' {
            break
        }
        'am;import;help;path' {
            break
        }
        'am;import;help;secret' {
            break
        }
        'am;import;help;help' {
            break
        }
        'am;env' {
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
        'am;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new alias, path, or secret')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Imports all alias provided via stdin, e.g. `alias -L | am import alias`')
            [CompletionResult]::new('env', 'env', [CompletionResultType]::ParameterValue, 'Load environment variables into the current shell')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Initialize the alias-manager for your shell, usually put `eval "$(am init)"` in your shell rc file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'am;help;add' {
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'Add a new alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Add a new path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'Add a new secret')
            break
        }
        'am;help;add;alias' {
            break
        }
        'am;help;add;path' {
            break
        }
        'am;help;add;secret' {
            break
        }
        'am;help;import' {
            [CompletionResult]::new('alias', 'alias', [CompletionResultType]::ParameterValue, 'alias')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'path')
            [CompletionResult]::new('secret', 'secret', [CompletionResultType]::ParameterValue, 'secret')
            break
        }
        'am;help;import;alias' {
            break
        }
        'am;help;import;path' {
            break
        }
        'am;help;import;secret' {
            break
        }
        'am;help;env' {
            break
        }
        'am;help;init' {
            break
        }
        'am;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
