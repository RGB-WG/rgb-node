
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
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('contract', 'contract', [CompletionResultType]::ParameterValue, 'Contract management')
            [CompletionResult]::new('outpoint', 'outpoint', [CompletionResultType]::ParameterValue, 'Information on outpoints')
            [CompletionResult]::new('transfer', 'transfer', [CompletionResultType]::ParameterValue, 'Work with state transfers')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rgb-cli;contract' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all known contract ids')
            [CompletionResult]::new('register', 'register', [CompletionResultType]::ParameterValue, 'Add new contract to the node')
            [CompletionResult]::new('state', 'state', [CompletionResultType]::ParameterValue, 'Query contract state')
            [CompletionResult]::new('consignment', 'consignment', [CompletionResultType]::ParameterValue, 'Request contract consignment')
            [CompletionResult]::new('embed', 'embed', [CompletionResultType]::ParameterValue, 'Embed contract information into PSBT file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rgb-cli;contract;list' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;contract;register' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--force', 'force', [CompletionResultType]::ParameterName, 'Force importing of valid contract containing non-mined transactions')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;contract;state' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;contract;consignment' {
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Transition types to be always included into the consignment')
            [CompletionResult]::new('--node-type', 'node-type', [CompletionResultType]::ParameterName, 'Transition types to be always included into the consignment')
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;contract;embed' {
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Transition types to be always included into the consignment')
            [CompletionResult]::new('--node-type', 'node-type', [CompletionResultType]::ParameterName, 'Transition types to be always included into the consignment')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Output file to save the PSBT updated with state transition(s) information. If not given, the source PSBT file is overwritten')
            [CompletionResult]::new('--out', 'out', [CompletionResultType]::ParameterName, 'Output file to save the PSBT updated with state transition(s) information. If not given, the source PSBT file is overwritten')
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;contract;help' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;outpoint' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('state', 'state', [CompletionResultType]::ParameterValue, 'Return known outpoint state')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rgb-cli;outpoint;state' {
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Outpoints to return the state for')
            [CompletionResult]::new('--outpoint', 'outpoint', [CompletionResultType]::ParameterName, 'Outpoints to return the state for')
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;outpoint;help' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;transfer' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('compose', 'compose', [CompletionResultType]::ParameterValue, 'Build state transfer consignment draft')
            [CompletionResult]::new('combine', 'combine', [CompletionResultType]::ParameterValue, 'Update PSBT with the information from the state transition')
            [CompletionResult]::new('finalize', 'finalize', [CompletionResultType]::ParameterValue, 'Finalize and (optionally) send state transfer consignment to beneficiary')
            [CompletionResult]::new('consume', 'consume', [CompletionResultType]::ParameterValue, 'Validate incoming transfer consignment and consume it into the stash')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rgb-cli;transfer;compose' {
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Transition types to be always included into the state transfer')
            [CompletionResult]::new('--node-type', 'node-type', [CompletionResultType]::ParameterName, 'Transition types to be always included into the state transfer')
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;transfer;combine' {
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Output file to save the PSBT updated with state transition(s) information. If not given, the source PSBT file is overwritten')
            [CompletionResult]::new('--out', 'out', [CompletionResultType]::ParameterName, 'Output file to save the PSBT updated with state transition(s) information. If not given, the source PSBT file is overwritten')
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;transfer;finalize' {
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Bifrost server to send state transfer to')
            [CompletionResult]::new('--send', 'send', [CompletionResultType]::ParameterName, 'Bifrost server to send state transfer to')
            [CompletionResult]::new('-e', 'e', [CompletionResultType]::ParameterName, 'Beneficiary blinded TXO seal - or witness transaction output numbers containing allocations for the beneficiary')
            [CompletionResult]::new('--endseal', 'endseal', [CompletionResultType]::ParameterName, 'Beneficiary blinded TXO seal - or witness transaction output numbers containing allocations for the beneficiary')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Output file to save the PSBT updated with state transition(s) information. If not given, the source PSBT file is overwritten')
            [CompletionResult]::new('--out', 'out', [CompletionResultType]::ParameterName, 'Output file to save the PSBT updated with state transition(s) information. If not given, the source PSBT file is overwritten')
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;transfer;consume' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Consume even if the endpoint witness transaction is not yet mined')
            [CompletionResult]::new('--force', 'force', [CompletionResultType]::ParameterName, 'Consume even if the endpoint witness transaction is not yet mined')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;transfer;help' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'rgb-cli;help' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting daemon RPC interface')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('--chain', 'chain', [CompletionResultType]::ParameterName, 'Blockchain to use')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
