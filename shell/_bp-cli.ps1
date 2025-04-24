
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'bp-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'bp-cli'
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
        'bp-cli' {
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Remote address of the RGB node to connect to')
            [CompletionResult]::new('--remote', '--remote', [CompletionResultType]::ParameterName, 'Remote address of the RGB node to connect to')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('ping', 'ping', [CompletionResultType]::ParameterValue, 'ping')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'bp-cli;ping' {
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'bp-cli;help' {
            [CompletionResult]::new('ping', 'ping', [CompletionResultType]::ParameterValue, 'ping')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'bp-cli;help;ping' {
            break
        }
        'bp-cli;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
