$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-sleep-copy'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-sleep-copy'
$priorPath = 'docs/evidence/t4-g03-retest/tendon-wake-internal/map.json'
$outputPath = "$root/precision-revalidation-v2.json"
$baseSourcePath = "$root/base-resident.rs"
$currentSourcePath = "$root/current-resident.rs"
foreach ($path in @($outputPath, $baseSourcePath, $currentSourcePath)) {
    if (Test-Path -LiteralPath $path) { throw "精度原件已存在：$path" }
}

function Get-Sha256([byte[]]$Bytes) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function Get-LfBytes([byte[]]$Bytes) {
    [Text.Encoding]::UTF8.GetBytes([Text.Encoding]::UTF8.GetString($Bytes).Replace("`r`n", "`n"))
}

function Read-CheckedSource([object]$Row) {
    $bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Row.path))
    $lf = Get-LfBytes $bytes
    if ((Get-Sha256 $lf) -cne $Row.sha256) { throw "来源摘要漂移：$($Row.path)" }
    [Text.Encoding]::UTF8.GetString($lf)
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
    $absolute = Join-Path (Get-Location) $Path
    $stream = [IO.FileStream]::new($absolute, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $stream.Write($Bytes, 0, $Bytes.Length) } finally { $stream.Dispose() }
}

function Get-Entry([string]$Ptx, [string]$Name) {
    $marker = ".visible .entry $Name("
    $start = $Ptx.IndexOf($marker, [StringComparison]::Ordinal)
    if ($start -lt 0) { throw "PTX缺少入口：$Name" }
    $tail = $Ptx.Substring($start + $marker.Length)
    $next = $tail.IndexOf('.visible .entry ', [StringComparison]::Ordinal)
    if ($next -ge 0) { $tail.Substring(0, $next) } else { $tail }
}

function Get-Ops([string]$Body) {
    @($Body.Replace("`r`n", "`n").Split("`n") | ForEach-Object {
        $line = $_.Trim()
        if ($line.Length -eq 0 -or $line.StartsWith('//')) { return }
        ,@($line -split '[\s,;]+' | Where-Object Length -gt 0)
    })
}

$prior = Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json
if ($prior.groups.Count -ne 18 -or $prior.outputs.Count -ne 5) { throw '旧映射规模漂移' }
$base = Read-GitBlob '15e31190182dd7c58312424851d477d9495b24d6' 'src/physics/resident.rs'
$currentBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath 'src/physics/resident.rs'))
Write-NewBytes $baseSourcePath $base.bytes
Write-NewBytes $currentSourcePath $currentBytes
$baseLf = Get-LfBytes $base.bytes
$currentLf = Get-LfBytes $currentBytes
if ((Get-Sha256 $baseLf) -cne '6389af92c39b687cdf3cd01fa3855d651bf264d6df7e2ff52fd8f230950718eb') {
    throw '基线resident对象不匹配旧门禁'
}

$sources = [Collections.Generic.List[string]]::new()
for ($i = 0; $i -lt $prior.sources.Count; $i++) {
    if ($i -eq 2) {
        $sources.Add([Text.Encoding]::UTF8.GetString($currentLf))
    } else {
        $sources.Add((Read-CheckedSource $prior.sources[$i]))
    }
}
$groups = [Collections.Generic.List[object]]::new()
$ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($row in $prior.groups) {
    if (-not $ids.Add($row.id)) { throw "重复精度组：$($row.id)" }
    $sourceIndex = [int]$row.source
    if (-not $sources[$sourceIndex].Contains([string]$row.product)) { throw "产品锚点缺失：$($row.id)" }
    $upstreamChecked = $false
    if ($null -ne $row.upstream) {
        if (-not $sources[4].Contains([string]$row.upstream)) { throw "上游锚点缺失：$($row.id)" }
        $upstreamChecked = $true
    }
    $groups.Add([ordered]@{
        id = $row.id
        scalar = $row.scalar
        product_source = $prior.sources[$sourceIndex].path
        product_anchor = $row.product
        product_checked = $true
        upstream_anchor = $row.upstream
        upstream_checked = $upstreamChecked
    })
}
if ($groups.Count -ne 18) { throw '精度组复核数量错误' }

