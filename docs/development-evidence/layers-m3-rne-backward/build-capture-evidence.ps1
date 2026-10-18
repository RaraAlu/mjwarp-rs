$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-rne-backward'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-rne-backward'
$capture = "$runtime/run-01-product/gpu-capture"
$gpuResultPath = "$runtime/run-01-product/result.json"
$deviceResultPath = "$runtime/check-06-device/result.json"
$deviceStdoutPath = "$runtime/check-06-device/stdout.log"
$historical = 'docs/development-evidence/layers-m3-rne-cfrc/formal-run/actual-capture/rne_backward-a5b528ff34301f8e358d13a53b5dfbd69676fcc02987d39a3148d9693b8ebd22.ptx'
$copyPath = "$root/rne_backward-a5b528ff34301f8e358d13a53b5dfbd69676fcc02987d39a3148d9693b8ebd22.ptx"
$outputPath = "$root/capture-inventory.json"
foreach ($path in @($copyPath, $outputPath)) {
    if (Test-Path -LiteralPath $path) { throw "输出已存在：$path" }
}
foreach ($path in @($capture, $gpuResultPath, $deviceResultPath, $deviceStdoutPath, $historical)) {
    if (-not (Test-Path -LiteralPath $path)) { throw "缺少捕获证据：$path" }
}

$gpuResult = Get-Content -LiteralPath $gpuResultPath -Raw | ConvertFrom-Json
$deviceResult = Get-Content -LiteralPath $deviceResultPath -Raw | ConvertFrom-Json
$device = Get-Content -LiteralPath $deviceStdoutPath -Raw | ConvertFrom-Json
if ($gpuResult.exit_code -ne 0 -or $deviceResult.exit_code -ne 0) { throw '正式运行没有通过' }
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or
    $device.resource_lock -cne 'gpu:windows-t4' -or
    $device.cuda_root -cne 'target/toolchains/cuda-12.8.1' -or
    $device.nvrtc_version -cne '12.8') {
    throw '设备或NVRTC身份不符'
}

$captureRoot = (Resolve-Path -LiteralPath $capture).Path
$files = @(Get-ChildItem -LiteralPath $capture -Recurse -File | Sort-Object FullName | ForEach-Object {
    $relative = [IO.Path]::GetRelativePath($captureRoot, $_.FullName).Replace('\', '/')
    [ordered]@{
        relative = $relative
        bytes = $_.Length
        sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
})
$main = @($files | Where-Object { $_.relative -notmatch '/' })
$foreign = @($files | Where-Object { $_.relative.StartsWith('foreign-session/', [StringComparison]::Ordinal) })
$rootOnly = @($files | Where-Object { $_.relative.StartsWith('root-only/', [StringComparison]::Ordinal) })
$mainBackward = @($main | Where-Object { $_.relative -match '^rne_backward-[0-9a-f]{64}\.ptx$' })
$foreignBackward = @($foreign | Where-Object { $_.relative -match '/rne_backward-[0-9a-f]{64}\.ptx$' })
$rootBackward = @($rootOnly | Where-Object { $_.relative -match '/rne_backward-[0-9a-f]{64}\.ptx$' })
if ($mainBackward.Count -ne 1 -or $foreignBackward.Count -ne 1 -or $rootBackward.Count -ne 1) {
    throw '三套反向PTX捕获不唯一'
}

$actualPath = Join-Path $capture $mainBackward[0].relative
$actualBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $actualPath))
$historicalBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $historical))
if (-not [Linq.Enumerable]::SequenceEqual[byte]($actualBytes, $historicalBytes)) {
    throw '当前反向PTX与历史当前PTX不同'
}
[IO.File]::Copy((Resolve-Path -LiteralPath $actualPath), (Join-Path (Get-Location) $copyPath))

$record = [ordered]@{
    schema = 1
    task_id = '2026-10-19-layers-m3-rne-backward'
    capture_root = $capture
    gpu_result = [ordered]@{
        path = $gpuResultPath
        sha256 = (Get-FileHash -LiteralPath $gpuResultPath -Algorithm SHA256).Hash.ToLowerInvariant()
        exit_code = $gpuResult.exit_code
        duration_ms = $gpuResult.duration_ms
        registration_sha256 = $gpuResult.registration_sha256
    }
    device_result = [ordered]@{
        path = $deviceResultPath
        sha256 = (Get-FileHash -LiteralPath $deviceResultPath -Algorithm SHA256).Hash.ToLowerInvariant()
        device = $device.device
        resource_lock = $device.resource_lock
        cuda_root = $device.cuda_root
        nvrtc_version = $device.nvrtc_version
        nvrtc_sha256 = $device.nvrtc_sha256
        nvrtc_builtins_sha256 = $device.nvrtc_builtins_sha256
        task_memory = $device.task_memory
    }
    source = [ordered]@{
        cuda = 'src/physics/rne.cu'
        cuda_sha256 = (Get-FileHash -LiteralPath 'src/physics/rne.cu' -Algorithm SHA256).Hash.ToLowerInvariant()
        rust = 'src/physics/rne.rs'
        rust_sha256 = (Get-FileHash -LiteralPath 'src/physics/rne.rs' -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    ptx = [ordered]@{
        current = $mainBackward[0]
        historical_path = $historical
        historical_sha256 = (Get-FileHash -LiteralPath $historical -Algorithm SHA256).Hash.ToLowerInvariant()
        committed_copy = $copyPath
        committed_copy_sha256 = (Get-FileHash -LiteralPath $copyPath -Algorithm SHA256).Hash.ToLowerInvariant()
        byte_equal = $true
    }
    scopes = [ordered]@{
        main = [ordered]@{ count=$main.Count; files=$main }
        foreign_session = [ordered]@{ count=$foreign.Count; files=$foreign }
        root_only = [ordered]@{ count=$rootOnly.Count; files=$rootOnly }
    }
    boundaries = @(
        'root-only contains compile captures but executes zero backward launches',
        'task GPU memory is unknown and was not rerun for measurement',
        'exact integer bit comparison does not prove general sibling atomic determinism',
        'general atomic tolerance contract remains unchanged but was not revalidated here'
    )
}
$json = ($record | ConvertTo-Json -Depth 20).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema=1
    total_files=$files.Count
    main_files=$main.Count
    foreign_files=$foreign.Count
    root_only_files=$rootOnly.Count
    ptx_sha256=$mainBackward[0].sha256
    historical_equal=$true
    task_memory=$device.task_memory
} | ConvertTo-Json
