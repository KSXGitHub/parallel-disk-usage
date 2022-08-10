
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
            [CompletionResult]::new('--bytes-format', 'bytes-format', [CompletionResultType]::ParameterName, 'How to display the numbers of bytes')
            [CompletionResult]::new('--quantity', 'quantity', [CompletionResultType]::ParameterName, 'Aspect of the files/directories to be measured')
            [CompletionResult]::new('--max-depth', 'max-depth', [CompletionResultType]::ParameterName, 'Maximum depth to display the data (must be greater than 0)')
            [CompletionResult]::new('--total-width', 'total-width', [CompletionResultType]::ParameterName, 'Width of the visualization')
            [CompletionResult]::new('--column-width', 'column-width', [CompletionResultType]::ParameterName, 'Maximum widths of the tree column and width of the bar column')
            [CompletionResult]::new('--min-ratio', 'min-ratio', [CompletionResultType]::ParameterName, 'Minimal size proportion required to appear')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--json-input', 'json-input', [CompletionResultType]::ParameterName, 'Read JSON data from stdin')
            [CompletionResult]::new('--json-output', 'json-output', [CompletionResultType]::ParameterName, 'Print JSON data instead of an ASCII chart')
            [CompletionResult]::new('--top-down', 'top-down', [CompletionResultType]::ParameterName, 'Print the tree top-down instead of bottom-up')
            [CompletionResult]::new('--align-left', 'align-left', [CompletionResultType]::ParameterName, 'Fill the bars from left to right')
            [CompletionResult]::new('--no-sort', 'no-sort', [CompletionResultType]::ParameterName, 'Preserve order of entries')
            [CompletionResult]::new('--silent-errors', 'silent-errors', [CompletionResultType]::ParameterName, 'Prevent filesystem error messages from appearing in stderr')
            [CompletionResult]::new('--progress', 'progress', [CompletionResultType]::ParameterName, 'Report progress being made at the expense of performance')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
