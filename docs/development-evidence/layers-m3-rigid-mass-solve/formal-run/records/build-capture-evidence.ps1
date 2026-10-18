$ErrorActionPreference = 'Stop'
$taskId = '2026-10-19-layers-m3-rigid-mass-solve'
$root = 'docs/development-evidence/layers-m3-rigid-mass-solve'
$runtime = "target/normify-evidence/$taskId"
$run1 = "$runtime/run-01-product/gpu-capture"
$run2 = "$runtime/run-02-product/gpu-capture"
$outputPath = "$root/capture-inventory-v2.json"
if (Test-Path -LiteralPath $outputPath) { throw "捕获盘点已存在：$outputPath" }
foreach ($path in @($run1, $run2, "$runtime/run-01-product/result.json", "$runtime/run-02-product/result.json")) {
    if (-not (Test-Path -LiteralPath $path)) { throw "缺少捕获原件：$path" }
}

function Get-Capture([string]$Path) {
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $files = @(Get-ChildItem -LiteralPath $Path -Recurse -File | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            relative=[IO.Path]::GetRelativePath($resolved, $_.FullName).Replace('\', '/')
            bytes=$_.Length
            sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    })
    $dirs = @('typed-main','foreign-session','product-invalid','product-nonfinite','product-recovery','zero-dof')
    if ($files.Count -ne 18) { throw "捕获文件数量错误：$Path" }
    foreach ($dir in $dirs) {
        $group = @($files | Where-Object { $_.relative.StartsWith("$dir/", [StringComparison]::Ordinal) })
        if ($group.Count -ne 3 -or
            @($group | Where-Object relative -Match '\.cu$').Count -ne 1 -or
            @($group | Where-Object relative -Match '\.ptx$').Count -ne 1 -or
            @($group | Where-Object relative -Match '\.json$').Count -ne 1) {
            throw "编译捕获不完整：$Path/$dir"
        }
    }
    $ptx = @($files | Where-Object relative -Match '\.ptx$')
    $cu = @($files | Where-Object relative -Match '\.cu$')
    if (@($ptx.sha256 | Sort-Object -Unique).Count -ne 1 -or
        @($cu.sha256 | Sort-Object -Unique).Count -ne 1) {
        throw "同轮编译产物不一致：$Path"
    }
    [ordered]@{root=$Path;count=$files.Count;files=$files;ptx_sha256=$ptx[0].sha256;cuda_sha256=$cu[0].sha256}
}

$first = Get-Capture $run1
$current = Get-Capture $run2
if ($first.ptx_sha256 -cne $current.ptx_sha256 -or $first.cuda_sha256 -cne $current.cuda_sha256) {
    throw '两轮NVRTC产物不一致'
}
$gpuIdentity = Get-Content -LiteralPath "$root/pre-validation-source-identity.json" -Raw | ConvertFrom-Json
$identity = Get-Content -LiteralPath "$root/pre-validation-source-identity-v2.json" -Raw | ConvertFrom-Json
$sourceBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $identity.source))
$sourceHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($sourceBytes)).ToLowerInvariant()
$sourceBlob = @(& git hash-object --no-filters -- $identity.source)[0].Trim()
if ($identity.sha256 -cne $sourceHash -or $identity.working_blob -cne $sourceBlob) {
    throw '验证期间Rust源码发生漂移'
}
$baseText = @(& git show "a2d057618d2884234e9b6779e619cce5abfc2adc`:src/physics/mass_solve.rs") -join "`n"
$currentText = [IO.File]::ReadAllText((Resolve-Path -LiteralPath 'src/physics/mass_solve.rs')).Replace("`r`n", "`n")
$pattern = '(?s)const MASS_SOLVE_CUDA: &str = r#"(?<cuda>.*?)"#;'
$baseMatch = [regex]::Match($baseText, $pattern)
$currentMatch = [regex]::Match($currentText, $pattern)
if (-not $baseMatch.Success -or -not $currentMatch.Success -or
    $baseMatch.Groups['cuda'].Value -cne $currentMatch.Groups['cuda'].Value) {
    throw '冻结CUDA字符串发生变化'
}
$ptxSource = Get-ChildItem -LiteralPath "$run2/typed-main" -Filter '*.ptx' -File
if (@($ptxSource).Count -ne 1) { throw '权威PTX不唯一' }
$copyPath = "$root/rigid_mass_solve-$($current.ptx_sha256).ptx"
if (-not (Test-Path -LiteralPath $copyPath -PathType Leaf) -or
    (Get-FileHash -LiteralPath $copyPath -Algorithm SHA256).Hash.ToLowerInvariant() -cne $current.ptx_sha256) {
    throw "PTX副本缺失或漂移：$copyPath"
}
$device = Get-Content -LiteralPath "$runtime/check-06-device/stdout.log" -Raw | ConvertFrom-Json
$record = [ordered]@{
    schema=1
    task_id=$taskId
    gpu_validation_source_identity=$gpuIdentity
    current_source_identity=$identity
    source_unchanged_since_host_test_fix=$true
    product_source_equal_to_gpu_validation_source=$identity.product_prefix_equal_to_gpu_validation_source
    full_rust_file_equal_to_gpu_validation_source=$false
    frozen_cuda_unchanged_from_base=$true
    first_run=$first
    first_run_limit='首轮2世界运行早于三项监督修复，且没有独立Rust源码快照'
    first_run_transfers_to_current_source=$false
    authoritative_run=$current
    authoritative_worlds=257
    capture_per_compile=$true
    ptx=[ordered]@{sha256=$current.ptx_sha256;copy=$copyPath;first_and_second_byte_equal=$true}
    cuda=[ordered]@{sha256=$current.cuda_sha256;base_and_current_string_equal=$true}
    device_result=$device
}
$json = ($record | ConvertTo-Json -Depth 20).Replace("`r`n", "`n") + "`n"
$outputBytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
$outputStream = [IO.File]::Open(
    (Join-Path (Get-Location) $outputPath),
    [IO.FileMode]::CreateNew,
    [IO.FileAccess]::Write,
    [IO.FileShare]::Read
)
try { $outputStream.Write($outputBytes, 0, $outputBytes.Length) } finally { $outputStream.Dispose() }
$outputHash = (Get-FileHash -LiteralPath $outputPath -Algorithm SHA256).Hash.ToLowerInvariant()
$outputBlob = @(& git hash-object --no-filters -- $outputPath)[0].Trim()
if ($LASTEXITCODE -ne 0 -or $outputBlob -notmatch '^[0-9a-f]{40}$') { throw '无法登记捕获盘点对象' }
[ordered]@{
    schema=1
    output_path=$outputPath
    output_sha256=$outputHash
    output_blob=$outputBlob
    record=$record
} | ConvertTo-Json -Depth 20
