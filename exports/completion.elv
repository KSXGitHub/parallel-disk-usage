
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
            cand --bytes-format 'How to display the numbers of bytes'
            cand --quantity 'Aspect of the files/directories to be measured'
            cand --max-depth 'Maximum depth to display the data (must be greater than 0)'
            cand --total-width 'Width of the visualization'
            cand --column-width 'Maximum widths of the tree column and width of the bar column'
            cand --min-ratio 'Minimal size proportion required to appear'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand --json-input 'Read JSON data from stdin'
            cand --json-output 'Print JSON data instead of an ASCII chart'
            cand --top-down 'Print the tree top-down instead of bottom-up'
            cand --align-left 'Fill the bars from left to right'
            cand --no-sort 'Preserve order of entries'
            cand --silent-errors 'Prevent filesystem error messages from appearing in stderr'
            cand --progress 'Report progress being made at the expense of performance'
        }
    ]
    $completions[$command]
}
