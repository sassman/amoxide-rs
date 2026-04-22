# am wrapper: sync after mutations
function am --wraps=am
    command am $argv
    set -l am_status $status
    if test $am_status -ne 0
        return $am_status
    end
    switch "$argv[1]"
        case add a remove r use u trust tui t
            command am sync __SHELL__ | source
        case untrust
            command am sync --quiet __SHELL__ | source
        case profile p
            switch "$argv[2]"
                case use u add a remove r
                    command am sync __SHELL__ | source
            end
    end
end
