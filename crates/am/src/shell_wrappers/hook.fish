# am cd hook
function __am_hook --on-variable PWD
    am sync __SHELL__ | source
end
__am_hook
