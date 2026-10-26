# Option-A fix QA: three-build case_evidence matrix + default gate + G5 pre/post + 5-case dump.
# Main tree is NOT modified; copies live under target/qa-a-check; each configuration has its own
# clean CARGO_TARGET_DIR. Logs for evidence are small extracts; raw full logs stay in target/.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$work = Join-Path $repo 'target\qa-a-check'
$logs = Join-Path $repo 'docs\development-evidence\g10-p5-faithful-fma-fix-independent-check\logs'
New-Item -ItemType Directory -Force -Path $work, $logs | Out-Null
$exits = [System.Collections.Generic.List[string]]::new()

function Clear-CargoEnv {
    $before = @(Get-ChildItem Env: | Where-Object { $_.Name -match '^(CARGO|RUST)' } | Sort-Object Name)
    $lines = @("before_count=$($before.Count)")
    foreach ($entry in $before) { $lines += "env:$($entry.Name)=$($entry.Value)" }
    foreach ($entry in $before) { Remove-Item "Env:$($entry.Name)" -ErrorAction SilentlyContinue }
    $lines | Out-File -Encoding utf8 (Join-Path $logs 'env-snapshot.txt')
}

function Run-Cargo([string]$cwd, [string]$targetDir, [string]$logName, [string[]]$cargoArgs) {
    $env:CARGO_TARGET_DIR = $targetDir
    Push-Location $cwd
    & cargo @cargoArgs *> (Join-Path $logs $logName)
    $exit = $LASTEXITCODE
    Pop-Location
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    $script:exits.Add("$logName exit=$exit")
    return $exit
}

function Toolchain-Info([string]$mode, [string]$toolchain, [string]$rustflags) {
    $rustcArgs = @()
    if ($toolchain -ne '') { $rustcArgs = @($toolchain) }
    $lines = @(
        "mode=$mode",
        "toolchain=$toolchain",
        "RUSTFLAGS=$rustflags",
        "git_head=$((& git -C $repo rev-parse HEAD) -join '')",
        "cargo_version=$((& cargo -V) -join '')",
        ((& rustc @rustcArgs -vV) -join "`n"),
        "config.toml:",
        (Get-Content (Join-Path $repo '.cargo\config.toml') -Raw)
    )
    $lines | Out-File -Encoding utf8 (Join-Path $logs "toolchain-$mode.txt")
}

Clear-CargoEnv
$modes = @(
    @{ id = 'gnu-default'; toolchain = ''; rustflags = '' },
    @{ id = 'gnu-fma'; toolchain = ''; rustflags = '-C target-feature=+fma' },
    @{ id = 'msvc'; toolchain = '+1.99.0-x86_64-pc-windows-msvc'; rustflags = '' }
)

