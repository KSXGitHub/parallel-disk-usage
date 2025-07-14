complete -c pdu -l bytes-format -d 'How to display the numbers of bytes' -r -f -a "plain\t'Display plain number of bytes without units'
metric\t'Use metric scale, i.e. 1K = 1000B, 1M = 1000K, and so on'
binary\t'Use binary scale, i.e. 1K = 1024B, 1M = 1024K, and so on'"
complete -c pdu -l quantity -d 'Aspect of the files/directories to be measured' -r -f -a "apparent-size\t'Measure apparent sizes'
block-size\t'Measure block sizes (block-count * 512B)'
block-count\t'Count numbers of blocks'"
complete -c pdu -l max-depth -l depth -d 'Maximum depth to display the data (must be greater than 0)' -r
complete -c pdu -l total-width -l width -d 'Width of the visualization' -r
complete -c pdu -l column-width -d 'Maximum widths of the tree column and width of the bar column' -r
complete -c pdu -l min-ratio -d 'Minimal size proportion required to appear' -r
complete -c pdu -l threads -d 'Set the maximum number of threads to spawn. Could be either "auto", "max", or a number' -r
complete -c pdu -l json-input -d 'Read JSON data from stdin'
complete -c pdu -l json-output -d 'Print JSON data instead of an ASCII chart'
complete -c pdu -l deduplicate-hardlinks -l detect-links -l dedupe-links -d 'Detect duplicated hardlinks and remove their sizes from total'
complete -c pdu -l top-down -d 'Print the tree top-down instead of bottom-up'
complete -c pdu -l align-right -d 'Set the root of the bars to the right'
complete -c pdu -l no-sort -d 'Preserve order of entries'
complete -c pdu -l silent-errors -l no-errors -d 'Prevent filesystem error messages from appearing in stderr'
complete -c pdu -l progress -d 'Report progress being made at the expense of performance'
complete -c pdu -l omit-json-shared-details -d 'Do not output `.shared.details` in the JSON output'
complete -c pdu -l omit-json-shared-summary -d 'Do not output `.shared.summary` in the JSON output'
complete -c pdu -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pdu -s V -l version -d 'Print version'
