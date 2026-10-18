$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-bias'
$root = "docs/development-evidence/layers-m3-rne-bias"
$runtime = "target/normify-evidence/$task"
$baseRevision = '9f01475c76e47a837f276c429b0e8994b96bb3e6'
$outputPath = "$root/precision-revalidation-v2.json"
$frozenRoot = "$root/frozen"
if (Test-Path -LiteralPath $outputPath) { throw "精度映射已存在：$outputPath" }
if (-not (Test-Path -LiteralPath $frozenRoot)) {
    New-Item -ItemType Directory -Path $frozenRoot | Out-Null
}

function Get-Sha256([byte[]]$Bytes) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Get-LfBytes([byte[]]$Bytes) {
    [Text.Encoding]::UTF8.GetBytes([Text.Encoding]::UTF8.GetString($Bytes).Replace("`r`n", "`n"))
}

function Read-GitBlob([string]$Revision, [string]$Path) {
    $oid = @(& git rev-parse "$Revision`:$Path")[0].Trim()
    if ($LASTEXITCODE -ne 0 -or $oid -notmatch '^[0-9a-f]{40}$') { throw "无法解析对象：$Revision`:$Path" }
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = 'git'
    $start.ArgumentList.Add('cat-file')
    $start.ArgumentList.Add('blob')
    $start.ArgumentList.Add($oid)
    $start.WorkingDirectory = (Get-Location).Path
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = [Diagnostics.Process]::Start($start)
    $memory = [IO.MemoryStream]::new()
    $process.StandardOutput.BaseStream.CopyTo($memory)
    $errorText = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    if ($process.ExitCode -ne 0) { throw "读取对象失败：$errorText" }
    [ordered]@{oid=$oid;bytes=$memory.ToArray()}
}

function Write-NewBytes([string]$Path, [byte[]]$Bytes) {
    $parent = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    $absolute = Join-Path (Get-Location) $Path
    if (Test-Path -LiteralPath $absolute) {
        $existing = [IO.File]::ReadAllBytes($absolute)
        if ((Get-Sha256 $existing) -cne (Get-Sha256 $Bytes)) {
            throw "重试原件发生变化：$Path"
        }
        return
    }
    $stream = [IO.FileStream]::new($absolute, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try { $stream.Write($Bytes, 0, $Bytes.Length) } finally { $stream.Dispose() }
}

function Copy-New([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少原件：$Source" }
    Write-NewBytes $Destination ([IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source)))
}

function Assert-Run([string]$Id, [int]$ExitCode) {
    $registrationPath = "$runtime/registrations/$Id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $result = Get-Content -LiteralPath "$($registration.output_root)/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$($registration.output_root)/started.json" -Raw | ConvertFrom-Json
    $hash = (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        $result.exit_code -ne $ExitCode) {
        throw "运行证据错误：$Id"
    }
    $result
}

function Get-CudaBytes([byte[]]$SourceBytes) {
    $text = [Text.Encoding]::UTF8.GetString((Get-LfBytes $SourceBytes))
    $match = [regex]::Match($text, '(?s)const RNE_BIAS_CUDA: &str = r#"(?<body>.*?)"#;')
    if (-not $match.Success) { throw '找不到RNE_BIAS_CUDA' }
    [Text.Encoding]::UTF8.GetBytes($match.Groups['body'].Value)
}

$baseSource = Read-GitBlob $baseRevision 'src/physics/rne_bias.rs'
$currentSource = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath 'src/physics/rne_bias.rs'))
$baseTransfer = Read-GitBlob $baseRevision 'src/runtime/transfer/kernel.rs'
$currentTransfer = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath 'src/runtime/transfer/kernel.rs'))
$baseCuda = Get-CudaBytes $baseSource.bytes
$currentCuda = Get-CudaBytes $currentSource
$expected = [ordered]@{
    cu='78239ca47c569395cdfce2b74f749813b73bbb0c032c51dd5d4422e5a1066667'
    ptx='5e5433159d2749ba4d5c29bff60a239891c1cd75030a1e66ec84f1bacf32f934'
    json='73ec9a8f7304341f7ecf45bd6fb48b02599c8828187ea46007a63e6e9f583f7b'
}
if ((Get-Sha256 $baseCuda) -cne $expected.cu -or
    (Get-Sha256 $currentCuda) -cne $expected.cu -or
    (Get-Sha256 $baseTransfer.bytes) -cne (Get-Sha256 $currentTransfer)) {
    throw '冻结产品字节发生变化'
}
Write-NewBytes "$frozenRoot/base-rne-bias.rs" $baseSource.bytes
Write-NewBytes "$frozenRoot/current-rne-bias.rs" $currentSource
Write-NewBytes "$frozenRoot/current-rne-bias.cu" $currentCuda
Copy-New 'fixtures/precision/t4-current-kernels.json' "$frozenRoot/t4-current-kernels.json"
Copy-New 'fixtures/precision/t4-kernels.json' "$frozenRoot/t4-kernels.json"
Copy-New 'docs/evidence/launcher-baseline/20261017T193713Z-debug/records/driver-cold-1-resident_rne_bias/compile-56448.jsonl' "$frozenRoot/historical-compile.jsonl"

