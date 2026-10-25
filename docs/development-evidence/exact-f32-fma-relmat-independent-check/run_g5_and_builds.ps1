# QA: clean-target product builds (seam boundary binary scan) + G5 bitwise reruns.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$evidence = Join-Path $repo 'docs\development-evidence\exact-f32-fma-relmat-independent-check'
$logs = Join-Path $evidence 'logs'
$root = Join-Path $repo 'target\qa-fma-check'
New-Item -ItemType Directory -Force -Path $logs | Out-Null

function Clear-CargoEnv {
    $before = @(Get-ChildItem Env: | Where-Object { $_.Name -match '^(CARGO|RUST)' } | Sort-Object Name)
    $lines = @("cargo_rust_env_before_count=$($before.Count)")
    foreach ($entry in $before) { $lines += "env:$($entry.Name)=$($entry.Value)" }
    $lines | Out-File -Encoding utf8 (Join-Path $logs 'env-snapshot.txt')
    foreach ($entry in $before) { Remove-Item "Env:$($entry.Name)" -ErrorAction SilentlyContinue }
}
Clear-CargoEnv

# product debug + release builds in dedicated fresh target dirs
foreach ($profile in @('debug', 'release')) {
    $td = Join-Path $root ("prod-{0}" -f $profile)
    if (Test-Path $td) { Remove-Item -Recurse -Force $td }
    $env:CARGO_TARGET_DIR = $td
    $log = Join-Path $logs ("prod-build-{0}.log" -f $profile)
    if ($profile -eq 'release') {
        & cargo build --locked --release *> $log
    } else {
        & cargo build --locked *> $log
    }
    Write-Host "prod-build $profile exit=$LASTEXITCODE"
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
}

# test binary build in its own fresh target dir
$tdTests = Join-Path $root 'tests'
if (Test-Path $tdTests) { Remove-Item -Recurse -Force $tdTests }
$env:CARGO_TARGET_DIR = $tdTests
& cargo test --locked --lib --no-run *> (Join-Path $logs 'test-binary-build.log')
Write-Host "test-binary build exit=$LASTEXITCODE"

# G5 bitwise reruns (same clean target dir)
$g5 = @(
    @{ name = 'lib-math-fma'; args = @('test', '--locked', '--lib', 'math::fma', '--', '--test-threads=1') },
    @{ name = 'lib-relmat'; args = @('test', '--locked', '--lib', 'physics::collision::relmat', '--', '--test-threads=1') },
    @{ name = 'test-g10_p5_sdf_relmat'; args = @('test', '--locked', '--test', 'g10_p5_sdf_relmat', '--', '--nocapture', '--test-threads=1') },
    @{ name = 'test-g10_p5_sdf_host'; args = @('test', '--locked', '--test', 'g10_p5_sdf_host') },
    @{ name = 'test-gjk_epa'; args = @('test', '--locked', '--test', 'gjk_epa', '--', '--nocapture', '--test-threads=1') }
)
$results = @()
foreach ($item in $g5) {
    $log = Join-Path $logs ("g5-{0}.log" -f $item.name)
    $cmdArgs = $item.args
    & cargo @cmdArgs *> $log
    $results += [ordered]@{ name = $item.name; exit = $LASTEXITCODE; log = ("docs/development-evidence/exact-f32-fma-relmat-independent-check/logs/g5-{0}.log" -f $item.name) }
    Write-Host "g5 $($item.name) exit=$LASTEXITCODE"
}

# seam-forced run of the product's own fma tests (test-only env)
$env:MJWARP_FORCE_SOFT_FMA = '1'
& cargo test --locked --lib math::fma -- --test-threads=1 *> (Join-Path $logs 'lib-math-fma-seam.log')
$seamExit = $LASTEXITCODE
Remove-Item Env:MJWARP_FORCE_SOFT_FMA -ErrorAction SilentlyContinue
Write-Host "lib math::fma seam exit=$seamExit"
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
$results | ConvertTo-Json | Out-File -Encoding utf8 (Join-Path $logs 'g5-summary.json')
Write-Host 'G5_RUNNER_DONE'
