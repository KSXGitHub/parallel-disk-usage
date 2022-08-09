complete -c pdu -l bytes-format -d 'How to display the numbers of bytes' -r -f -a "{plain	Display the value as-is,metric	Display the value with a unit suffix in [metric scale](formatter::METRIC),binary	Display the value with a unit suffix in [binary scale](formatter::BINARY)}"
complete -c pdu -l quantity -d 'Aspect of the files/directories to be measured' -r -f -a "{len	Measure apparent sizes, equivalent to the [len](std::fs::Metadata::len) method,blksize	Measure block sizes, equivalent to the [blksize](std::os::unix::prelude::MetadataExt::blksize) method (POSIX only),blocks	Count numbers of blocks, equivalent to the [blocks](std::os::unix::prelude::MetadataExt::blocks) method (POSIX only)}"
complete -c pdu -l max-depth -d 'Maximum depth to display the data (must be greater than 0)' -r
complete -c pdu -l total-width -d 'Width of the visualization' -r
complete -c pdu -l column-width -d 'Maximum widths of the tree column and width of the bar column' -r
complete -c pdu -l min-ratio -d 'Minimal size proportion required to appear' -r
complete -c pdu -s h -l help -d 'Print help information'
complete -c pdu -l json-input -d 'Read JSON data from stdin'
complete -c pdu -l json-output -d 'Print JSON data instead of an ASCII chart'
complete -c pdu -l top-down -d 'Print the tree top-down instead of bottom-up'
complete -c pdu -l align-left -d 'Fill the bars from left to right'
complete -c pdu -l no-sort -d 'Preserve order of entries'
complete -c pdu -l silent-errors -d 'Prevent filesystem error messages from appearing in stderr'
complete -c pdu -l progress -d 'Report progress being made at the expense of performance'
