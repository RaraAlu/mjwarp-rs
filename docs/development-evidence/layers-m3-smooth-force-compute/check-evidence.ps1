$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-smooth-force-compute'
$task = "docs/development-evidence/layers-m3-smooth-force-compute"
$runtime = "target/normify-evidence/$taskId"
$sourcePath = 'src/physics/smooth_force.rs'
$base = '87a13b7ef921328ddd0554f2dee34aed9ca2de98'
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))

$required = @(
    'struct SmoothForceValidateSpec;',
    'struct SmoothForceComputeSpec;',
    'struct SmoothForceComputeKernel {',
    'validate: SmoothForceValidateKernel',
    'compute: SmoothForceComputeKernel',
    'self.validate.launch(',
    'self.compute.launch(',
    'Checked::<SmoothForceComputeMetadata>::assume_uploaded(',
    'smooth_force_compute_worlds(&dims)',
    'if(t>=0&&awake[1ull*w*stride+t]==0){output[4+tid]=0.0f;return;}',
    'float acc=__fsub_rn(f[0],f[work]);',
    'acc=__fadd_rn(acc,f[2ull*work]);output[4+tid]=__fadd_rn(acc,f[3ull*work]);'
)
foreach ($text in $required) {
    if (-not $source.Contains($text)) { throw "缺少源码证据：$text" }
}
if ($source.Contains('compute: SynchronousKernel')) { throw '计算入口仍使用原始字段' }
if ($source.Contains('staged_output')) { throw '本片引入了额外结果暂存' }

$cuda = [regex]::Match(
    $source,
    'const CUDA: &str = r#"(.*?)"#;',
    [Text.RegularExpressions.RegexOptions]::Singleline
)
if (-not $cuda.Success) { throw '找不到CUDA源码' }
$sha = [Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($cuda.Groups[1].Value))
$cudaHash = [Convert]::ToHexString($sha).ToLowerInvariant()
if ($cudaHash -cne '1a6b10c3c953b38982420382fb7284ce8fe472289863dcce20c489ac5d63f0a4') {
    throw "CUDA源码漂移：$cudaHash"
}
$fixtureHash = (Get-FileHash -LiteralPath 'fixtures/smooth-force/reference.json' -Algorithm SHA256).Hash.ToLowerInvariant()
if ($fixtureHash -cne '9be63dc0aff23410814f432db6c15600b656ddcff2fd692d46abd0195bc2fa0a') {
    throw "夹具漂移：$fixtureHash"
}
$diff = @(& git diff --unified=0 --no-ext-diff $base -- $sourcePath)
if ($LASTEXITCODE -ne 0) { throw "源码差异读取失败：$LASTEXITCODE" }
$toleranceDrift = @($diff | Where-Object {
    $_ -match '^[+-](?![+-])' -and $_ -match '(tolerance|epsilon|abs\(\)\s*<=|[0-9]e-[0-9])'
})
if ($toleranceDrift.Count -ne 0) { throw "容差源码漂移：$($toleranceDrift -join ' | ')" }

foreach ($path in @("$task/claims.json", "$task/execution-plan.json", "$task/unmigrated-launch-inventory.json")) {
    $null = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
}
$claims = Get-Content -LiteralPath "$task/claims.json" -Raw | ConvertFrom-Json
$plan = Get-Content -LiteralPath "$task/execution-plan.json" -Raw | ConvertFrom-Json
$inventory = Get-Content -LiteralPath "$task/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
if ($claims.base_revision -cne $base -or $plan.base_revision -cne $base) { throw '基线提交错误' }
$expectedCounts = [ordered]@{
    actual_cuda_entry_kinds = 88
    typed_actual_entry_kinds = 24
    unmigrated_actual_entry_kinds = 64
    product_low_level_call_sites = 99
    typed_low_level_call_sites = 20
    unmigrated_low_level_call_sites = 79
    wrapper_orchestration_call_sites = 22
    kernel_abi_families = 20
}
foreach ($name in $expectedCounts.Keys) {
    if ($inventory.counts.$name -ne $expectedCounts[$name]) {
        throw "盘点数量错误：$name=$($inventory.counts.$name)"
    }
}
foreach ($entryName in @('smooth_force_validate', 'smooth_force')) {
    $entry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -eq $entryName)
    if ($entry.Count -ne 1 -or $entry[0].migration_status -cne 'typed_safe_wrapper') {
        throw "入口未完成类型化：$entryName"
    }
}

