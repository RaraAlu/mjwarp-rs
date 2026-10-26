# QA pre-fix fact check: case_evidence on the r2 parent copy (GNU default + MSVC).
# Confirms independently: host-gnu pre was GREEN by coupling (3.838112e-7) and host-msvc
# pre was RED (6.351978e-7).
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$work = Join-Path $repo 'target\qa-a-check'
$logs = Join-Path $repo 'docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\logs'
$tree = Join-Path $work 'pre-tree'
New-Item -ItemType Directory -Force -Path $logs | Out-Null
$exits = [System.Collections.Generic.List[string]]::new()

function Run-Pre([string]$mode, [string]$toolchain, [string]$targetDir) {
    $env:CARGO_TARGET_DIR = $targetDir
    $prefix = @()
    if ($toolchain -ne '') { $prefix = @($toolchain) }
    Push-Location $tree
    $caseArgs = $prefix + @('test', '--locked', '--lib',
        'physics::collision::gjk::case_evidence', '--', '--nocapture')
    & cargo @caseArgs *> (Join-Path $logs "pre-fix-case-evidence-$mode-qa.log")
    $exitPrint = $LASTEXITCODE
    $assertArgs = $prefix + @('test', '--locked', '--lib',
        'physics::collision::gjk::case_evidence::upstream_f32_within_4ulp', '--', '--exact')
    & cargo @assertArgs *> (Join-Path $logs "pre-fix-case-evidence-$mode-assert-qa.log")
    $exitAssert = $LASTEXITCODE
    Pop-Location
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    $script:exits.Add("pre-$mode print=$exitPrint assert=$exitAssert")
    Write-Host "pre-$mode print=$exitPrint assert=$exitAssert"
}

Run-Pre 'gnu-default' '' (Join-Path $work 'target-pre-fact-gnu')
Run-Pre 'msvc' '+1.99.0-x86_64-pc-windows-msvc' (Join-Path $work 'target-pre-fact-msvc')
$exits | Out-File -Encoding utf8 (Join-Path $logs 'pre-factcheck-exits.txt')
Write-Host 'QA_A_PRE_FACTCHECK_DONE'
