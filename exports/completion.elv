
use builtin;
use str;

set edit:completion:arg-completer[pdu] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'pdu'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'pdu'= {
            cand -b 'How to display the numbers of bytes'
            cand --bytes-format 'How to display the numbers of bytes'
            cand -q 'Aspect of the files/directories to be measured'
            cand --quantity 'Aspect of the files/directories to be measured'
            cand -d 'Maximum depth to display the data. Could be either "inf" or a positive integer'
            cand --max-depth 'Maximum depth to display the data. Could be either "inf" or a positive integer'
            cand --depth 'Maximum depth to display the data. Could be either "inf" or a positive integer'
            cand -w 'Width of the visualization'
            cand --total-width 'Width of the visualization'
            cand --width 'Width of the visualization'
            cand --column-width 'Maximum widths of the tree column and width of the bar column'
            cand -m 'Minimal size proportion required to appear'
            cand --min-ratio 'Minimal size proportion required to appear'
            cand --threads 'Set the maximum number of threads to spawn. Could be either "auto", "max", or a number'
            cand --json-input 'Read JSON data from stdin'
            cand --json-output 'Print JSON data instead of an ASCII chart'
            cand -H 'Detect and subtract the sizes of hardlinks from their parent directory totals'
            cand --deduplicate-hardlinks 'Detect and subtract the sizes of hardlinks from their parent directory totals'
            cand --detect-links 'Detect and subtract the sizes of hardlinks from their parent directory totals'
            cand --dedupe-links 'Detect and subtract the sizes of hardlinks from their parent directory totals'
            cand --top-down 'Print the tree top-down instead of bottom-up'
            cand --align-right 'Set the root of the bars to the right'
            cand --no-sort 'Do not sort the branches in the tree'
            cand -s 'Prevent filesystem error messages from appearing in stderr'
            cand --silent-errors 'Prevent filesystem error messages from appearing in stderr'
            cand --no-errors 'Prevent filesystem error messages from appearing in stderr'
            cand -p 'Report progress being made at the expense of performance'
            cand --progress 'Report progress being made at the expense of performance'
            cand --omit-json-shared-details 'Do not output `.shared.details` in the JSON output'
            cand --omit-json-shared-summary 'Do not output `.shared.summary` in the JSON output'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
