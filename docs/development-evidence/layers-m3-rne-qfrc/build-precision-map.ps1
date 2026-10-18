$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-qfrc'
$root = "docs/development-evidence/layers-m3-rne-qfrc"
$runtime = "target/normify-evidence/$task"
$baseRevision = '40aac80d4ce361949b70708722eddfe2c0f3bfc8'
$outputPath = "$root/precision-revalidation.json"
$frozenRoot = "$root/frozen"
if (Test-Path -LiteralPath $outputPath) { throw "精度映射已存在：$outputPath" }
if (Test-Path -LiteralPath $frozenRoot) { throw "冻结目录已存在：$frozenRoot" }
New-Item -ItemType Directory -Path $frozenRoot | Out-Null

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

$baseRne = Read-GitBlob $baseRevision 'src/physics/rne.rs'
$currentRne = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath 'src/physics/rne.rs'))
Write-NewBytes "$frozenRoot/base-rne.rs" $baseRne.bytes
Write-NewBytes "$frozenRoot/current-rne.rs" $currentRne
Copy-New 'src/physics/rne.cu' "$frozenRoot/current-rne.cu"
Copy-New 'docs/evidence/rne-acceleration/map.json' "$frozenRoot/historical-map.json"

foreach ($path in @('src/physics/rne.cu', 'src/runtime/transfer/kernel.rs', 'docs/evidence/rne-acceleration/map.json')) {
    & git diff --quiet $baseRevision -- $path
    if ($LASTEXITCODE -ne 0) { throw "冻结产品或证据发生变化：$path" }
}
$baseText = [Text.Encoding]::UTF8.GetString($baseRne.bytes)
$currentText = [Text.Encoding]::UTF8.GetString($currentRne)
$includeLine = 'const RNE_CUDA: &str = include_str!("rne.cu");'
if (-not $baseText.Contains($includeLine) -or -not $currentText.Contains($includeLine)) {
    throw 'RNE_CUDA绑定发生变化'
}

$hash = 'a5b528ff34301f8e358d13a53b5dfbd69676fcc02987d39a3148d9693b8ebd22'
$historicalRoot = 'target/rne-acceleration-validation/attempt-3/kernels'
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
        $source = "$($set.root)/rne_qfrc-$hash.$extension"
        $destination = "$frozenRoot/$($set.name)/rne_qfrc.$extension"
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
$expectedHashes = [ordered]@{
    cu='cf624cc043591268e1b8565ce150e8dc8e08d9a23a44c68087ed20416d3df17e'
    ptx=$hash
    json='559db5841ed49772a529a772d39398a0433d17b670a6f42841896f2211b3c13c'
}
foreach ($row in $artifacts) {
    foreach ($extension in $expectedHashes.Keys) {
        if ($row.files[$extension].sha256 -cne $expectedHashes[$extension]) {
            throw "qfrc产物不一致：$($row.name)/$extension"
        }
    }
    $receipt = Get-Content -LiteralPath $row.files.json.path -Raw | ConvertFrom-Json
    if ($receipt.entry -cne 'rne_qfrc' -or
        $receipt.sourceSha256 -cne $expectedHashes.cu -or
        $receipt.ptxSha256 -cne $expectedHashes.ptx -or
        $receipt.fmad -ne $true -or
        $receipt.useFastMath -ne $false -or
        $receipt.precisionGate -cne 'f32-physics' -or
        @($receipt.nvrtcVersion)[0] -ne 12 -or
        @($receipt.nvrtcVersion)[1] -ne 8 -or
        @($receipt.options)[0] -cne '--std=c++17' -or
        @($receipt.options)[1] -cne '--gpu-architecture=compute_70' -or
        $receipt.passed -ne $true) {
        throw "qfrc编译回执错误：$($row.name)"
    }
}

