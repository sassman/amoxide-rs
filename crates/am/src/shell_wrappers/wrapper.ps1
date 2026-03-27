# am wrapper: reload aliases after mutations
function am {
    $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
    & $amBin @args
    if ($LASTEXITCODE -ne 0) { return }
    # tui — always reload
    if ($args.Count -ge 1 -and $args[0] -in 'tui', 't') {
        $out = & $amBin reload __SHELL__ | Out-String
        if ($out.Trim()) { Invoke-Expression $out }
        $out = & $amBin hook __SHELL__ | Out-String
        if ($out.Trim()) { Invoke-Expression $out }
        return
    }
    # profile mutation — reload
    if ($args.Count -ge 1 -and $args[0] -in 'profile', 'p') {
        if ($args.Count -ge 2 -and $args[1] -in 'set', 's', 'use', 'u', 'add', 'a', 'remove', 'r') {
            $out = & $amBin reload __SHELL__ | Out-String
            if ($out.Trim()) { Invoke-Expression $out }
        }
    }
    # alias mutation — reload
    elseif ($args.Count -ge 1 -and $args[0] -in 'add', 'a', 'remove', 'r') {
        if ($args -contains '-l' -or $args -contains '--local') {
            $out = & $amBin hook __SHELL__ | Out-String
            if ($out.Trim()) { Invoke-Expression $out }
        } else {
            $out = & $amBin reload __SHELL__ | Out-String
            if ($out.Trim()) { Invoke-Expression $out }
        }
    }
}