$outputs = [Collections.Generic.List[object]]::new()
foreach ($row in $prior.outputs) {
    $declaration = if ($row.name.StartsWith('tree_')) { "pub $($row.name): &'a [i32]" } else { "pub $($row.name): i32" }
    if (-not $sources[0].Contains($declaration) -or
        -not $sources[5].Contains([string]$row.upstream) -or $row.scalar -cne 'i32') {
        throw "输出映射错误：$($row.name)"
    }
    $outputs.Add([ordered]@{
        name=$row.name
        scalar=$row.scalar
        product_declaration=$declaration
        upstream_declaration=$row.upstream
        checked=$true
    })
}
if ($outputs.Count -ne 5 -or
    -not $sources[1].Contains('pub const TREE_FULLY_AWAKE: i32 = -11;') -or
    -not $sources[4].Contains('K_AWAKE_VAL = -(1 + types.MJ_MINAWAKE)') -or
    -not $sources[5].Contains('MJ_MINAWAKE = mujoco.mjMINAWAKE')) {
    throw '整数输出或唤醒常量映射错误'
}

$captureRoot = "$runtime/run-01-product-captured/gpu-capture"
$sourceSha = '90f06a049e10a979e44f8d067ec20a9633ab9dc3ff0a3f2698f787478774f7c3'
$ptxSha = '0f0740d3df05b413ea24faf778f76e32e651183ade60c45b2dc324f97e50d2ff'
$template = $sources[0].Split('const TENDON_WAKE_CUDA: &str = r#"', 2)[1].Split('"#;', 2)[0]
$ptxChecks = [Collections.Generic.List[object]]::new()
foreach ($name in @('sleep_copy', 'tendon_wake')) {
    $prefix = Join-Path $captureRoot "$name-$ptxSha"
    $cudaBytes = [IO.File]::ReadAllBytes("$prefix.cu")
    $ptxBytes = [IO.File]::ReadAllBytes("$prefix.ptx")
    $receiptBytes = [IO.File]::ReadAllBytes("$prefix.json")
    if ((Get-Sha256 $cudaBytes) -cne $sourceSha -or (Get-Sha256 $ptxBytes) -cne $ptxSha) {
        throw "当前捕获摘要漂移：$name"
    }
    $cuda = [Text.Encoding]::UTF8.GetString($cudaBytes)
    if (-not $cuda.Contains($template)) { throw "捕获源码缺少内核模板：$name" }
    $ptx = [Text.Encoding]::UTF8.GetString($ptxBytes)
    $ops = @(Get-Ops (Get-Entry $ptx $name))
    if (-not @($ops | Where-Object { $_[0] -ceq 'ld.global.u32' }).Count -or
        -not @($ops | Where-Object { $_[0] -ceq 'st.global.u32' }).Count) {
        throw "PTX缺少整数传输：$name"
    }
    foreach ($op in $ops) {
        if (@($op[0].Split('.') | Where-Object { $_ -in @('f64','f16','f16x2','bf16','bf16x2','tf32') }).Count) {
            throw "PTX出现禁止精度：$name/$($op[0])"
        }
        if ($op[0].Split('.') -contains 'f32') {
            if ($op[0] -notin @('ld.global.f32','sub.f32','setp.lt.f32','setp.geu.f32') -or $name -cne 'tendon_wake') {
                throw "PTX浮点指令错误：$name/$($op[0])"
            }
        }
    }
    if ($name -ceq 'tendon_wake') {
        $subtract = @($ops | Where-Object { $_[0] -ceq 'sub.f32' })
        $low = @($ops | Where-Object { $_[0] -ceq 'setp.lt.f32' })
        $high = @($ops | Where-Object { $_[0] -ceq 'setp.geu.f32' })
        if ($subtract.Count -ne 2 -or $low.Count -ne 1 -or $high.Count -ne 1 -or
            $subtract[0][2] -cne $subtract[1][3] -or $low[0][2] -cne $subtract[0][1] -or
            $high[0][2] -cne $subtract[1][1] -or $low[0][3] -cne $high[0][3] -or
            @($ops | Where-Object { $_[0] -ceq 'ld.global.f32' }).Count -ne 4 -or
            @($ops | Where-Object { $_[0] -ceq 'rem.u32' }).Count -ne 2 -or
            -not @($ops | Where-Object { $_[0] -ceq 'min.s32' }).Count -or
            -not @($ops | Where-Object { $_[0] -ceq 'mov.u32' -and $_[-1] -ceq '-11' }).Count -or
            -not @($ops | Where-Object { $_[0] -ceq 'setp.lt.s32' -and $_[-1] -ceq '0' }).Count -or
            @($ops | Where-Object { $_[0].StartsWith('atom.') }).Count) {
            throw 'tendon_wake PTX语义映射错误'
        }
    }
    $receipt = [Text.Encoding]::UTF8.GetString($receiptBytes) | ConvertFrom-Json
    if ($receipt.entry -cne $name -or $receipt.sourceSha256 -cne $sourceSha -or
        $receipt.ptxSha256 -cne $ptxSha -or $receipt.passed -ne $true -or
        $receipt.useFastMath -ne $false) {
        throw "编译回执错误：$name"
    }
    $ptxChecks.Add([ordered]@{
        entry=$name
        cuda_sha256=(Get-Sha256 $cudaBytes)
        ptx_sha256=(Get-Sha256 $ptxBytes)
        receipt_sha256=(Get-Sha256 $receiptBytes)
        integer_load_store=$true
        forbidden_precision_absent=$true
        float_instruction_contract=$true
    })
}

