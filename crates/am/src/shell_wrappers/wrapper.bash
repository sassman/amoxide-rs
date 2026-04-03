am() {
  command am "$@"
  local am_status=$?
  if [[ $am_status -ne 0 ]]; then return $am_status; fi
  case "$1" in
    tui|t) eval "$(command am reload __SHELL__)"; eval "$(command am hook __SHELL__)"; return ;;
  esac
  case "$1:$2" in
    profile:use|p:use|profile:u|p:u|profile:add|p:add|profile:a|p:a|profile:remove|p:remove|profile:r|p:r) eval "$(command am reload __SHELL__)" ;;
  esac
  case "$1" in
    add|a|remove|r)
      case "$*" in
        *\ -l\ *|*\ --local\ *|*\ -l|*\ --local) eval "$(command am hook __SHELL__)" ;;
        *) eval "$(command am reload __SHELL__)" ;;
      esac ;;
  esac
}
