am() {
  command am "$@"
  case "$1:$2" in
    profile:set|p:set|profile:s|p:s|profile:add|p:add|profile:a|p:a|profile:remove|p:remove|profile:r|p:r) eval "$(command am reload __SHELL__)" ;;
  esac
  case "$1" in
    add|a|remove|r)
      case "$*" in
        *\ -l\ *|*\ --local\ *|*\ -l|*\ --local) eval "$(command am hook __SHELL__)" ;;
        *) eval "$(command am reload __SHELL__)" ;;
      esac ;;
  esac
}