$resident = $sources[2]
$update = $resident.Split('pub fn update_tendon_wake(', 2)[1].Split('pub fn ', 2)[0]
$copyIndex = $update.IndexOf('device.copy_kernel.launch(', [StringComparison]::Ordinal)
$wakeIndex = $update.IndexOf('device.wake_kernel.launch(', [StringComparison]::Ordinal)
$swapIndex = $update.IndexOf('std::mem::swap(&mut out.state, &mut out.scratch);', [StringComparison]::Ordinal)
$readback = $resident.Split('fn read_output<T:', 2)[1].Split("`n}", 2)[0]
if ($copyIndex -lt 0 -or $copyIndex -ge $wakeIndex -or $wakeIndex -ge $swapIndex -or
    -not $update.Contains('data.ready.require(data.ready.tendon, "tendon")?;') -or
    -not $resident.Contains('state: TransferBuffer<i32>') -or
    -not $resident.Contains('scratch: TransferBuffer<i32>') -or
    -not $resident.Contains('.write_range(4 + range.start, &state.packed())?;') -or
    -not $readback.Contains('Result<Vec<T>, TransferError>') -or
    -not $readback.Contains('Ok(values)') -or
    -not $sources[3].Contains('if !(*value).into().is_finite()') -or
    -not $sources[0].Contains('if(!m[6]||nt==0) return;') -or
    $sources[0].IndexOf('awake[i]=asleep[i]<0', [StringComparison]::Ordinal) -le
        $sources[0].IndexOf('for(int t=0;t<nt;++t)', [StringComparison]::Ordinal)) {
    throw '事务发布或诊断读回映射错误'
}

$registrationPath = "$runtime/registrations/gpu-product-captured.json"
$resultPath = "$runtime/run-01-product-captured/result.json"
$registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
$result = Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json
if ($result.exit_code -ne 0 -or $result.registration_sha256 -cne
    (Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()) {
    throw 'GPU产品运行未绑定登记'
}

$record = [ordered]@{
    schema = 2
    task = '2026-10-19-layers-m3-sleep-copy'
    prior_map = [ordered]@{
        path=$priorPath
        sha256=(Get-FileHash -LiteralPath $priorPath -Algorithm SHA256).Hash.ToLowerInvariant()
        groups=@($groups)
        outputs=@($outputs)
    }
    resident_identity = [ordered]@{
        base_revision='15e31190182dd7c58312424851d477d9495b24d6'
        base_blob=$base.oid
        base_raw_sha256=Get-Sha256 $base.bytes
        base_lf_sha256=Get-Sha256 $baseLf
        base_evidence=$baseSourcePath
        current_raw_sha256=Get-Sha256 $currentBytes
        current_lf_sha256=Get-Sha256 $currentLf
        current_evidence=$currentSourcePath
        old_identity_gate_passed=$false
    }
    generated_product = [ordered]@{
        checks=@($ptxChecks)
        capture_registration=$registrationPath
        capture_registration_sha256=(Get-FileHash -LiteralPath $registrationPath -Algorithm SHA256).Hash.ToLowerInvariant()
        capture_result=$resultPath
        capture_result_sha256=(Get-FileHash -LiteralPath $resultPath -Algorithm SHA256).Hash.ToLowerInvariant()
        second_variant_exact_match=$true
    }
    transaction = [ordered]@{
        order=@('sleep_copy','tendon_wake','swap')
        ready_dependency_checked=$true
        state_and_scratch_i32=$true
        diagnostic_conversion_only=$true
    }
    boundaries = [ordered]@{
        old_map_modified=$false
        old_test_modified=$false
        host_migration_rule='sourceSha256 and ptxSha256 unchanged'
        local_slice_evidence='pass'
        np01_complete=$false
        np02_complete=$false
        np03_complete=$false
        np04_complete=$false
        performance_accepted=$false
        linux_run=$false
    }
}
$json = ($record | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema=2
    checked_groups=$groups.Count
    checked_outputs=$outputs.Count
    checked_entries=$ptxChecks.Count
    base_blob=$base.oid
    old_identity_gate_passed=$false
    local_slice_evidence='pass'
} | ConvertTo-Json
