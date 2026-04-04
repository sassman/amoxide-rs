# am wrapper: reload aliases after mutations
function am --wraps=am
    command am $argv
    set -l am_status $status
    if test $am_status -ne 0
        return $am_status
    end
    # tui may have changed anything → always reload after
    if begin; test "$argv[1]" = tui; or test "$argv[1]" = t; end
        command am reload __SHELL__ | source
        command am hook __SHELL__ | source
        return
    end
    # top-level use → reload aliases
    if begin; test "$argv[1]" = use; or test "$argv[1]" = u; end
        command am reload __SHELL__ | source
        return
    end
    # profile mutation → reload aliases
    if begin; test "$argv[1]" = profile; or test "$argv[1]" = p; end
        if begin; test "$argv[2]" = use; or test "$argv[2]" = u; or test "$argv[2]" = add; or test "$argv[2]" = a; or test "$argv[2]" = remove; or test "$argv[2]" = r; end
            command am reload __SHELL__ | source
        end
    else if begin; test "$argv[1]" = add; or test "$argv[1]" = a; or test "$argv[1]" = remove; or test "$argv[1]" = r; end
        if contains -- -l $argv; or contains -- --local $argv
            # local alias change → reload project aliases
            command am hook __SHELL__ | source
        else
            # profile/global alias change → reload
            command am reload __SHELL__ | source
        end
    end
end