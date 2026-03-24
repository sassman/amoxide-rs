# am cd hook
__am_hook() { eval "$(am hook __SHELL__)"; }
chpwd_functions+=(__am_hook)
__am_hook