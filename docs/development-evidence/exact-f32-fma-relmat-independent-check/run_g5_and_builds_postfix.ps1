# QA post-fix (F1 fixed) clean-target runs: product debug/release builds (seam
# binary scan), test-binary build, G5 bitwise reruns, the new default-gate
# edge-class test (env unset), and the seam-forced product test run.
# All artifacts go to logs-postfix/ + target/qa-fma-check/postfix (fresh dirs);
# prefix logs are left untouched as the historical F1 record.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$OutputEncoding = [Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$repo = 'D:\Rust\mjwarp-rs'
$evidence = Join-Path $repo 'docs\development-evidence\exact-f32-fma-relmat-independent-check'
$logs = Join-Path $evidence 'logs-postfix'
$root = Join-Path $repo 'target\qa-fma-check-postfix'
New-Item -ItemType Directory -Force -Path $logs | Out-Null
Remove-Item Env:MJWARP_FORCE_SOFT_FMA -ErrorAction SilentlyContinue

$headInfo = @(
    ('utc=' + (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')),
    ('git_head=' + ((& git -C $repo rev-parse HEAD) -join '')),
    ('src_math_fma_sha256=' + (Get-FileHash (Join-Path $repo 'src\math\fma.rs') -Algorithm SHA256).Hash.ToLower()),
    ('src_math_fma_tests_sha256=' + (Get-FileHash (Join-Path $repo 'src\math\fma\tests.rs') -Algorithm SHA256).Hash.ToLower()),
    '--- git status --porcelain (product tree) ---'
)
$headInfo += (& git -C $repo status --porcelain)
$headInfo += '--- rustc -vV (default toolchain) ---'
$headInfo += (& rustc -vV)
$headInfo | Out-File -Encoding utf8 (Join-Path $logs 'head-info-postfix.txt')

function Clear-CargoEnv {
    $before = @(Get-ChildItem Env: | Where-Object { $_.Name -match '^(CARGO|RUST)' } | Sort-Object Name)
    $lines = @("cargo_rust_env_before_count=$($before.Count)")
    foreach ($entry in $before) { $lines += "env:$($entry.Name)=$($entry.Value)" }
    $lines += ("MJWARP_FORCE_SOFT_FMA_set=" + ($null -ne (Get-Item Env:MJWARP_FORCE_SOFT_FMA -ErrorAction SilentlyContinue)))
    $lines | Out-File -Encoding utf8 (Join-Path $logs 'env-snapshot-postfix.txt')
    foreach ($entry in $before) { Remove-Item "Env:$($entry.Name)" -ErrorAction SilentlyContinue }
}
Clear-CargoEnv

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

$tdTests = Join-Path $root 'tests'
if (Test-Path $tdTests) { Remove-Item -Recurse -Force $tdTests }
$env:CARGO_TARGET_DIR = $tdTests
& cargo test --locked --lib --no-run *> (Join-Path $logs 'test-binary-build.log')
Write-Host "test-binary build exit=$LASTEXITCODE"

# New default-gate test that must catch F1-class bugs without env/seam/hardware.
& cargo test --locked --lib math::fma::tests::soft_fma_exact_bits_by_edge_class -- --exact --nocapture *> (Join-Path $logs 'g5-lib-soft-fma-edge-class.log')
Write-Host "edge-class test exit=$LASTEXITCODE"

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
    $results += [ordered]@{ name = $item.name; exit = $LASTEXITCODE; log = ("docs/development-evidence/exact-f32-fma-relmat-independent-check/logs-postfix/g5-{0}.log" -f $item.name) }
    Write-Host "g5 $($item.name) exit=$LASTEXITCODE"
}

$env:MJWARP_FORCE_SOFT_FMA = '1'
& cargo test --locked --lib math::fma -- --test-threads=1 *> (Join-Path $logs 'lib-math-fma-seam.log')
$seamExit = $LASTEXITCODE
Remove-Item Env:MJWARP_FORCE_SOFT_FMA -ErrorAction SilentlyContinue
Write-Host "lib math::fma seam exit=$seamExit"
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
$results | ConvertTo-Json | Out-File -Encoding utf8 (Join-Path $logs 'g5-summary-postfix.json')
Write-Host 'G5_POSTFIX_RUNNER_DONE'