foreach ($mode in $modes) {
    $id = $mode.id
    if ($mode.rustflags -ne '') { $env:RUSTFLAGS = $mode.rustflags }
    else { Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue }
    Toolchain-Info $id $mode.toolchain $mode.rustflags
    $prefix = @()
    if ($mode.toolchain -ne '') { $prefix = @($mode.toolchain) }
    $td = Join-Path $work ("target-{0}" -f $id)
    Run-Cargo $repo $td "$id-case-evidence.log" ($prefix + @(
        'test', '--locked', '--lib', 'physics::collision::gjk::case_evidence', '--', '--nocapture')) | Out-Null
    Run-Cargo $repo $td "$id-case-evidence-assert.log" ($prefix + @(
        'test', '--locked', '--lib', 'physics::collision::gjk::case_evidence::upstream_f32_within_4ulp',
        '--', '--exact')) | Out-Null
    if ($id -eq 'gnu-fma') {
        $env:CARGO_TARGET_DIR = $td
        & cargo @($prefix + @('test', '--locked', '--lib', '--no-run', '-v')) *> (Join-Path $work 'gnu-fma-build-verbose.log')
        $env:CARGO_TARGET_DIR = $td
        $line = (Select-String -Path (Join-Path $work 'gnu-fma-build-verbose.log') -Pattern '--crate-name mjwarp_rs ' |
            Select-Object -Last 1).Line
        @("rustc_line=$line",
          "has_target_feature_fma=$($line -match 'target-feature=\+fma')",
          "has_link_dead_code=$($line -match 'link-dead-code')") |
            Out-File -Encoding utf8 (Join-Path $logs 'gnu-fma-rustc-line.txt')
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    }
    if ($id -eq 'gnu-default') {
        Run-Cargo $repo $td 'g5-post-relmat.log' @('test', '--locked', '--lib', 'physics::collision::relmat') | Out-Null
        Run-Cargo $repo $td 'g5-post-sdf-relmat.log' @('test', '--locked', '--test', 'g10_p5_sdf_relmat', '--', '--nocapture') | Out-Null
        Run-Cargo $repo $td 'g5-post-sdf-host.log' @('test', '--locked', '--test', 'g10_p5_sdf_host') | Out-Null
        Run-Cargo $repo $td 'g5-post-gjk-epa.log' @('test', '--locked', '--test', 'gjk_epa', '--', '--nocapture') | Out-Null
        $env:CARGO_TARGET_DIR = $td
        Push-Location $repo
        & cargo test --locked *> (Join-Path $work 'default-full-test.log')
        $fullExit = $LASTEXITCODE
        Pop-Location
        Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        $exits.Add("default-full-test exit=$fullExit")
        Select-String -Path (Join-Path $work 'default-full-test.log') -Pattern '^test result:' |
            ForEach-Object { $_.Line } |
            Out-File -Encoding utf8 (Join-Path $logs 'default-gate-summary.txt')
        Select-String -Path (Join-Path $work 'default-full-test.log') -Pattern '^running \d+ tests?$' |
            Measure-Object | ForEach-Object { "test_binaries_with_running_lines=$($_.Count)" } |
            Out-File -Encoding utf8 -Append (Join-Path $logs 'default-gate-summary.txt')
        (Select-String -Path (Join-Path $work 'default-full-test.log') -Pattern 'case_evidence' |
            ForEach-Object { $_.Line }) | Out-File -Encoding utf8 (Join-Path $logs 'default-gate-case-evidence-lines.txt')
    }
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
}

# G5 pre (parent commit copy) for pre/post diff=0
$preTree = Join-Path $work 'pre-tree'
$tdPre = Join-Path $work 'target-pre'
Run-Cargo $preTree $tdPre 'g5-pre-relmat.log' @('test', '--locked', '--lib', 'physics::collision::relmat') | Out-Null
Run-Cargo $preTree $tdPre 'g5-pre-sdf-relmat.log' @('test', '--locked', '--test', 'g10_p5_sdf_relmat', '--', '--nocapture') | Out-Null
Run-Cargo $preTree $tdPre 'g5-pre-sdf-host.log' @('test', '--locked', '--test', 'g10_p5_sdf_host') | Out-Null
Run-Cargo $preTree $tdPre 'g5-pre-gjk-epa.log' @('test', '--locked', '--test', 'gjk_epa', '--', '--nocapture') | Out-Null

# 5-case independent dump (r2 + QA dump test, separate copy)
$dumpTree = Join-Path $work 'dump-tree'
$tdDump = Join-Path $work 'target-dump'
$dumpPath = Join-Path $work 'qa-contacts-dump.json'
$env:QA_A_DUMP = $dumpPath
Run-Cargo $dumpTree $tdDump 'dump-test.log' @('test', '--locked', '--lib',
    'physics::collision::gjk::case_evidence::qa_dump_upstream_contacts', '--', '--exact', '--nocapture') | Out-Null
Remove-Item Env:QA_A_DUMP -ErrorAction SilentlyContinue
"dump_exists=$(Test-Path $dumpPath) dump_bytes=$((Get-Item $dumpPath -ErrorAction SilentlyContinue).Length)" |
    Out-File -Encoding utf8 (Join-Path $logs 'dump-test-summary.txt')

$exits | Out-File -Encoding utf8 (Join-Path $logs 'exits.txt')
Write-Host 'QA_A_MATRIX_DONE'
