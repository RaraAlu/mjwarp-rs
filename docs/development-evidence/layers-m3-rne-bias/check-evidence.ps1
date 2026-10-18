$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-bias'
$root = 'docs/development-evidence/layers-m3-rne-bias'
$runtime = "target/normify-evidence/$task"
$baseRevision = '9f01475c76e47a837f276c429b0e8994b96bb3e6'

function Get-Sha256([byte[]]$Bytes) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Assert-Run([string]$Id, [int]$ExitCode) {
    $registrationPath = "$runtime/registrations/$Id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $output = [string]$registration.output_root
    $result = Get-Content -LiteralPath "$output/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$output/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        -not (Test-Path -LiteralPath $result.stdout) -or
        -not (Test-Path -LiteralPath $result.stderr) -or
        $result.exit_code -ne $ExitCode) {
        throw "运行证据错误：$Id"
    }
}

$successfulRuns = @(
    'host-wrapper-retry-01',
    'format-final',
    'default-lib',
    'clippy-all-targets-retry-01',
    'input-tolerance-gate',
    'device-preflight',
    'gpu-product-captured',
    'inventory',
    'precision-map-retry-01'
)
foreach ($id in $successfulRuns) { Assert-Run $id 0 }
Assert-Run 'format' 1
Assert-Run 'clippy-all-targets' 101
Assert-Run 'precision-map' 1

$sourcePath = 'src/physics/rne_bias.rs'
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath)).Replace("`r`n", "`n")
$testStart = [regex]::Match($source, '(?m)^#\[cfg\(test\)\]\s*\nmod tests')
if (-not $testStart.Success) { throw '找不到测试模块边界' }
$product = $source.Substring(0, $testStart.Index)
$cuda = [regex]::Match($source, '(?s)const RNE_BIAS_CUDA: &str = r#"(?<body>.*?)"#;')
if (-not $cuda.Success) { throw '找不到RNE_BIAS_CUDA' }
$cudaHash = Get-Sha256 ([Text.Encoding]::UTF8.GetBytes($cuda.Groups['body'].Value))
if ($cudaHash -cne '78239ca47c569395cdfce2b74f749813b73bbb0c032c51dd5d4422e5a1066667') {
    throw '产品CUDA源码变化'
}
if ([regex]::Matches($product, 'self\.kernel\.launch_flex_edges\s*\(').Count -ne 1 -or
    [regex]::Matches($product, 'self\.kernel\s*\.\s*launch\s*\(').Count -ne 1 -or
    [regex]::Matches($product, 'RneBiasKernel::compile').Count -ne 1 -or
    -not $product.Contains('access: Access::ReadWrite') -or
    -not $product.Contains('let launch = self.kernel.preflight(')) {
    throw '包装调用结构不符'
}
foreach ($token in @(
    'MetadataRevision',
    'validate_rne_bias_metadata',
    'rne_bias_lengths',
    'RneBiasSpec',
    'RneBiasMetadataBuffer',
    'SessionMismatch',
    'OverlappingCopy'
)) {
    if (-not $product.Contains($token)) { throw "缺少包装证据：$token" }
}

$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
if ($inventory.base_commit -cne $baseRevision -or
    $inventory.counts.actual_cuda_entry_kinds -ne 88 -or
    $inventory.counts.typed_actual_entry_kinds -ne 29 -or
    $inventory.counts.unmigrated_actual_entry_kinds -ne 59 -or
    $inventory.counts.product_low_level_call_sites -ne 99 -or
    $inventory.counts.typed_low_level_call_sites -ne 25 -or
    $inventory.counts.unmigrated_low_level_call_sites -ne 74 -or
    $inventory.counts.wrapper_orchestration_call_sites -ne 27 -or
    $inventory.counts.kernel_abi_families -ne 25 -or
    $inventory.next_candidate.entry -ceq 'rne_bias') {
    throw '盘点结果错误'
}
$biasEntry = @($inventory.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rne_bias')
if ($biasEntry.Count -ne 1 -or $biasEntry[0].migration_status -cne 'typed_safe_wrapper') {
    throw 'rne_bias没有完成重分类'
}

$precision = Get-Content -LiteralPath "$root/precision-revalidation-v2.json" -Raw | ConvertFrom-Json
if ($precision.source_identity.base_revision -cne $baseRevision -or
    $precision.frozen_product.rne_bias_cuda_sha256 -cne $cudaHash -or
    $precision.frozen_product.rne_bias_cuda_unchanged -ne $true -or
    $precision.frozen_product.transfer_kernel_unchanged -ne $true -or
    $precision.generated_product.historical_and_all_current_captures_exact -ne $true -or
    $precision.generated_product.receipt_contract_exact -ne $true -or
    $precision.generated_product.ptx_f64_mentions -ne 0 -or
    $precision.generated_product.ptx_atomic_mentions -ne 0 -or
    $precision.validations.worlds -ne 513 -or
    $precision.historical_precision.failed_blocks -ne 1042 -or
    $precision.historical_precision.repaired_by_this_slice -ne $false -or
    @($precision.boundaries.PSObject.Properties | Where-Object Value -ne $false).Count -ne 0) {
    throw '精度映射或边界错误'
}

$capture = "$runtime/run-01-product/gpu-capture"
$captureFiles = @(Get-ChildItem -LiteralPath $capture -Recurse -File)
if ($captureFiles.Count -ne 39) { throw "捕获文件数量错误：$($captureFiles.Count)" }
foreach ($path in @(
    "$capture/rne_bias-5e5433159d2749ba4d5c29bff60a239891c1cd75030a1e66ec84f1bacf32f934.cu",
    "$capture/rne_bias-5e5433159d2749ba4d5c29bff60a239891c1cd75030a1e66ec84f1bacf32f934.ptx",
    "$capture/rne_bias-5e5433159d2749ba4d5c29bff60a239891c1cd75030a1e66ec84f1bacf32f934.json",
    "$capture/foreign-session/rne_bias-5e5433159d2749ba4d5c29bff60a239891c1cd75030a1e66ec84f1bacf32f934.ptx",
    "$capture/zero-dof/rne_bias-5e5433159d2749ba4d5c29bff60a239891c1cd75030a1e66ec84f1bacf32f934.ptx"
)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "缺少捕获原件：$path" }
}

& git diff --quiet $baseRevision -- src/runtime/transfer/kernel.rs fixtures/precision/t4-current-kernels.json fixtures/precision/t4-kernels.json
if ($LASTEXITCODE -ne 0) { throw '冻结运行时或T4清单发生变化' }
$plan = Get-Content -LiteralPath "$root/execution-plan.json" -Raw | ConvertFrom-Json
$commands = @($plan.registered_validations | ForEach-Object { "$($_.executable) $($_.arguments -join ' ')" }) -join "`n"
if ($commands -match '(?i)(^|\s)python(\.exe)?(\s|$)') { throw '发现Python执行链' }

[ordered]@{
    schema=1
    task_id=$task
    successful_runs=$successfulRuns.Count
    expected_failures=3
    capture_files=$captureFiles.Count
    cuda_sha256=$cudaHash
    next_candidate=$inventory.next_candidate.entry
    result='pass'
} | ConvertTo-Json
