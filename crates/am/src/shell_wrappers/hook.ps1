# am cd hook: track directory changes and reload project aliases
$env:__AM_LAST_DIR = $PWD.Path
$__am_original_prompt = $function:prompt
function global:prompt {
    if ($PWD.Path -ne $env:__AM_LAST_DIR) {
        $env:__AM_LAST_DIR = $PWD.Path
        $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
        $hookLines = & $amBin hook __SHELL__
        foreach ($line in $hookLines) {
            if ($line.Trim()) { Invoke-Expression $line }
        }
    }
    & $__am_original_prompt
}