$expectedRuns = [ordered]@{
    'host-wrapper' = 0
    'device-preflight' = 0
    'gpu-sleep-off' = 0
    'gpu-sleep-on' = 0
    'format' = 1
    'format-apply' = 0
    'format-retry-01' = 0
    'clippy' = 0
    'inventory' = 0
}
foreach ($id in $expectedRuns.Keys) {
    $registrationPath = "$runtime/registrations/$id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $result = Get-Content -LiteralPath "$($registration.output_root)/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$($registration.output_root)/started.json" -Raw | ConvertFrom-Json
    $registrationHash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($registration.registered_before_execution -ne $true -or
        [DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $registrationHash -or
        $started.registration_sha256 -cne $registrationHash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        $result.exit_code -ne $expectedRuns[$id]) {
        throw "运行登记或结果错误：$id"
    }
    foreach ($log in @($registration.stdout, $registration.stderr)) {
        if (-not (Test-Path -LiteralPath $log -PathType Leaf)) { throw "运行日志缺失：$log" }
    }
}
$selfRegistration = Get-Content -LiteralPath "$runtime/registrations/evidence.json" -Raw | ConvertFrom-Json
if ([DateTime]$selfRegistration.registered_at_utc -ge [DateTime](Get-Item -LiteralPath "$runtime/check-06-evidence/started.json").CreationTimeUtc) {
    throw '证据检查未事前登记'
}

$device = Get-Content -LiteralPath "$runtime/check-02-device/stdout.log" -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,') { throw '本机T4记录错误' }
if ($device.cuda_root -cne 'target/toolchains/cuda-12.8.1') { throw 'CUDA根目录错误' }

$offReport = Get-Content -LiteralPath "$runtime/run-01-sleep-off/test-report.json" -Raw | ConvertFrom-Json
$onReport = Get-Content -LiteralPath "$runtime/run-02-sleep-on/test-report.json" -Raw | ConvertFrom-Json
foreach ($flag in @('prelaunch_rejection_output_unchanged', 'post_compute_nonfinite_wrote_output', 'failed_result_unreadable', 'same_plan_recovery')) {
    if ($offReport.$flag -ne $true) { throw "关闭休眠报告缺少通过项：$flag" }
}
foreach ($flag in @('internal_awake', 'external_sleep', 'ntree_zero_reused_compute_kernel', 'validation_precedes_compute', 'same_plan_recovery')) {
    if ($onReport.$flag -ne $true) { throw "开启休眠报告缺少通过项：$flag" }
}
foreach ($report in @($offReport, $onReport)) {
    if ($report.plans_compiled -ne 1 -or $report.compiled_entries.Count -ne 2 -or
        $report.task_gpu_memory -cne 'unknown') {
        throw 'GPU报告边界错误'
    }
}

$baseline = 'docs/evidence/passive-force-sleep/gpu-1/ptx'
$captures = @(
    [ordered]@{name='sleep-off';root="$runtime/run-01-sleep-off/gpu-capture";source='3b80d5c3b9e11b06141a9d886ccee6853db1deddbfc852943669f21bc2de0d5c';ptx='2da1129e0a90e79fe317c7a3d9515d0c69863298e64bb0de0cb47a39bc61cb6d'},
    [ordered]@{name='sleep-on';root="$runtime/run-02-sleep-on/gpu-capture";source='097d1f66aa759c0ddd35333bd04d02e60d10828cef1e49ef0b01179231b5f6b3';ptx='d0e78698a54904f24018564234157da53d5e01c55751158e0e15e082b3d41c0f'}
)
foreach ($capture in $captures) {
    $records = @(Get-ChildItem -LiteralPath $capture.root -Filter '*.json' -File)
    $files = @(Get-ChildItem -LiteralPath $capture.root -File | Sort-Object Name)
    if ($records.Count -ne 2 -or $files.Count -ne 6) { throw "$($capture.name)捕获数量错误" }
    $entries = @($records | ForEach-Object {
        $record = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
        if ($record.sourceSha256 -cne $capture.source -or $record.ptxSha256 -cne $capture.ptx) {
            throw "$($capture.name)源码或PTX摘要漂移"
        }
        $record.entry
    })
    if ((Compare-Object ($entries | Sort-Object) @('smooth_force', 'smooth_force_validate')).Count -ne 0 -or
        @($entries | Sort-Object -Unique).Count -ne 2) {
        throw "$($capture.name)入口集合或编译次数错误"
    }
    foreach ($file in $files) {
        $prior = Join-Path $baseline $file.Name
        if (-not (Test-Path -LiteralPath $prior)) { throw "缺少冻结捕获：$($file.Name)" }
        if ((Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash -cne
            (Get-FileHash -LiteralPath $prior -Algorithm SHA256).Hash) {
            throw "捕获字节漂移：$($file.Name)"
        }
    }
}

$historicalGap = 'docs/development-evidence/layers-m3-smooth-force-validate/retry-registration-gap.json'
$gap = Get-Content -LiteralPath $historicalGap -Raw | ConvertFrom-Json
if ($gap.pre_registration -cne 'missing' -or $gap.attempts.Count -ne 6) { throw '历史流程缺口失真' }
& git diff --quiet HEAD -- $historicalGap
if ($LASTEXITCODE -ne 0) { throw '历史流程缺口被修改' }

$status = @(& git status --porcelain=v1 --untracked-files=all)
if ($LASTEXITCODE -ne 0) { throw "工作区读取失败：$LASTEXITCODE" }
foreach ($line in $status) {
    $path = $line.Substring(3).Replace('\', '/')
    if ($path -ne $sourcePath -and -not $path.StartsWith("$task/", [StringComparison]::Ordinal)) {
        throw "范围外工作区文件：$path"
    }
}
& git diff --check
if ($LASTEXITCODE -ne 0) { throw "git diff --check退出码：$LASTEXITCODE" }
[ordered]@{
    schema = 1
    cuda_body_sha256 = $cudaHash
    fixture_sha256 = $fixtureHash
    frozen_capture_variants = 2
    capture_entries_per_variant = 2
    capture_files_per_variant = 6
    capture_matches_prior_bytes = $true
    registered_completed_runs = $expectedRuns.Count
    preserved_format_failure = $true
    current_retry_registration_complete = $true
    historical_missing_registrations = 6
    typed_actual_entry_kinds = 24
    unmigrated_actual_entry_kinds = 64
    typed_low_level_call_sites = 20
    unmigrated_low_level_call_sites = 79
    wrapper_orchestration_call_sites = 22
    validate_typed = $true
    smooth_force_typed = $true
    task_gpu = 'NVIDIA GeForce RTX 4070 Ti SUPER'
    task_memory = 'unknown'
} | ConvertTo-Json
