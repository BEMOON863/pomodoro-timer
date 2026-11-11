<#
.SYNOPSIS
    Post-edit verification for the aaa Rust project.

.DESCRIPTION
    One deterministic run of three steps:
      1. cargo fmt --all -- --check
      2. cargo check --all-targets
      3. cargo clippy --all-targets -- -D warnings

    Run it after every change to *.rs or Cargo.toml (AGENTS.md, rule 2).
    Exit code 0 means all clean; 1 means at least one step failed.

    Compatible with Windows PowerShell 5.1 and PowerShell 7+.
    Messages are intentionally ASCII so encoding cannot break them.

.PARAMETER SkipFmt
    Skip the formatting check.

.PARAMETER MaxErrorLines
    How many trailing output lines to show for a failed step. Default 60.

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\verify.ps1

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\verify.ps1 -SkipFmt
#>
[CmdletBinding()]
param(
    [switch]$SkipFmt,
    [int]$MaxErrorLines = 60
)

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path -LiteralPath (Join-Path $projectRoot 'Cargo.toml'))) {
    Write-Host ("Cargo.toml not found next to scripts: {0}" -f $projectRoot) -ForegroundColor Red
    exit 1
}

function Invoke-VerifyStep {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string[]]$Command
    )

    Write-Host ("==> {0}" -f $Name) -ForegroundColor Cyan

    $exe = $Command[0]
    $argv = @($Command | Select-Object -Skip 1)

    # Windows PowerShell 5.1 turns native stderr into a terminating
    # NativeCommandError when ErrorActionPreference is Stop, so relax it
    # locally and judge by the exit code instead.
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        $lines = @(& $exe @argv 2>&1 | ForEach-Object {
                $text = [string]$_
                # Windows PowerShell 5.1 wraps native stderr in an ErrorRecord
                # whose type name leaks into the text; drop that noise.
                if ($text -ne 'System.Management.Automation.RemoteException') { $text }
            })
        $code = $LASTEXITCODE
        if ($null -eq $code) { $code = 1 }
    }
    finally {
        $sw.Stop()
        $ErrorActionPreference = $previous
    }

    $seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1)
    $ok = ($code -eq 0)

    if ($ok) {
        Write-Host ("    PASS  ({0}s)" -f $seconds) -ForegroundColor Green
    }
    else {
        Write-Host ("    FAIL  exit={0} ({1}s)" -f $code, $seconds) -ForegroundColor Red
        $tail = $lines
        if ($lines.Count -gt $MaxErrorLines) {
            $tail = @($lines | Select-Object -Last $MaxErrorLines)
            Write-Host ("    ... showing last {0} of {1} lines" -f $MaxErrorLines, $lines.Count) -ForegroundColor DarkYellow
        }
        foreach ($line in $tail) {
            Write-Host ("    {0}" -f $line) -ForegroundColor DarkGray
        }
    }

    return [pscustomobject]@{
        Name    = $Name
        Ok      = $ok
        Code    = $code
        Seconds = $seconds
    }
}

Push-Location -LiteralPath $projectRoot
try {
    $steps = @()
    if (-not $SkipFmt) {
        $steps += , @{
            Name    = 'cargo fmt --check'
            Command = @('cargo', 'fmt', '--all', '--', '--check')
        }
    }
    $steps += , @{
        Name    = 'cargo check'
        Command = @('cargo', 'check', '--all-targets', '--message-format=short')
    }
    $steps += , @{
        Name    = 'cargo clippy'
        Command = @('cargo', 'clippy', '--all-targets', '--message-format=short', '--', '-D', 'warnings')
    }

    $total = [System.Diagnostics.Stopwatch]::StartNew()
    $results = @()
    foreach ($step in $steps) {
        $results += Invoke-VerifyStep -Name $step.Name -Command $step.Command
    }
    $total.Stop()

    $failed = @($results | Where-Object { -not $_.Ok })

    Write-Host ''
    Write-Host '-- summary --' -ForegroundColor Cyan
    foreach ($r in $results) {
        $mark = if ($r.Ok) { 'OK  ' } else { 'FAIL' }
        $color = if ($r.Ok) { 'Green' } else { 'Red' }
        Write-Host ("  {0}  {1}  ({2}s)" -f $mark, $r.Name, $r.Seconds) -ForegroundColor $color
    }
    Write-Host ("  total: {0:N1}s" -f $total.Elapsed.TotalSeconds) -ForegroundColor DarkGray

    if ($failed.Count -gt 0) {
        Write-Host ''
        Write-Host ("VERIFICATION FAILED: {0} of {1} steps failed. Fix before continuing (AGENTS.md, rule 2)." -f $failed.Count, $results.Count) -ForegroundColor Red
        exit 1
    }

    Write-Host ''
    Write-Host 'VERIFICATION PASSED.' -ForegroundColor Green
    exit 0
}
finally {
    Pop-Location
}
