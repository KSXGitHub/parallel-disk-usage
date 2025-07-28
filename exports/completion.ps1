
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'pdu' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'pdu'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'pdu' {
            [CompletionResult]::new('-b', '-b', [CompletionResultType]::ParameterName, 'How to display the numbers of bytes')
            [CompletionResult]::new('--bytes-format', '--bytes-format', [CompletionResultType]::ParameterName, 'How to display the numbers of bytes')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Aspect of the files/directories to be measured')
            [CompletionResult]::new('--quantity', '--quantity', [CompletionResultType]::ParameterName, 'Aspect of the files/directories to be measured')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Maximum depth to display the data. Could be either "inf" or a positive integer')
            [CompletionResult]::new('--max-depth', '--max-depth', [CompletionResultType]::ParameterName, 'Maximum depth to display the data. Could be either "inf" or a positive integer')
            [CompletionResult]::new('--depth', '--depth', [CompletionResultType]::ParameterName, 'Maximum depth to display the data. Could be either "inf" or a positive integer')
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Width of the visualization')
            [CompletionResult]::new('--total-width', '--total-width', [CompletionResultType]::ParameterName, 'Width of the visualization')
            [CompletionResult]::new('--width', '--width', [CompletionResultType]::ParameterName, 'Width of the visualization')
            [CompletionResult]::new('--column-width', '--column-width', [CompletionResultType]::ParameterName, 'Maximum widths of the tree column and width of the bar column')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'Minimal size proportion required to appear')
            [CompletionResult]::new('--min-ratio', '--min-ratio', [CompletionResultType]::ParameterName, 'Minimal size proportion required to appear')
            [CompletionResult]::new('--threads', '--threads', [CompletionResultType]::ParameterName, 'Set the maximum number of threads to spawn. Could be either "auto", "max", or a positive integer')
            [CompletionResult]::new('--json-input', '--json-input', [CompletionResultType]::ParameterName, 'Read JSON data from stdin')
            [CompletionResult]::new('--json-output', '--json-output', [CompletionResultType]::ParameterName, 'Print JSON data instead of an ASCII chart')
            [CompletionResult]::new('-H', '-H ', [CompletionResultType]::ParameterName, 'Detect and subtract the sizes of hardlinks from their parent directory totals')
            [CompletionResult]::new('--deduplicate-hardlinks', '--deduplicate-hardlinks', [CompletionResultType]::ParameterName, 'Detect and subtract the sizes of hardlinks from their parent directory totals')
            [CompletionResult]::new('--detect-links', '--detect-links', [CompletionResultType]::ParameterName, 'Detect and subtract the sizes of hardlinks from their parent directory totals')
            [CompletionResult]::new('--dedupe-links', '--dedupe-links', [CompletionResultType]::ParameterName, 'Detect and subtract the sizes of hardlinks from their parent directory totals')
            [CompletionResult]::new('--top-down', '--top-down', [CompletionResultType]::ParameterName, 'Print the tree top-down instead of bottom-up')
            [CompletionResult]::new('--align-right', '--align-right', [CompletionResultType]::ParameterName, 'Set the root of the bars to the right')
            [CompletionResult]::new('--no-sort', '--no-sort', [CompletionResultType]::ParameterName, 'Do not sort the branches in the tree')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Prevent filesystem error messages from appearing in stderr')
            [CompletionResult]::new('--silent-errors', '--silent-errors', [CompletionResultType]::ParameterName, 'Prevent filesystem error messages from appearing in stderr')
            [CompletionResult]::new('--no-errors', '--no-errors', [CompletionResultType]::ParameterName, 'Prevent filesystem error messages from appearing in stderr')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Report progress being made at the expense of performance')
            [CompletionResult]::new('--progress', '--progress', [CompletionResultType]::ParameterName, 'Report progress being made at the expense of performance')
            [CompletionResult]::new('--omit-json-shared-details', '--omit-json-shared-details', [CompletionResultType]::ParameterName, 'Do not output `.shared.details` in the JSON output')
            [CompletionResult]::new('--omit-json-shared-summary', '--omit-json-shared-summary', [CompletionResultType]::ParameterName, 'Do not output `.shared.summary` in the JSON output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
