# am cd hook: track directory changes and reload project aliases
$env:__AM_LAST_DIR = $PWD.Path
$__am_original_prompt = $function:prompt
function global:prompt {
    if ($PWD.Path -ne $env:__AM_LAST_DIR) {
        $env:__AM_LAST_DIR = $PWD.Path
        $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
        $hookCode = (& $amBin hook __SHELL__) -join "`r`n"
        if ($hookCode) { Invoke-Command -ScriptBlock ([scriptblock]::Create($hookCode)) -NoNewScope }
    }
    if ($__am_original_prompt) { & $__am_original_prompt } else { "PS $($PWD.Path)> " }
}
