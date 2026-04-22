am() {
  command am "$@"
  local am_status=$?
  if [[ $am_status -ne 0 ]]; then return $am_status; fi
  case "$1" in
    add|a|remove|r|use|u|trust|tui|t)
      eval "$(command am sync __SHELL__)" ;;
    untrust)
      eval "$(command am sync --quiet __SHELL__)" ;;
    profile|p)
      case "$2" in
        use|u|add|a|remove|r) eval "$(command am sync __SHELL__)" ;;
      esac ;;
  esac
}
