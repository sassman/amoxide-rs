# am cd hook
__am_hook() { eval "$(am sync __SHELL__)"; }
chpwd_functions+=(__am_hook)
__am_hook
