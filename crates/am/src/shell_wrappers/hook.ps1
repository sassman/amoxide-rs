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
# Clear inherited alias tracking so the immediate sync always re-loads correctly.
# Stale _AM_ALIASES from a parent process or a previous `am sync` in this session
# would otherwise make `am sync` think all aliases are already defined, causing it
# to output nothing and leaving the shell with no aliases.
Remove-Item -ErrorAction SilentlyContinue Env:_AM_ALIASES
Remove-Item -ErrorAction SilentlyContinue Env:_AM_PROJECT_PATH
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
