$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rigid-mass-matrix'
$root = 'docs/development-evidence/layers-m3-rigid-mass-matrix'
$runs = "$root/runs"
$runtime = "target/normify-evidence/$task"
$sourcePath = 'src/physics/mass_matrix.rs'
$sourceSha = '06d2715a394ca9639de205979f53c3ae7b16f9227d06926286ddc000e51c7e5b'
$ptxSha = 'f695b0e8a44220743d444b6286748b8e1648a12616693e863dccb6e80c2548b0'
$cuSha = '48048bb6e1da02de252c4231cd09843e61f69df52c265ca9131d1d3f1d2b0b8d'

function Get-Sha([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

# 归档运行：登记先于执行，日志预建，退出码符合。
function Assert-Run([string]$Id, [int]$ExitCode = 0) {
    $registrationPath = "$runs/registrations/$Id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $output = ([string]$registration.output_root).Replace("$runtime/", "$runs/")
    $result = Get-Content -LiteralPath "$output/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$output/started.json" -Raw | ConvertFrom-Json
    $hash = Get-Sha $registrationPath
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        -not (Test-Path -LiteralPath "$output/stdout.log" -PathType Leaf) -or
        -not (Test-Path -LiteralPath "$output/stderr.log" -PathType Leaf) -or
        $result.exit_code -ne $ExitCode) {
        throw "运行证据错误：$Id"
    }
    $output
}

foreach ($id in 'format-apply', 'format-check', 'clippy-default', 'clippy-cuda', 'device-preflight', 'inventory') {
    Assert-Run $id | Out-Null
}
$hostWrapper = Get-Content -LiteralPath "$(Assert-Run 'host-wrapper')/stdout.log" -Raw
$hostModule = Get-Content -LiteralPath "$(Assert-Run 'host-module')/stdout.log" -Raw
$gpu = Get-Content -LiteralPath "$(Assert-Run 'gpu-wrapper')/stdout.log" -Raw
$regression = Get-Content -LiteralPath "$(Assert-Run 'gpu-mass-regression' 101)/stdout.log" -Raw
$report = Get-Content -LiteralPath "$(Assert-Run 'gpu-report-regression')/stdout.log" -Raw
if ($hostWrapper -notmatch '(?m)^running 2 tests$' -or $hostWrapper -notmatch '2 passed; 0 failed' -or
    $hostModule -notmatch '(?m)^running 12 tests$' -or $hostModule -notmatch '12 passed; 0 failed' -or
    $gpu -notmatch 'runs_rigid_mass_matrix_wrapper_paths_and_recovers \.\.\. ok' -or
    $gpu -notmatch '1 passed; 0 failed' -or
    $regression -notmatch '41 passed; 2 failed' -or
    $report -notmatch '2 passed; 0 failed') {
    throw '测试结果不符'
}
# 首轮两项失败只来自报告路径缺失。
$failures = [regex]::Matches($regression, "(?m)^test (\S+) \.\.\. FAILED$") | ForEach-Object { $_.Groups[1].Value }
$expected = @(
    'physics::mass_matrix::multiply::tests::gpu_multiplies_dense_and_sparse_with_skip_and_independent_alternate',
    'physics::mass_solve::sparse::mixed::solve::resident::tests::gpu_complete_bridge_copies_actual_dense_bits_and_keeps_original_field_tolerances'
)
if (@($failures).Count -ne 2 -or (Compare-Object @($failures) $expected)) { throw '回归失败项不符' }
foreach ($name in $expected) {
    if ($report -notmatch ([regex]::Escape($name) + ' \.\.\. ok')) { throw "复验缺项：$name" }
}

$device = Get-Content -LiteralPath "$runs/check-07-device/stdout.log" -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or $device.nvrtc_version -cne '12.8' -or
    $device.resource_lock -cne 'gpu:windows-t4') {
    throw '设备身份不符'
}

# 四次捕获的源码与PTX逐字节一致，并等于历史记录。
$capture = "$runs/run-01-gpu/gpu-capture"
foreach ($label in 'raw-baseline', 'typed-main', 'foreign-session', 'product-main') {
    $prefix = "$capture/$label/rigid_mass_matrix-$ptxSha"
    if ((Get-Sha "$prefix.ptx") -cne $ptxSha -or (Get-Sha "$prefix.cu") -cne $cuSha) {
        throw "捕获摘要不符：$label"
    }
    $record = Get-Content -LiteralPath "$prefix.json" -Raw | ConvertFrom-Json
    if ($record.ptxSha256 -cne $ptxSha -or $record.sourceSha256 -cne $cuSha -or $record.passed -ne $true -or
        $record.useFastMath -ne $false) {
        throw "捕获记录不符：$label"
    }
}
$historical = Get-Content -LiteralPath 'docs/evidence/t4-g03-retest/report.json' -Raw
if (-not $historical.Contains("rigid_mass_matrix-$ptxSha.json")) { throw '历史PTX记录缺失' }

if ((Get-Sha $sourcePath) -cne $sourceSha) { throw '当前源码与验证源码不符' }
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$start = $source.IndexOf('const MASS_MATRIX_CUDA: &str = r#"')
$end = $source.IndexOf('"#;', $start)
$kernel = $source.Substring($start + 'const MASS_MATRIX_CUDA: &str = r#"'.Length, $end - $start - 'const MASS_MATRIX_CUDA: &str = r#"'.Length)
$kernelSha = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($kernel))).ToLowerInvariant()
if ($kernelSha -cne $cuSha) { throw '内核源码摘要不符' }

$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
$entry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rigid_mass_matrix')
if ($inventory.task_id -cne $task -or $entry.Count -ne 1 -or $entry[0].migration_status -cne 'typed_safe_wrapper' -or
    $inventory.counts.unmigrated_actual_entry_kinds -ne 56 -or $inventory.counts.unmigrated_low_level_call_sites -ne 71 -or
    $inventory.source_file_sha256 -cne (Get-Sha $inventory.source_file)) {
    throw '清单不符'
}

[ordered]@{
    schema = 1
    task_id = $task
    runs = 11
    ptx_captures = 4
    source_sha256 = $sourceSha
    result = 'pass'
} | ConvertTo-Json
