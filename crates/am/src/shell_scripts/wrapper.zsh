am() {
  command am "$@"
  case "$1:$2" in
    profile:set|p:set|profile:s|p:s) eval "$(command am reload __SHELL__)" ;;
  esac
  case "$1" in
    add|a|remove|r)
      case "$*" in
        *\ -l\ *|*\ --local\ *|*\ -l|*\ --local) eval "$(command am hook __SHELL__)" ;;
        *) eval "$(command am reload __SHELL__)" ;;
      esac ;;
  esac
}