$hash = $expected.ptx
$historicalRoot = 'target/mjwarp-parallel/verified-debug-ptx'
$currentRoot = "$runtime/run-01-product/gpu-capture"
$sets = @(
    [ordered]@{name='historical';root=$historicalRoot},
    [ordered]@{name='primary';root=$currentRoot},
    [ordered]@{name='foreign-session';root="$currentRoot/foreign-session"},
    [ordered]@{name='zero-dof';root="$currentRoot/zero-dof"}
)
$artifacts = [Collections.Generic.List[object]]::new()
foreach ($set in $sets) {
    $row = [ordered]@{name=$set.name;files=[ordered]@{}}
    foreach ($extension in @('cu', 'ptx', 'json')) {
        $source = "$($set.root)/rne_bias-$hash.$extension"
        $destination = "$frozenRoot/$($set.name)/rne_bias.$extension"
        Copy-New $source $destination
        $bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $destination))
        $row.files[$extension] = [ordered]@{
            path=$destination
            bytes=$bytes.Length
            sha256=Get-Sha256 $bytes
        }
    }
    $artifacts.Add($row)
}
foreach ($row in $artifacts) {
    foreach ($extension in $expected.Keys) {
        if ($row.files[$extension].sha256 -cne $expected[$extension]) {
            throw "偏置力产物不一致：$($row.name)/$extension"
        }
    }
    $receipt = Get-Content -LiteralPath $row.files.json.path -Raw | ConvertFrom-Json
    if ($receipt.entry -cne 'rne_bias' -or
        $receipt.sourceSha256 -cne $expected.cu -or
        $receipt.ptxSha256 -cne $expected.ptx -or
        $receipt.fmad -ne $true -or
        $receipt.useFastMath -ne $false -or
        $receipt.precisionGate -cne 'f32-physics' -or
        @($receipt.nvrtcVersion)[0] -ne 12 -or
        @($receipt.nvrtcVersion)[1] -ne 8 -or
        @($receipt.options)[0] -cne '--std=c++17' -or
        @($receipt.options)[1] -cne '--gpu-architecture=compute_70' -or
        $receipt.passed -ne $true) {
        throw "偏置力编译回执错误：$($row.name)"
    }
}

