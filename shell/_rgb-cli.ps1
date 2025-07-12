
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'rgb-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'rgb-cli'
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
        'rgb-cli' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('--network', '--network', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Remote address of the RGB node to connect to')
            [CompletionResult]::new('--remote', '--remote', [CompletionResultType]::ParameterName, 'Remote address of the RGB node to connect to')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Get RGB node status information')
            [CompletionResult]::new('wallets', 'wallets', [CompletionResultType]::ParameterValue, 'List wallets known to the RGB node')
            [CompletionResult]::new('contracts', 'contracts', [CompletionResultType]::ParameterValue, 'List contracts known to the RGB node')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rgb-cli;status' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('--network', '--network', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'rgb-cli;wallets' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('--network', '--network', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'rgb-cli;contracts' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('--network', '--network', [CompletionResultType]::ParameterName, 'Bitcoin network')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Set a verbosity level')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'rgb-cli;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Get RGB node status information')
            [CompletionResult]::new('wallets', 'wallets', [CompletionResultType]::ParameterValue, 'List wallets known to the RGB node')
            [CompletionResult]::new('contracts', 'contracts', [CompletionResultType]::ParameterValue, 'List contracts known to the RGB node')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rgb-cli;help;status' {
            break
        }
        'rgb-cli;help;wallets' {
            break
        }
        'rgb-cli;help;contracts' {
            break
        }
        'rgb-cli;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
