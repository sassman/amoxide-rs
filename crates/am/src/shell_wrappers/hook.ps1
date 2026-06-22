# am cd hook: sync project aliases on directory change
$__am_original_prompt = $function:prompt
function global:prompt {
    if ($PWD.Path -ne $env:__AM_LAST_DIR) {
        $env:__AM_LAST_DIR = $PWD.Path
        $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
        $hookCode = (& $amBin sync __SHELL__) -join "`r`n"
        if ($hookCode) { Invoke-Command -ScriptBlock ([scriptblock]::Create($hookCode)) -NoNewScope }
    }
    if ($__am_original_prompt) { & $__am_original_prompt } else { "PS $($PWD.Path)> " }
}
# Immediately sync the current directory, matching bash/zsh/fish hook behavior.
& {
    $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
    if ($amBin) {
        if ($env:__AM_DEBUG -eq '1') { [Console]::Error.WriteLine("[am] hook: initial sync $($PWD.Path)") }
        $initCode = (& $amBin sync __SHELL__) -join "`r`n"
        if ($initCode) { Invoke-Command -ScriptBlock ([scriptblock]::Create($initCode)) -NoNewScope }
    }
}
$env:__AM_LAST_DIR = $PWD.Path
