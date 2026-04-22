# am wrapper: sync after mutations
function am {
    $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
    & $amBin @args
    if ($LASTEXITCODE -ne 0) { return }
    if ($args.Count -lt 1) { return }
    $first = $args[0]
    $second = if ($args.Count -ge 2) { $args[1] } else { $null }

    $runSync = {
        $out = (& $amBin sync __SHELL__) -join "`r`n"
        if ($out) { Invoke-Command -ScriptBlock ([scriptblock]::Create($out)) -NoNewScope }
    }
    $runSyncQuiet = {
        $out = (& $amBin sync --quiet __SHELL__) -join "`r`n"
        if ($out) { Invoke-Command -ScriptBlock ([scriptblock]::Create($out)) -NoNewScope }
    }

    if ($first -in 'add', 'a', 'remove', 'r', 'use', 'u', 'trust', 'tui', 't') {
        & $runSync
    } elseif ($first -eq 'untrust') {
        & $runSyncQuiet
    } elseif ($first -in 'profile', 'p') {
        if ($second -in 'use', 'u', 'add', 'a', 'remove', 'r') { & $runSync }
    }
}
