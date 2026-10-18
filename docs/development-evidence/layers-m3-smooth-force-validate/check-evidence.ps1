$ErrorActionPreference = 'Stop'
$task = 'docs/development-evidence/layers-m3-smooth-force-validate'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-smooth-force-validate'
$sourcePath = 'src/physics/smooth_force.rs'
$base = 'c2fc21e9569128188b74d0321a992f03a2955da6'
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$required = @(
    'struct SmoothForceValidateSpec;',
    'struct SmoothForceValidateKernel {',
    'validate: SmoothForceValidateKernel',
    'compute: SynchronousKernel',
    'self.validate.launch(',
    'self.compute.launch(',
    'struct SmoothForceMetadataBuffer {',
    'revision: MetadataRevision',
    'Checked::<SmoothForceValidateMetadata>::assume_uploaded(',
    'if(t>=0&&awake[1ull*w*stride+t]==0)continue;',
    'float acc=__fsub_rn(f[0],f[work]);',
    'acc=__fadd_rn(acc,f[2ull*work]);output[4+tid]=__fadd_rn(acc,f[3ull*work]);'
)
foreach ($text in $required) {
    if (-not $source.Contains($text)) { throw "缺少源码证据：$text" }
}
if ($source.Contains('kernels: [SynchronousKernel; 2]')) { throw '验证入口仍使用原始数组字段' }
if ($source.Contains('struct SmoothForceComputeSpec;')) { throw '计算入口被错误迁移' }

$cuda = [regex]::Match(
    $source,
    'const CUDA: &str = r#"(.*?)"#;',
    [Text.RegularExpressions.RegexOptions]::Singleline
)
if (-not $cuda.Success) { throw '找不到CUDA源码' }
$sha = [Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($cuda.Groups[1].Value))
$cudaHash = [Convert]::ToHexString($sha).ToLowerInvariant()
if ($cudaHash -ne '1a6b10c3c953b38982420382fb7284ce8fe472289863dcce20c489ac5d63f0a4') {
    throw "CUDA源码漂移：$cudaHash"
}
$fixtureHash = (Get-FileHash -LiteralPath 'fixtures/smooth-force/reference.json' -Algorithm SHA256).Hash.ToLowerInvariant()
if ($fixtureHash -ne '9be63dc0aff23410814f432db6c15600b656ddcff2fd692d46abd0195bc2fa0a') {
    throw "夹具漂移：$fixtureHash"
}
$diff = @(& git diff --unified=0 --no-ext-diff $base -- $sourcePath)
if ($LASTEXITCODE -ne 0) { throw "源码差异读取失败：$LASTEXITCODE" }
$toleranceDrift = @($diff | Where-Object {
    $_ -match '^[+-](?![+-])' -and $_ -match '(tolerance|epsilon|abs\(\)\s*<=|[0-9]e-[0-9])'
})
if ($toleranceDrift.Count -ne 0) { throw "容差源码漂移：$($toleranceDrift -join ' | ')" }

foreach ($path in @("$task/claims.json", "$task/execution-plan.json", "$task/retry-registration-gap.json", "$task/unmigrated-launch-inventory.json")) {
    $null = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
}
$gap = Get-Content -LiteralPath "$task/retry-registration-gap.json" -Raw | ConvertFrom-Json
if ($gap.pre_registration -cne 'missing' -or $gap.recorded_after_execution -ne $true -or $gap.attempts.Count -ne 6) {
    throw '重试登记缺口记录失真'
}
$expectedToolIds = [ordered]@{
    'check-01-host-retry-01' = 'exec-16226b0c-4dcd-4780-88a5-027fc5ecc295'
    'check-01-host-retry-02' = 'exec-564a2a8b-03d1-4084-9863-0ea75953280a'
    'run-01-sleep-off-retry-01' = 'exec-472297a4-ec56-4d38-8dca-b9256a1162df'
    'run-02-sleep-on-retry-01' = 'exec-e8bc510a-b518-42a8-a48f-5ed176091c7f'
    'check-04-clippy-retry-01' = 'exec-b553bc2e-9a77-4f94-8219-bce8a7eab67e'
    'check-03-format-final' = 'exec-1c5f11ad-dc42-4a81-bae6-fd937070c819'
}
foreach ($attempt in $gap.attempts) {
    if ($attempt.tool_call_id -cne $expectedToolIds[$attempt.root] -or $attempt.tool_id_source -cne 'supervisor read_thread') {
        throw "重试工具ID错误：$($attempt.root)"
    }
}
$inventory = Get-Content -LiteralPath "$task/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
$expectedCounts = [ordered]@{
    actual_cuda_entry_kinds = 88
    typed_actual_entry_kinds = 23
    unmigrated_actual_entry_kinds = 65
    product_low_level_call_sites = 99
    typed_low_level_call_sites = 19
    unmigrated_low_level_call_sites = 80
    wrapper_orchestration_call_sites = 21
    kernel_abi_families = 19
}
foreach ($name in $expectedCounts.Keys) {
    if ($inventory.counts.$name -ne $expectedCounts[$name]) {
        throw "盘点数量错误：$name=$($inventory.counts.$name)"
    }
}
$validate = @($inventory.actual_cuda_entry_kinds | Where-Object entry -eq 'smooth_force_validate')
$compute = @($inventory.actual_cuda_entry_kinds | Where-Object entry -eq 'smooth_force')
if ($validate.Count -ne 1 -or $validate[0].migration_status -cne 'typed_safe_wrapper') {
    throw '验证入口没有完成重分类'
}
if ($compute.Count -ne 1 -or $compute[0].migration_status -cne 'raw_unmigrated') {
    throw '计算入口不再保持原始状态'
}