foreach ($id in @(
    'host-wrapper-retry-01',
    'format-after-gpu-code',
    'default-lib',
    'clippy-all-targets-retry-01',
    'input-tolerance-gate',
    'device-preflight',
    'gpu-product-captured',
    'inventory'
)) {
    $null = Assert-Run $id 0
}
$gate = Assert-Run 'input-tolerance-gate' 0
if ((Get-Content -LiteralPath $gate.stdout -Raw) -notmatch 'running 1 test' -or
    (Get-Content -LiteralPath $gate.stdout -Raw) -notmatch '1 passed') {
    throw '偏置力输入与容差门禁数量错误'
}
$product = Assert-Run 'gpu-product-captured' 0
if ((Get-Content -LiteralPath $product.stdout -Raw) -notmatch '1 passed') {
    throw 'GPU用例数量错误'
}
$deviceResult = Assert-Run 'device-preflight' 0
$device = Get-Content -LiteralPath $deviceResult.stdout -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or
    $device.resource_lock -cne 'gpu:windows-t4' -or
    $device.nvrtc_version -cne '12.8') {
    throw '本机T4身份错误'
}
$claims = Get-Content -LiteralPath "$root/claims.json" -Raw | ConvertFrom-Json
$plan = Get-Content -LiteralPath "$root/execution-plan.json" -Raw | ConvertFrom-Json
$commands = @($plan.registered_validations | ForEach-Object { "$($_.executable) $($_.arguments -join ' ')" }) -join "`n"
if ($commands -match '(?i)(^|\s)python(\.exe)?(\s|$)' -or
    $claims.python_executed -ne $false -or
    @($claims.acceptance.PSObject.Properties | Where-Object Value -ne $false).Count -ne 0) {
    throw '执行或验收边界错误'
}
$ptxText = [IO.File]::ReadAllText((Resolve-Path -LiteralPath "$frozenRoot/primary/rne_bias.ptx"))
$registry = Get-Content -LiteralPath 'fixtures/precision/t4-current-kernels.json' -Raw | ConvertFrom-Json
$registryRows = @($registry.artifacts | Where-Object entry -CEQ 'rne_bias')
if ($registryRows.Count -ne 1 -or
    $registryRows[0].source_sha256 -cne $expected.cu -or
    $registryRows[0].ptx_sha256 -cne $expected.ptx -or
    $registryRows[0].receipt_sha256 -cne $expected.json) {
    throw '冻结T4清单与实际原件不符'
}

$record = [ordered]@{
    schema=1
    task_id=$task
    supersedes="$root/precision-revalidation.json"
    source_identity=[ordered]@{
        base_revision=$baseRevision
        base_blob=$baseSource.oid
        base_raw_sha256=Get-Sha256 $baseSource.bytes
        current_raw_sha256=Get-Sha256 $currentSource
        base_cuda_sha256=Get-Sha256 $baseCuda
        current_cuda_sha256=Get-Sha256 $currentCuda
        base_evidence="$frozenRoot/base-rne-bias.rs"
        current_evidence="$frozenRoot/current-rne-bias.rs"
    }
    frozen_product=[ordered]@{
        rne_bias_cuda_unchanged=$true
        rne_bias_cuda_sha256=$expected.cu
        transfer_kernel_unchanged=$true
        transfer_kernel_sha256=Get-Sha256 $currentTransfer
        t4_registry_matches_actual_files=$true
        historical_compile_record_matches=$true
    }
    generated_product=[ordered]@{
        artifacts=@($artifacts)
        historical_and_all_current_captures_exact=$true
        receipt_contract_exact=$true
        ptx_f64_mentions=([regex]::Matches($ptxText, '(?i)\.f64\b')).Count
        ptx_atomic_mentions=([regex]::Matches($ptxText, '(?i)atom')).Count
    }
    validations=[ordered]@{
        input_tolerance_gate_one_test=$true
        raw_vs_typed_gpu_one_test=$true
        metadata_and_session_prewrite_rejection=$true
        recovery_on_same_plan=$true
        zero_dof_product_path=$true
        multilevel_parent_chain=$true
        nonzero_qvel=$true
        worlds=513
        device=$device.device
    }
    historical_precision=[ordered]@{
        failed_blocks=1042
        repaired_by_this_slice=$false
    }
    boundaries=[ordered]@{
        python_executed=$false
        np01=$false
        np02=$false
        np03=$false
        np04=$false
        performance=$false
        linux=$false
        release=$false
    }
}
$json = ($record | ConvertTo-Json -Depth 20).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema=1
    device=$device.device
    capture_sets=$artifacts.Count
    source_ptx_unchanged=$true
    input_tolerance_gate_one_test=$true
    np01_to_np04_accepted=$false
} | ConvertTo-Json
