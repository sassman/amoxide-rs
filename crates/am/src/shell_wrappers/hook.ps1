# am cd hook: track directory changes and reload project aliases
$env:__AM_LAST_DIR = $PWD.Path
$__am_original_prompt = $function:prompt
$__am_hook_file = Join-Path $env:TEMP "am-hook.ps1"
function global:prompt {
    if ($PWD.Path -ne $env:__AM_LAST_DIR) {
        $env:__AM_LAST_DIR = $PWD.Path
        $amBin = (Get-Command -CommandType Application am | Select-Object -First 1).Source
        & $amBin hook __SHELL__ | Set-Content -Path $__am_hook_file -Encoding UTF8
        if ((Get-Item $__am_hook_file -ErrorAction SilentlyContinue).Length -gt 0) {
            . $__am_hook_file
        }
    }
    if ($__am_original_prompt) { & $__am_original_prompt } else { "PS $($PWD.Path)> " }
}