$requiredRuns = [ordered]@{
    'check-01-host/result.json' = 101
    'check-01-host-retry-01/result.json' = 0
    'check-01-host-retry-02/result.json' = 0
    'check-02-device/result.json' = 0
    'run-01-sleep-off/result.json' = 0
    'run-01-sleep-off-retry-01/result.json' = 0
    'run-02-sleep-on/result.json' = 0
    'run-02-sleep-on-retry-01/result.json' = 0
    'check-03-format/result.json' = 0
    'check-03-format-final/result.json' = 0
    'check-04-clippy/result.json' = 101
    'check-04-clippy-retry-01/result.json' = 0
}
foreach ($relative in $requiredRuns.Keys) {
    $run = Get-Content -LiteralPath "$runtime/$relative" -Raw | ConvertFrom-Json
    if ($run.exit_code -ne $requiredRuns[$relative]) { throw "运行退出码错误：$relative" }
}
$device = Get-Content -LiteralPath "$runtime/check-02-device/stdout.log" -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,') { throw '本机T4记录错误' }
if ($device.cuda_root -cne 'target/toolchains/cuda-12.8.1') { throw 'CUDA根目录错误' }

$offReport = Get-Content -LiteralPath "$runtime/run-01-sleep-off-retry-01/test-report.json" -Raw | ConvertFrom-Json
$onReport = Get-Content -LiteralPath "$runtime/run-02-sleep-on-retry-01/test-report.json" -Raw | ConvertFrom-Json
foreach ($report in @($offReport, $onReport)) {
    foreach ($flag in @('validation_rejection_entered_gpu', 'output_bits_unchanged', 'same_plan_recovery')) {
        if ($report.$flag -ne $true) { throw "GPU报告缺少通过项：$flag" }
    }
    if ($report.plans_compiled -ne 1 -or $report.task_gpu_memory -cne 'unknown') {
        throw 'GPU报告边界错误'
    }
}
foreach ($flag in @('internal_awake', 'external_sleep', 'ntree_zero_reused_validation_kernel')) {
    if ($onReport.$flag -ne $true) { throw "休眠报告缺少通过项：$flag" }
}

$baseline = 'docs/evidence/passive-force-sleep/gpu-1/ptx'
$captures = @(
    [ordered]@{name='sleep-off';root="$runtime/run-01-sleep-off-retry-01/gpu-capture";source='3b80d5c3b9e11b06141a9d886ccee6853db1deddbfc852943669f21bc2de0d5c';ptx='2da1129e0a90e79fe317c7a3d9515d0c69863298e64bb0de0cb47a39bc61cb6d'},
    [ordered]@{name='sleep-on';root="$runtime/run-02-sleep-on-retry-01/gpu-capture";source='097d1f66aa759c0ddd35333bd04d02e60d10828cef1e49ef0b01179231b5f6b3';ptx='d0e78698a54904f24018564234157da53d5e01c55751158e0e15e082b3d41c0f'}
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
    if ((Compare-Object ($entries | Sort-Object) @('smooth_force', 'smooth_force_validate')).Count -ne 0) {
        throw "$($capture.name)入口集合错误"
    }
    if (@($entries | Sort-Object -Unique).Count -ne 2) { throw "$($capture.name)入口重复编译" }
    foreach ($file in $files) {
        $prior = Join-Path $baseline $file.Name
        if (-not (Test-Path -LiteralPath $prior)) { throw "缺少冻结捕获：$($file.Name)" }
        $currentHash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
        $priorHash = (Get-FileHash -LiteralPath $prior -Algorithm SHA256).Hash
        if ($currentHash -cne $priorHash) { throw "捕获字节漂移：$($file.Name)" }
    }
}

$tracked = @(& git diff --name-only)
if ($LASTEXITCODE -ne 0) { throw "变更范围读取失败：$LASTEXITCODE" }
foreach ($path in $tracked) {
    if ($path -ne $sourcePath) { throw "范围外跟踪文件：$path" }
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
    typed_actual_entry_kinds = 23
    unmigrated_actual_entry_kinds = 65
    typed_low_level_call_sites = 19
    unmigrated_low_level_call_sites = 80
    wrapper_orchestration_call_sites = 21
    validate_typed = $true
    smooth_force_raw = $true
    task_gpu = 'NVIDIA GeForce RTX 4070 Ti SUPER'
    task_memory = 'unknown'
} | ConvertTo-Json
