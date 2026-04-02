# am cd hook: track directory changes and reload project aliases
__am_hook() {
  local previous_exit_status=$?
  if [[ "${__am_prev_dir:-}" != "$PWD" ]]; then
    __am_prev_dir="$PWD"
    eval "$(command am hook __SHELL__)"
  fi
  return $previous_exit_status
}
if [[ ";${PROMPT_COMMAND[*]:-};" != *";__am_hook;"* ]]; then
  if [[ "$(declare -p PROMPT_COMMAND 2>&1)" == "declare -a"* ]]; then
    PROMPT_COMMAND=(__am_hook "${PROMPT_COMMAND[@]}")
  else
    PROMPT_COMMAND="__am_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
  fi
fi
__am_hook
