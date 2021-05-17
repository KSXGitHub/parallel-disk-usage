complete -c pdu -n "__fish_use_subcommand" -l bytes-format -d 'How to display the numbers of bytes' -r -f -a "plain metric binary"
complete -c pdu -n "__fish_use_subcommand" -l quantity -d 'Aspect of the files/directories to be measured' -r -f -a "len blksize blocks"
complete -c pdu -n "__fish_use_subcommand" -l max-depth -d 'Maximum depth to display the data (must be greater than 0)'
complete -c pdu -n "__fish_use_subcommand" -l total-width -d 'Width of the visualization'
complete -c pdu -n "__fish_use_subcommand" -l column-width -d 'Maximum widths of the tree column and width of the bar column'
complete -c pdu -n "__fish_use_subcommand" -l minimal-ratio -d 'Minimal size proportion required to appear'
complete -c pdu -n "__fish_use_subcommand" -l top-down -d 'Print the tree top-down instead of bottom-up'
complete -c pdu -n "__fish_use_subcommand" -l no-sort -d 'Preserve order of entries'
complete -c pdu -n "__fish_use_subcommand" -l silent-errors -d 'Prevent filesystem error messages from appearing in stderr'
complete -c pdu -n "__fish_use_subcommand" -l progress -d 'Report progress being made at the expense of performance'
complete -c pdu -n "__fish_use_subcommand" -s h -l help -d 'Prints help information'
complete -c pdu -n "__fish_use_subcommand" -s V -l version -d 'Prints version information'