$invalid = Assert-Run 'old-identity-gate' 0
$invalidStdout = Get-Content -LiteralPath $invalid.stdout -Raw
if ($invalidStdout -notmatch 'running 0 tests' -or $invalidStdout -notmatch '0 passed') {
    throw '旧误标运行没有保留零用例事实'
}
$corrected = Assert-Run 'old-input-gate-retry-01' 0
$correctedStdout = Get-Content -LiteralPath $corrected.stdout -Raw
if ($correctedStdout -notmatch 'running 1 test' -or $correctedStdout -notmatch '1 passed') {
    throw '旧输入与容差门禁没有执行一项'
}
foreach ($id in @('host-wrapper-retry-01', 'default-lib', 'clippy-retry-01', 'format-final', 'device-preflight', 'gpu-product-captured', 'gpu-guards')) {
    $null = Assert-Run $id 0
}
$product = Assert-Run 'gpu-product-captured' 0
$guards = Assert-Run 'gpu-guards' 0
if ((Get-Content -LiteralPath $product.stdout -Raw) -notmatch '1 passed' -or
    (Get-Content -LiteralPath $guards.stdout -Raw) -notmatch '1 passed') {
    throw 'GPU用例数量错误'
}
$deviceResult = Assert-Run 'device-preflight' 0
$device = Get-Content -LiteralPath $deviceResult.stdout -Raw | ConvertFrom-Json
if ($device.device -notmatch '^NVIDIA GeForce RTX 4070 Ti SUPER,' -or
    $device.nvrtc_version -cne '12.8') {
    throw '本机T4身份错误'
}

$oldMap = Get-Content -LiteralPath 'docs/evidence/rne-acceleration/map.json' -Raw | ConvertFrom-Json
$currentLfHash = Get-Sha256 (Get-LfBytes $currentRne)
if ($oldMap.sources[0].path -cne 'src/physics/rne.rs' -or
    $oldMap.sources[0].sha256 -ceq $currentLfHash -or
    $oldMap.sources[1].sha256 -cne $expectedHashes.cu -or
    $oldMap.precision.full_operator_compiler_equivalence -ne $false) {
    throw '历史映射边界错误'
}
$claims = Get-Content -LiteralPath "$root/claims.json" -Raw | ConvertFrom-Json
$plan = Get-Content -LiteralPath "$root/execution-plan.json" -Raw | ConvertFrom-Json
$commands = @($plan.registered_validations | ForEach-Object { "$($_.executable) $($_.arguments -join ' ')" }) -join "`n"
if ($commands -match '(?i)(^|\s)python(\.exe)?(\s|$)' -or
    $claims.python_executed -ne $false -or
    @($claims.acceptance.PSObject.Properties | Where-Object Value -ne $false).Count -ne 0) {
    throw '执行或验收边界错误'
}

$record = [ordered]@{
    schema=1
    task_id=$task
    source_identity=[ordered]@{
        base_revision=$baseRevision
        base_blob=$baseRne.oid
        base_raw_sha256=Get-Sha256 $baseRne.bytes
        base_lf_sha256=Get-Sha256 (Get-LfBytes $baseRne.bytes)
        current_raw_sha256=Get-Sha256 $currentRne
        current_lf_sha256=$currentLfHash
        base_evidence="$frozenRoot/base-rne.rs"
        current_evidence="$frozenRoot/current-rne.rs"
        old_map_source_identity_matches_current=$false
    }
    frozen_product=[ordered]@{
        rne_cuda_unchanged=$true
        rne_cuda_sha256=$expectedHashes.cu
        rne_cuda_evidence="$frozenRoot/current-rne.cu"
        transfer_kernel_unchanged=$true
        rne_cuda_include_binding_unchanged=$true
    }
    generated_product=[ordered]@{
        artifacts=@($artifacts)
        historical_and_all_current_captures_exact=$true
        receipt_contract_exact=$true
    }
    validations=[ordered]@{
        invalid_old_identity_label_zero_tests=$true
        corrected_input_tolerance_gate_one_test=$true
        raw_vs_typed_gpu_one_test=$true
        guard_gpu_one_test=$true
        device=$device.device
    }
    historical_precision=[ordered]@{
        flg_acc_false_failed_blocks=1042
        max_normalized_error=2.762533
        tolerance='2e-5 + 2e-5 * abs(reference)'
        repaired_by_this_slice=$false
        full_operator_compiler_equivalence=$false
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
    invalid_empty_run_preserved=$true
    np01_to_np04_accepted=$false
} | ConvertTo-Json
