# am cd hook: track directory changes and reload project aliases
$env:__AM_LAST_DIR = $PWD.Path
$__am_original_prompt = $function:prompt
function prompt {
    if ($PWD.Path -ne $env:__AM_LAST_DIR) {
        $env:__AM_LAST_DIR = $PWD.Path
        $hookOutput = (& am hook __SHELL__) -join "`n"
        if ($hookOutput) { Invoke-Expression $hookOutput }
    }
    & $__am_original_prompt
}
