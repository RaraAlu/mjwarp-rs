# QA independent G0b review: run all QA harness steps and log outputs (CPU only, no GPU).
[CmdletBinding()]
param(
    [string]$Root = 'D:\Rust\mjwarp-rs'
)
$ErrorActionPreference = 'Stop'
Set-Location $Root
$qa = Join-Path $Root 'target/qa-g0b'
$log = Join-Path $qa 'logs'
New-Item -ItemType Directory -Force $log | Out-Null
$python = Join-Path $Root 'target/mjwarp-parallel/venv/Scripts/python.exe'
$stamp = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ssK')
"QA G0b independent review run log; start=$stamp; host=$env:COMPUTERNAME; rustc=n/a" |
    Out-File -Encoding utf8 (Join-Path $log '00-header.txt')

& (Join-Path $qa 'build_qa_312.ps1') 2>&1 | Out-File -Encoding utf8 (Join-Path $log '01-build-312.log')

$ErrorActionPreference = 'Continue'
$steps = @(
    @{ name = '02-dump-315.log'; args = @('-B', 'target/qa-g0b/qa_dump_315.py') },
    @{ name = '03-parity-compare.log'; args = @('-B', 'target/qa-g0b/qa_parity_compare.py') },
    @{ name = '04-audit-existing.log'; args = @('-B', 'target/qa-g0b/qa_audit_existing.py') },
    @{ name = '05-verify-references.log'; args = @('-B', 'target/qa-g0b/qa_verify_references.py') },
    @{ name = '06-recapture.log'; args = @('-B', 'target/qa-g0b/qa_recapture.py') },
    @{ name = '07-recapture-diff.log'; args = @('-B', 'target/qa-g0b/qa_recapture_diff.py') },
    @{ name = '08-evidence-hash.log'; args = @('-B', 'target/qa-g0b/qa_evidence_hash.py') }
)
foreach ($step in $steps) {
    $out = Join-Path $log $step.name
    "=== $($step.args[-1]) $(Get-Date -Format o) ===" | Out-File -Encoding utf8 $out
    & $python @($step.args) *>&1 | Out-File -Encoding utf8 -Append $out
    "=== exit=$LASTEXITCODE ===" | Out-File -Encoding utf8 -Append $out
}
"done $(Get-Date -Format o)" | Out-File -Encoding utf8 -Append (Join-Path $log '00-header.txt')
