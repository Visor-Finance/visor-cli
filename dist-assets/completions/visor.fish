# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_visor_global_optspecs
    string join \n json s/space= debug api-url= auth-url= h/help V/version
end

function __fish_visor_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_visor_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_visor_using_subcommand
    set -l cmd (__fish_visor_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c visor -n "__fish_visor_needs_command" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_needs_command" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_needs_command" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_needs_command" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_needs_command" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_needs_command" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_needs_command" -s V -l version -d 'Print version'
complete -c visor -n "__fish_visor_needs_command" -f -a "auth" -d 'Log in, log out, and check the stored login'
complete -c visor -n "__fish_visor_needs_command" -f -a "whoami" -d 'Show who is logged in, how, and the space commands act on'
complete -c visor -n "__fish_visor_needs_command" -f -a "doctor" -d 'Check the CLI version, the login, and the connection to Visor'
complete -c visor -n "__fish_visor_needs_command" -f -a "open" -d 'Open Visor in the browser: the dashboard or a page of it'
complete -c visor -n "__fish_visor_needs_command" -f -a "me" -d 'Show the user the token belongs to'
complete -c visor -n "__fish_visor_needs_command" -f -a "spaces" -d 'List spaces and choose the default one'
complete -c visor -n "__fish_visor_needs_command" -f -a "transactions" -d 'List, record, edit, and delete transactions'
complete -c visor -n "__fish_visor_needs_command" -f -a "tx" -d 'List, record, edit, and delete transactions'
complete -c visor -n "__fish_visor_needs_command" -f -a "accounts" -d 'List accounts'
complete -c visor -n "__fish_visor_needs_command" -f -a "categories" -d 'List categories'
complete -c visor -n "__fish_visor_needs_command" -f -a "tags" -d 'List tags'
complete -c visor -n "__fish_visor_needs_command" -f -a "cards" -d 'List credit cards and their bills'
complete -c visor -n "__fish_visor_needs_command" -f -a "subscriptions" -d 'List subscriptions'
complete -c visor -n "__fish_visor_needs_command" -f -a "fixed-bills" -d 'List fixed bills, such as rent or utilities'
complete -c visor -n "__fish_visor_needs_command" -f -a "installments" -d 'List installment plans'
complete -c visor -n "__fish_visor_needs_command" -f -a "goals" -d 'List goals'
complete -c visor -n "__fish_visor_needs_command" -f -a "investments" -d 'List investments'
complete -c visor -n "__fish_visor_needs_command" -f -a "connections" -d 'List Open Finance connections and their sync status'
complete -c visor -n "__fish_visor_needs_command" -f -a "overview" -d 'Show balances, the month so far, and the projection for a space'
complete -c visor -n "__fish_visor_needs_command" -f -a "pix" -d 'Send a Pix that you confirm with biometrics at your bank (Premium)'
complete -c visor -n "__fish_visor_needs_command" -f -a "notifications" -d 'List notifications'
complete -c visor -n "__fish_visor_needs_command" -f -a "completions" -d 'Print a shell completion script'
complete -c visor -n "__fish_visor_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "login" -d 'Log in through the browser, or with a personal API token'
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "logout" -d 'Remove the stored login'
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "status" -d 'Show which token is in use and whom it belongs to'
complete -c visor -n "__fish_visor_using_subcommand auth; and not __fish_seen_subcommand_from login logout status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -l with-token -d 'Use a personal API token, read from standard input or a hidden prompt'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -l no-browser -d 'Print the login page URL instead of opening a browser'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from login" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from logout" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from logout" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from logout" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from logout" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from logout" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from logout" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from status" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from status" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from status" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from status" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from status" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "login" -d 'Log in through the browser, or with a personal API token'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "logout" -d 'Remove the stored login'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show which token is in use and whom it belongs to'
complete -c visor -n "__fish_visor_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand whoami" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand whoami" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand whoami" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand whoami" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand whoami" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand whoami" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand doctor" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand doctor" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand doctor" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand doctor" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand doctor" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand open" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand open" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand open" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand open" -l print -d 'Print the URL instead of opening it'
complete -c visor -n "__fish_visor_using_subcommand open" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand open" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand open" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand me" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand me" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand me" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand me" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand me" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand me" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -f -a "list" -d 'List the spaces you can access'
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -f -a "ls" -d 'List the spaces you can access'
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -f -a "use" -d 'Set the space other commands act on'
complete -c visor -n "__fish_visor_using_subcommand spaces; and not __fish_seen_subcommand_from list ls use help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from use" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from use" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from use" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from use" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from use" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "list" -d 'List the spaces you can access'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "use" -d 'Set the space other commands act on'
complete -c visor -n "__fish_visor_using_subcommand spaces; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "list" -d 'List transactions, newest first'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "ls" -d 'List transactions, newest first'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "get" -d 'Show one transaction'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "add" -d 'Record a manual transaction'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "edit" -d 'Edit a transaction; bank-owned fields of a connected one cannot change'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "delete" -d 'Delete a transaction'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "rm" -d 'Delete a transaction'
complete -c visor -n "__fish_visor_using_subcommand transactions; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l from -d 'First day to include, YYYY-MM-DD (Brasília time)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l to -d 'Last day to include, YYYY-MM-DD (Brasília time) [default: today]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l account -d 'Account name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l tag -d 'Tag name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l type -d 'Only debits or only credits' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l ignored -d 'Only ignored (true) or only counted (false) transactions' -r -f -a "true\t''
false\t''"
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l search -d 'Text in the description, merchant, or notes' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l limit -d 'Maximum number of results; the CLI fetches as many pages as it needs' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l after -d 'Continue after the cursor that a previous list printed' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l csv -d 'Print CSV for a spreadsheet: one row per transaction, names instead of IDs'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l all -d 'Fetch every result, however many; each page counts toward the rate limit'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l from -d 'First day to include, YYYY-MM-DD (Brasília time)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l to -d 'Last day to include, YYYY-MM-DD (Brasília time) [default: today]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l account -d 'Account name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l tag -d 'Tag name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l type -d 'Only debits or only credits' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l ignored -d 'Only ignored (true) or only counted (false) transactions' -r -f -a "true\t''
false\t''"
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l search -d 'Text in the description, merchant, or notes' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l limit -d 'Maximum number of results; the CLI fetches as many pages as it needs' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l after -d 'Continue after the cursor that a previous list printed' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l csv -d 'Print CSV for a spreadsheet: one row per transaction, names instead of IDs'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l all -d 'Fetch every result, however many; each page counts toward the rate limit'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from get" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from get" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from get" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from get" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from get" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l amount -d 'Positive amount, such as 42.90 or 42,90' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -s d -l description -d 'What it was, as it should appear in Visor' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -s c -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -s a -l account -d 'Account name or ID; the transaction takes its currency' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l type -d 'Debit for money out, credit for money in' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l at -d 'When it happened: YYYY-MM-DD, an RFC 3339 time, or `now`' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l merchant -d 'Merchant name' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l notes -d 'Free-form notes' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l tag -d 'Tag name or ID; repeat for several' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l idempotency-key -d 'Reuse a key to retry without recording twice [default: a new UUID]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l ignored -d 'Leave it out of totals'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l amount -d 'Manual transactions only' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -s d -l description -d 'Manual transactions only' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -s c -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l type -d 'Manual transactions only' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l at -d 'Manual transactions only: YYYY-MM-DD, an RFC 3339 time, or `now`' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l notes -d 'New notes; an empty string removes them' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l tag -d 'Replace every tag; repeat for several' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l ignored -d 'Leave it out of totals (true) or count it again (false)' -r -f -a "true\t''
false\t''"
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l clear-tags -d 'Remove every tag'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from delete" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from delete" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from delete" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from delete" -s y -l yes -d 'Do not ask for confirmation'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from delete" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from delete" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from rm" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from rm" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from rm" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from rm" -s y -l yes -d 'Do not ask for confirmation'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from rm" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from rm" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from rm" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from help" -f -a "list" -d 'List transactions, newest first'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from help" -f -a "get" -d 'Show one transaction'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from help" -f -a "add" -d 'Record a manual transaction'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit a transaction; bank-owned fields of a connected one cannot change'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from help" -f -a "delete" -d 'Delete a transaction'
complete -c visor -n "__fish_visor_using_subcommand transactions; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "list" -d 'List transactions, newest first'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "ls" -d 'List transactions, newest first'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "get" -d 'Show one transaction'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "add" -d 'Record a manual transaction'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "edit" -d 'Edit a transaction; bank-owned fields of a connected one cannot change'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "delete" -d 'Delete a transaction'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "rm" -d 'Delete a transaction'
complete -c visor -n "__fish_visor_using_subcommand tx; and not __fish_seen_subcommand_from list ls get add edit delete rm help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l from -d 'First day to include, YYYY-MM-DD (Brasília time)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l to -d 'Last day to include, YYYY-MM-DD (Brasília time) [default: today]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l account -d 'Account name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l tag -d 'Tag name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l type -d 'Only debits or only credits' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l ignored -d 'Only ignored (true) or only counted (false) transactions' -r -f -a "true\t''
false\t''"
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l search -d 'Text in the description, merchant, or notes' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l limit -d 'Maximum number of results; the CLI fetches as many pages as it needs' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l after -d 'Continue after the cursor that a previous list printed' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l csv -d 'Print CSV for a spreadsheet: one row per transaction, names instead of IDs'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l all -d 'Fetch every result, however many; each page counts toward the rate limit'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l from -d 'First day to include, YYYY-MM-DD (Brasília time)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l to -d 'Last day to include, YYYY-MM-DD (Brasília time) [default: today]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l account -d 'Account name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l tag -d 'Tag name or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l type -d 'Only debits or only credits' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l ignored -d 'Only ignored (true) or only counted (false) transactions' -r -f -a "true\t''
false\t''"
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l search -d 'Text in the description, merchant, or notes' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l limit -d 'Maximum number of results; the CLI fetches as many pages as it needs' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l after -d 'Continue after the cursor that a previous list printed' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l csv -d 'Print CSV for a spreadsheet: one row per transaction, names instead of IDs'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l all -d 'Fetch every result, however many; each page counts toward the rate limit'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from get" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from get" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from get" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from get" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from get" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l amount -d 'Positive amount, such as 42.90 or 42,90' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -s d -l description -d 'What it was, as it should appear in Visor' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -s c -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -s a -l account -d 'Account name or ID; the transaction takes its currency' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l type -d 'Debit for money out, credit for money in' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l at -d 'When it happened: YYYY-MM-DD, an RFC 3339 time, or `now`' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l merchant -d 'Merchant name' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l notes -d 'Free-form notes' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l tag -d 'Tag name or ID; repeat for several' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l idempotency-key -d 'Reuse a key to retry without recording twice [default: a new UUID]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l ignored -d 'Leave it out of totals'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l amount -d 'Manual transactions only' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -s d -l description -d 'Manual transactions only' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -s c -l category -d 'Category name, key, or ID' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l type -d 'Manual transactions only' -r -f -a "debit\t'Money out'
credit\t'Money in'"
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l at -d 'Manual transactions only: YYYY-MM-DD, an RFC 3339 time, or `now`' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l notes -d 'New notes; an empty string removes them' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l tag -d 'Replace every tag; repeat for several' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l ignored -d 'Leave it out of totals (true) or count it again (false)' -r -f -a "true\t''
false\t''"
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l clear-tags -d 'Remove every tag'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from edit" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from delete" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from delete" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from delete" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from delete" -s y -l yes -d 'Do not ask for confirmation'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from delete" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from delete" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from rm" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from rm" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from rm" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from rm" -s y -l yes -d 'Do not ask for confirmation'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from rm" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from rm" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from rm" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from help" -f -a "list" -d 'List transactions, newest first'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from help" -f -a "get" -d 'Show one transaction'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from help" -f -a "add" -d 'Record a manual transaction'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from help" -f -a "edit" -d 'Edit a transaction; bank-owned fields of a connected one cannot change'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from help" -f -a "delete" -d 'Delete a transaction'
complete -c visor -n "__fish_visor_using_subcommand tx; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand accounts; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from list" -l include-hidden -d 'Include the ones hidden in Visor'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from ls" -l include-hidden -d 'Include the ones hidden in Visor'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand accounts; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand categories; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from list" -l include-hidden -d 'Include the ones hidden in Visor'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from ls" -l include-hidden -d 'Include the ones hidden in Visor'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand categories; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand tags; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand tags; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -f -a "list" -d 'List credit cards with their limits and current bill'
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -f -a "ls" -d 'List credit cards with their limits and current bill'
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -f -a "bills" -d 'List the bills of a card'
complete -c visor -n "__fish_visor_using_subcommand cards; and not __fish_seen_subcommand_from list ls bills help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -l from -d 'First due date to include, YYYY-MM-DD [default: start of this month]' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -l to -d 'Last due date to include, YYYY-MM-DD [default: end of this month]' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from bills" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from help" -f -a "list" -d 'List credit cards with their limits and current bill'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from help" -f -a "bills" -d 'List the bills of a card'
complete -c visor -n "__fish_visor_using_subcommand cards; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand subscriptions; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand fixed-bills; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand installments; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from list" -l status -d 'Only plans with this status' -r -f -a "active\t''
completed\t''"
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from ls" -l status -d 'Only plans with this status' -r -f -a "active\t''
completed\t''"
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand installments; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand goals; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from list" -l status -d 'Only goals with this status' -r -f -a "active\t''
archived\t''
completed\t''"
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from ls" -l status -d 'Only goals with this status' -r -f -a "active\t''
archived\t''
completed\t''"
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand goals; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand investments; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand investments; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand connections; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand connections; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand overview" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand overview" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand overview" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand overview" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand overview" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand overview" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -s d -l description -d 'What the Pix is for, up to 140 characters' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l bank -d 'Bank the Pix comes from [default: the one bank you already linked]' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l key-type -d 'Key type, for a key that can be a CPF or a phone' -r -f -a "cpf\t''
cnpj\t''
phone\t''
email\t''
evp\t'A random key'"
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l idempotency-key -d 'Reuse a key to retry without starting a second Pix [default: a new UUID]' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l qr -d 'Confirm on your phone: show a QR code of the confirmation page'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l open -d 'Confirm on this computer: open the confirmation page in the browser'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l no-wait -d 'Print the Pix ID and the link without waiting for the Pix to finish'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l link-device -d 'Link the bank again on the device you confirm with, for one that says "Dispositivo não reconhecido"'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -a "status" -d 'Show the status of a Pix you started'
complete -c visor -n "__fish_visor_using_subcommand pix; and not __fish_seen_subcommand_from status help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from status" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from status" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from status" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from status" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from status" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show the status of a Pix you started'
complete -c visor -n "__fish_visor_using_subcommand pix; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -f -a "ls" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand notifications; and not __fish_seen_subcommand_from list ls help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l limit -d 'Maximum number of results; the CLI fetches as many pages as it needs' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l after -d 'Continue after the cursor that a previous list printed' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l unread -d 'Only unread notifications'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l all -d 'Fetch every result, however many; each page counts toward the rate limit'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l limit -d 'Maximum number of results; the CLI fetches as many pages as it needs' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l after -d 'Continue after the cursor that a previous list printed' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l unread -d 'Only unread notifications'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l all -d 'Fetch every result, however many; each page counts toward the rate limit'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from help" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand notifications; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand completions" -s s -l space -d 'Space to act on, by name or ID [default: the one set with `visor spaces use`]' -r
complete -c visor -n "__fish_visor_using_subcommand completions" -l api-url -d 'API base URL' -r
complete -c visor -n "__fish_visor_using_subcommand completions" -l auth-url -d 'Base URL of the browser login (OAuth authorization server)' -r
complete -c visor -n "__fish_visor_using_subcommand completions" -l json -d 'Print the API response body as JSON instead of a table'
complete -c visor -n "__fish_visor_using_subcommand completions" -l debug -d 'Print each HTTP request (method, URL, status, time, request ID) on standard error'
complete -c visor -n "__fish_visor_using_subcommand completions" -s h -l help -d 'Print help'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "auth" -d 'Log in, log out, and check the stored login'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "whoami" -d 'Show who is logged in, how, and the space commands act on'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "doctor" -d 'Check the CLI version, the login, and the connection to Visor'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "open" -d 'Open Visor in the browser: the dashboard or a page of it'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "me" -d 'Show the user the token belongs to'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "spaces" -d 'List spaces and choose the default one'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "transactions" -d 'List, record, edit, and delete transactions'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "accounts" -d 'List accounts'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "categories" -d 'List categories'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "tags" -d 'List tags'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "cards" -d 'List credit cards and their bills'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "subscriptions" -d 'List subscriptions'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "fixed-bills" -d 'List fixed bills, such as rent or utilities'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "installments" -d 'List installment plans'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "goals" -d 'List goals'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "investments" -d 'List investments'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "connections" -d 'List Open Finance connections and their sync status'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "overview" -d 'Show balances, the month so far, and the projection for a space'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "pix" -d 'Send a Pix that you confirm with biometrics at your bank (Premium)'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "notifications" -d 'List notifications'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "completions" -d 'Print a shell completion script'
complete -c visor -n "__fish_visor_using_subcommand help; and not __fish_seen_subcommand_from auth whoami doctor open me spaces transactions accounts categories tags cards subscriptions fixed-bills installments goals investments connections overview pix notifications completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "login" -d 'Log in through the browser, or with a personal API token'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "logout" -d 'Remove the stored login'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "status" -d 'Show which token is in use and whom it belongs to'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from spaces" -f -a "list" -d 'List the spaces you can access'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from spaces" -f -a "use" -d 'Set the space other commands act on'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from transactions" -f -a "list" -d 'List transactions, newest first'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from transactions" -f -a "get" -d 'Show one transaction'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from transactions" -f -a "add" -d 'Record a manual transaction'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from transactions" -f -a "edit" -d 'Edit a transaction; bank-owned fields of a connected one cannot change'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from transactions" -f -a "delete" -d 'Delete a transaction'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from accounts" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from categories" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from tags" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from cards" -f -a "list" -d 'List credit cards with their limits and current bill'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from cards" -f -a "bills" -d 'List the bills of a card'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from subscriptions" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from fixed-bills" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from installments" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from goals" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from investments" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from connections" -f -a "list" -d 'List them'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from pix" -f -a "status" -d 'Show the status of a Pix you started'
complete -c visor -n "__fish_visor_using_subcommand help; and __fish_seen_subcommand_from notifications" -f -a "list" -d 'List them'
