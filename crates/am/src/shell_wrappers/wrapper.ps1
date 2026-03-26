# am wrapper: reload aliases after mutations
function am {
    $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
    & $amBin @args
    if ($LASTEXITCODE -ne 0) { return }
    # tui — always reload
    if ($args.Count -ge 1 -and $args[0] -in 'tui', 't') {
        Invoke-Expression (& $amBin reload __SHELL__)
        Invoke-Expression (& $amBin hook __SHELL__)
        return
    }
    # profile mutation — reload
    if ($args.Count -ge 1 -and $args[0] -in 'profile', 'p') {
        if ($args.Count -ge 2 -and $args[1] -in 'set', 's', 'use', 'u', 'add', 'a', 'remove', 'r') {
            Invoke-Expression (& $amBin reload __SHELL__)
        }
    }
    # alias mutation — reload
    elseif ($args.Count -ge 1 -and $args[0] -in 'add', 'a', 'remove', 'r') {
        if ($args -contains '-l' -or $args -contains '--local') {
            Invoke-Expression (& $amBin hook __SHELL__)
        } else {
            Invoke-Expression (& $amBin reload __SHELL__)
        }
    }
}