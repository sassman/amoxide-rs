# am cd hook
function __am_hook --on-variable PWD
    am hook __SHELL__ | source
end
__am_hook