# ibron shell integration for PowerShell (Windows PowerShell 5.1+ and PowerShell 7+)
#
# Emits OSC 133 A/B/C/D semantic-prompt markers so ibron can identify
# command blocks, plus OSC 7 for the current working directory.
#
# Bypasses:
#   $env:IBRON_SHELL_SKIP_ALL = "1"            # disable everything
#   $env:IBRON_SHELL_SKIP_SEMANTIC_ZONES = "1" # disable OSC 133 only
#   $env:IBRON_SHELL_SKIP_CWD = "1"            # disable OSC 7 only
#
# Install: add the following line to $PROFILE:
#   . "$env:APPDATA\ibron\shell-integration\ibron.ps1"

if ($env:IBRON_SHELL_SKIP_ALL -eq "1") { return }
if (-not [Environment]::UserInteractive) { return }

# Track whether a command actually ran between two prompts so we don't emit
# a spurious D;0 on the very first prompt of the session.
$global:__ibron_CommandHasRun = $false
$global:__ibron_LastExitCode = 0

function global:__ibron_EmitA {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    [Console]::Write("`e]133;A`e\")
}

function global:__ibron_EmitB {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    [Console]::Write("`e]133;B`e\")
}

function global:__ibron_EmitC {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    [Console]::Write("`e]133;C`e\")
}

function global:__ibron_EmitD {
    if ($env:IBRON_SHELL_SKIP_SEMANTIC_ZONES -eq "1") { return }
    if (-not $global:__ibron_CommandHasRun) { return }
    [Console]::Write("`e]133;D;$($global:__ibron_LastExitCode)`e\")
    $global:__ibron_CommandHasRun = $false
}

function global:__ibron_EmitOsc7 {
    if ($env:IBRON_SHELL_SKIP_CWD -eq "1") { return }
    $cwd = (Get-Location).ProviderPath
    if (-not $cwd) { return }
    # OSC 7 is file://<host><path>. We use an empty host, which is legal and
    # matches how iTerm2 / ghostty emit it when no hostname is configured.
    $encoded = [Uri]::EscapeUriString($cwd -replace '\\', '/')
    [Console]::Write("`e]7;file://$encoded`e\")
}

# Wrap the user's existing prompt function so we compose with whatever they
# have customized. We capture it once at install time.
if (-not (Test-Path function:global:__ibron_OriginalPrompt)) {
    if (Test-Path function:prompt) {
        ${function:global:__ibron_OriginalPrompt} = ${function:prompt}
    } else {
        function global:__ibron_OriginalPrompt { "PS $(Get-Location)> " }
    }
}

function global:prompt {
    $exit = $LASTEXITCODE
    if ($null -ne $exit) { $global:__ibron_LastExitCode = $exit }
    __ibron_EmitD
    __ibron_EmitA
    __ibron_EmitOsc7
    $text = & __ibron_OriginalPrompt
    __ibron_EmitB
    return $text
}

# PSReadLine integration: emit C right when the user submits a line.
if (Get-Module -ListAvailable -Name PSReadLine) {
    Import-Module PSReadLine -ErrorAction SilentlyContinue
    if (Get-Module PSReadLine) {
        Set-PSReadLineKeyHandler -Key Enter -ScriptBlock {
            [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
            __ibron_EmitC
            $global:__ibron_CommandHasRun = $true
        }
    }
}
