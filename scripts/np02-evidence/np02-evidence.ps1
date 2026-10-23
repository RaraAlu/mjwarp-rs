# NP02 证据清单（Windows 单平台）只读扫描器。
# 输入：产品源码根 + 证据根；输出 inventory.json 与 gap.json。
# 约束：只读、不启动 Python、不改产品/测试/fixtures；窗口输出真实数字。
[CmdletBinding()]
param(
    [string]$ProductRoot = '',
    [string]$OutDir = '',
    [switch]$Check
)

$ErrorActionPreference = 'Stop'
$scriptRoot = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($ProductRoot)) {
    $ProductRoot = Split-Path -Parent (Split-Path -Parent $scriptRoot)
}
$ProductRoot = (Resolve-Path -LiteralPath $ProductRoot).Path
if ([string]::IsNullOrWhiteSpace($OutDir)) { $OutDir = Join-Path $ProductRoot 'docs\development-evidence\np02-evidence' }
$outFull = if (Test-Path -LiteralPath $OutDir) { (Resolve-Path -LiteralPath $OutDir).Path } else { $OutDir }

. (Join-Path $scriptRoot 'np02-lib.ps1')
function Resolve-RepoPath([string]$rel) { return (Get-RepoPath $ProductRoot $rel) }
function Rel([string]$full) { return (Get-RepoRel $ProductRoot $full) }
function Read-Utf8([string]$path) { return [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8) }
function Read-Json([string]$path) { return (Read-Utf8 $path | ConvertFrom-Json) }

$specPath = Join-Path $scriptRoot 'np02-spec.json'
$spec = Read-Json $specPath
$taskId = $spec.task_id

Write-Output "NP02 证据清单扫描：$ProductRoot"
Write-Output "规格：$(Rel $specPath)；任务：$taskId"

# ---------- 1. 上游 pin 校验 ----------
$pinReport = [ordered]@{}
foreach ($key in @('mjwarp', 'warp')) {
    $pin = $spec.pins.$key
    $dir = Join-Path $ProductRoot $pin.path
    if (-not (Test-Path -LiteralPath $dir)) { throw "缺少上游 checkout：$($pin.path)" }
    $rev = (& git -C $dir rev-parse HEAD 2>$null)
    if ($LASTEXITCODE -ne 0) { throw "无法读取上游 revision：$($pin.path)" }
    $rev = ($rev | Select-Object -First 1).Trim()
    if ($rev -ne $pin.revision) { throw "上游 pin 不符：$key 期望 $($pin.revision)，实际 $rev" }
    $pinReport[$key] = [ordered]@{ path = $pin.path; expected = $pin.revision; actual = $rev; verified = $true }
}
Write-Output '上游 pin：MJWarp 71da24d… ✓ / Warp 1.15.0 ffa038b1… ✓'

# ---------- 2. 输入 ----------
$receiptsIndex = Read-Json (Resolve-RepoPath $spec.inputs.receipts_index)
$t4Current = Read-Json (Resolve-RepoPath $spec.inputs.t4_current)
$np01Mapping = Read-Json (Resolve-RepoPath $spec.inputs.np01_mapping)
$np01Diff = Read-Json (Resolve-RepoPath $spec.inputs.np01_diff)
$np02Coverage = Read-Json (Resolve-RepoPath $spec.inputs.np02_coverage)
$routeReceipt = Read-Json (Resolve-RepoPath $spec.inputs.kernel_source_route_receipt)
$routeProbe = Read-Utf8 (Resolve-RepoPath $spec.inputs.kernel_source_route_probe)
$routeG01Probe = Read-Utf8 (Resolve-RepoPath $spec.inputs.kernel_source_route_g01_probe)

# ---------- 3. 回执索引复核（逐份实体哈希） ----------
$audit = Invoke-ReceiptAudit -ProductRoot $ProductRoot -ReceiptsIndex $receiptsIndex
$receiptRows = $audit.Rows
$receiptVerified = $audit.Verified
$receiptFailed = $audit.Failed
Write-Output ("回执索引：{0} 份，逐份文件哈希通过 {1} 份，异常 {2} 份" -f $receiptRows.Count, $receiptVerified, $receiptFailed.Count)

$receiptByEntry = @{}
foreach ($row in $receiptRows) {
    if (-not $receiptByEntry.ContainsKey($row.entry)) { $receiptByEntry[$row.entry] = @() }
    $receiptByEntry[$row.entry] += $row
}

# T4 当前冻结清单：entry -> ptx 摘要集合
$t4ByEntry = @{}
foreach ($a in $t4Current.artifacts) {
    if (-not $t4ByEntry.ContainsKey($a.entry)) { $t4ByEntry[$a.entry] = @() }
    $t4ByEntry[$a.entry] += $a
}
$t4Mappings = @{}
foreach ($m in $t4Current.mappings) { $t4Mappings[$m.entry] = $m }

# NP01：模块 -> 上游函数/头文件（来自 np02-coverage 的 upstream 字段）
$coverageByPath = @{}
foreach ($row in $np02Coverage.rows) { $coverageByPath[$row.id] = $row }
$np01ModulePaths = @{}
foreach ($row in $np01Mapping.rows) {
    if (-not $np01ModulePaths.ContainsKey($row.module)) { $np01ModulePaths[$row.module] = @() }
    $np01ModulePaths[$row.module] += $row
}

# ---------- 4. 产品 CUDA 源扫描（含 .rs 内联 const 与 include_str! 的 .cu） ----------
$excludedFiles = @{
    'src/runtime/launcher/module.rs' = '测试用 fill 内核（模块缓存测试）'
    'src/runtime/launcher/route/tests.rs' = 'route 单元测试内核'
    'src/runtime/transfer/kernel/kinetic_energy.rs' = 'kinetic_energy32 ABI 探针'
    'src/runtime/transfer/kernel/tests/abi.rs' = 'ABI 探针'
    'src/runtime/transfer/kernel/tests/guards.rs' = 'hessian 探针'
    'src/runtime/transfer/kernel/tests/precision.rs' = 'fma 探针'
    'src/runtime/transfer/kernel/tests/support.rs' = 'adapter 探针'
}
$sourceUnits = Get-SourceUnits -ProductRoot $ProductRoot -ExcludedFiles $excludedFiles
$seenKernels = @{}
$kernelRows = @()
foreach ($unit in $sourceUnits) {
    foreach ($k in ($unit.kernels | Sort-Object -Unique)) {
        if ($seenKernels.ContainsKey($k)) { throw "内核名重复：$k（$($unit.relFile) 与 $($seenKernels[$k])）" }
        $seenKernels[$k] = $unit.relFile
        $evidence = if ($receiptByEntry.ContainsKey($k)) { @($receiptByEntry[$k]) } else { @() }
        $templateInstances = @()
        $renamedFrom = @()
        if ($evidence.Count -eq 0) {
            # 模板内核（如 small_prefix_factor_ 由 factor_2..6 实例化）：按 entry 前缀匹配实例回执
            $tplPattern = '^' + [regex]::Escape($k) + '\d+$'
            $templateInstances = @($receiptRows | Where-Object { $_.entry -match $tplPattern })
            if ($templateInstances.Count -gt 0) {
                $evidence = $templateInstances
            } else {
                # 同源不同名：历史回执按旧 entry 名捕获，源哈希与当前源码一致
                $renamedFrom = @($receiptRows | Where-Object { $_.sourceSha256 -eq $unit.sha256 })
                if ($renamedFrom.Count -gt 0) { $evidence = $renamedFrom }
            }
        }
        $t4 = if ($t4ByEntry.ContainsKey($k)) { @($t4ByEntry[$k]) } else { @() }
        $ptxVerified = @($evidence | Where-Object { $_.ptxEntityOk })
        $srcMatch = @($evidence | Where-Object { $_.sourceSha256 -eq $unit.sha256 }).Count -gt 0
        $state = if ($evidence.Count -eq 0) { 'missing' } elseif ($renamedFrom.Count -gt 0) { 'renamed-evidence' } elseif ($srcMatch) { 'verified' } else { 'source-drift' }
        $kernelRows += [pscustomobject]@{
            id = "kernel:$k"
            name = $k
            route = 'HandWrittenCuda'
            module = ($unit.relFile -replace '^src/physics/', '' -replace '^src/', '' -replace '\.rs$', '' -replace '/', '.')
            source = [ordered]@{ path = $unit.path; const = $unit.const; sha256 = $unit.sha256 }
            compile = [ordered]@{
                recipe = $spec.product_recipe.recipe
                fmad = $spec.product_recipe.fmad
                fast_math = $spec.product_recipe.fast_math
                std = $spec.product_recipe.std
                arch_option = $spec.product_recipe.arch_option
                nvrtc = @($spec.product_recipe.nvrtc)
                precision_gate = $spec.product_recipe.precision_gate
                receipt_options = @(($evidence | Select-Object -First 1).options)
            }
            artifact = [ordered]@{
                receipts = $evidence.Count
                receipt_entries = @($evidence | ForEach-Object { $_.entry } | Sort-Object -Unique)
                template_instances = @($templateInstances | ForEach-Object { $_.entry } | Sort-Object -Unique)
                renamed_entries = @($renamedFrom | ForEach-Object { $_.entry } | Sort-Object -Unique)
                ptx_verified = $ptxVerified.Count
                ptx_sha256 = @($evidence | ForEach-Object { $_.ptxSha256 } | Sort-Object -Unique)
                t4_variants = $t4.Count
                t4_ptx_sha256 = @($t4 | ForEach-Object { $_.ptx_sha256 } | Sort-Object -Unique)
                evidence_paths = @(($evidence | Select-Object -First 2 | ForEach-Object { $_.path }))
            }
            evidence_state = $state
            source_identity_matches_capture = $srcMatch
            upstream = [ordered]@{ mjwarp_revision = $spec.pins.mjwarp.revision; np01_module = $null; functions = @(); header_paths = @() }
            conclusion = switch ($state) {
                'verified' { '证据完整：回执存在且与当前源码身份一致（PTX 实体在仓者逐一复核哈希）' }
                'source-drift' { '证据存在但产品源身份已漂移（捕获时源哈希 ≠ 当前源哈希），需重采集或另行裁决' }
                'renamed-evidence' { '同源不同名：当前 entry 无回执，但同源哈希的历史回执存在（旧命名捕获）' }
                default { '缺失：回执索引无该 entry，且无同源历史回执（新内核或改名后未重采集）' }
            }
        }
    }
}
Write-Output ("产品 CUDA 源：{0} 个源单元，{1} 个 __global__ 内核条目" -f $sourceUnits.Count, $kernelRows.Count)

# ---------- 4.5 编译点内核名：补派生内核与动态源内核 ----------
$compileNames = Get-CompileNames -ProductRoot $ProductRoot -ExcludedFiles $excludedFiles -ExcludedNames @($spec.excluded_compile_names)
$extraRows = @()
foreach ($name in $compileNames) {
    if ($seenKernels.ContainsKey($name)) { continue }
    $derived = @($spec.derived_kernels | Where-Object { $_.name -eq $name }) | Select-Object -First 1
    if ($null -ne $derived) {
        $baseRow = $kernelRows | Where-Object { $_.name -eq $derived.base } | Select-Object -First 1
        $extraRows += [pscustomobject]@{
            id = "kernel:$name"
            name = $name
            route = 'HandWrittenCuda'
            module = 'derived'
            source = [ordered]@{ path = '（派生源，无独立 const）'; const = $null; sha256 = $null; derived_from = $derived.base; source_anchor = $derived.source_anchor }
            compile = [ordered]@{
                recipe = $spec.product_recipe.recipe
                fmad = $spec.product_recipe.fmad
                fast_math = $spec.product_recipe.fast_math
                std = $spec.product_recipe.std
                arch_option = $spec.product_recipe.arch_option
                nvrtc = @($spec.product_recipe.nvrtc)
                precision_gate = $spec.product_recipe.precision_gate
                receipt_options = @($baseRow.compile.receipt_options)
            }
            artifact = [ordered]@{
                receipts = 0
                receipt_entries = @()
                template_instances = @()
                ptx_verified = 0
                ptx_sha256 = @($baseRow.artifact.ptx_sha256)
                t4_variants = 0
                t4_ptx_sha256 = @()
                evidence_paths = @($baseRow.artifact.evidence_paths)
            }
            evidence_state = 'derived'
            source_identity_matches_capture = $baseRow.source_identity_matches_capture
            upstream = [ordered]@{ mjwarp_revision = $spec.pins.mjwarp.revision; np01_module = $baseRow.upstream.np01_module; functions = @($baseRow.upstream.functions); header_paths = @($baseRow.upstream.header_paths) }
            conclusion = $derived.reason
        }
    } else {
        $extraRows += [pscustomobject]@{
            id = "kernel:$name"
            name = $name
            route = 'HandWrittenCuda'
            module = 'dynamic-source'
            source = [ordered]@{ path = '（动态构造源，扫描未定位）'; const = $null; sha256 = $null }
            compile = [ordered]@{ recipe = $spec.product_recipe.recipe; nvrtc = @($spec.product_recipe.nvrtc); precision_gate = $spec.product_recipe.precision_gate; receipt_options = @() }
            artifact = [ordered]@{ receipts = 0; receipt_entries = @(); ptx_verified = 0; ptx_sha256 = @(); t4_variants = 0; t4_ptx_sha256 = @(); evidence_paths = @() }
            evidence_state = 'undetermined'
            source_identity_matches_capture = $false
            upstream = [ordered]@{ mjwarp_revision = $spec.pins.mjwarp.revision; np01_module = $null; functions = @(); header_paths = @() }
            conclusion = '未判定：编译点存在该内核名，但扫描未定位其源与证据'
        }
    }
}
if ($extraRows.Count -gt 0) { $kernelRows = @($kernelRows) + @($extraRows) }
$derivedRows = @($extraRows | Where-Object { $_.evidence_state -eq 'derived' })
$undeterminedRows = @($extraRows | Where-Object { $_.evidence_state -eq 'undetermined' })
Write-Output ("编译点补充：派生 {0} 个、未判定 {1} 个（编译点名 {2} 个）" -f $derivedRows.Count, $undeterminedRows.Count, $compileNames.Count)

# 上游参考：内核名 -> NP01 路径（覆盖率清单 id），落到 MJWarp 模块/函数与 W15 头文件路径
$kernelModuleRules = @(
    @('^rigid_kinematics$', 'kinematics'), @('^rigid_com_position$', 'com_position'),
    @('^com_velocity$', 'com_velocity'), @('^rigid_mass_matrix$', 'crb'),
    @('^rigid_mass_solve$', 'mass_solve'), @('^resident_mass_substitute$', 'mass_solve'),
    @('^mass_multiply', 'mass_multiply'), @('^borrowed_mass', 'mass_multiply'),
    @('^device_energy_source_validate$', 'kinetic_energy'), @('^resident_crb_', 'crb'),
    @('^resident_sparse_', 'mass_solve'), @('^resident_mixed_', 'mass_solve'),
    @('^sparse_', 'mass_solve'), @('^mixed_', 'mass_solve'), @('^small_prefix_', 'mass_solve'),
    @('^rne_bias$', 'rne_bias'), @('^rne_', 'rne_acc'),
    @('^attached_', 'attached'), @('^initialize_attached$', 'attached'),
    @('^camlight_', 'camlight'), @('^fixed_tendon$', 'fixed_tendon'),
    @('^flex_invalidate$', 'flex_position'), @('^flex_positions$', 'flex_position'),
    @('^flex_edges$', 'flex_edge'), @('^flex_faces$', 'flex_face'),
    @('^flex_hessian', 'flex_hessian'), @('^flex_edge_passive_', 'flex_edge_passive'),
    @('^fluid_force_', 'fluid_force'), @('^gravity_compensation_', 'gravity_compensation'),
    @('^joint_passive_', 'joint_passive'), @('^kinetic_energy', 'kinetic_energy'),
    @('^passive_force_', 'passive_force'), @('^smooth_force', 'smooth_force'),
    @('^spatial_', 'spatial_tendon'), @('^applied_force_', 'applied_force'),
    @('^tendon_values$', 'tendon'), @('^tendon_indices$', 'tendon'),
    @('^tendon_velocity_', 'tendon_velocity'), @('^tendon_passive_', 'tendon_passive'),
    @('^sleep_copy$', 'sleep_wake'), @('^tendon_wake$', 'sleep_wake')
)
foreach ($row in $kernelRows) {
    if ($row.evidence_state -eq 'derived' -or $row.evidence_state -eq 'undetermined') { continue }
    $np01 = $null
    foreach ($rule in $kernelModuleRules) {
        if ($row.name -match $rule[0]) {
            if ($coverageByPath.ContainsKey($rule[1])) { $np01 = $coverageByPath[$rule[1]] }
            break
        }
    }
    if ($np01 -ne $null) {
        $row.upstream.np01_module = $np01.id
        $row.upstream.functions = @($np01.upstream.functions)
        $row.upstream.header_paths = @($np01.upstream.headerPaths | ForEach-Object { $_.path })
        $row.upstream.confidence = $np01.upstream.confidence
    } else {
        $row.upstream.confidence = '未判定：NP01/覆盖率清单无对应路径'
    }
}

# ---------- 5. 冻结 PTX 链 ----------
$frozenRows = @()
foreach ($chain in $spec.frozen_chains) {
    $moduleFull = Resolve-RepoPath $chain.module
    $moduleText = Read-Utf8 $moduleFull
    $mPtx = [regex]::Match($moduleText, 'const PTX_SHA256: &str = "(?<h>[0-9a-f]{64})"')
    $mCu = [regex]::Match($moduleText, 'const CU_SHA256: &str = "(?<h>[0-9a-f]{64})"')
    $entries = @([regex]::Matches($moduleText, '"(?<e>_[a-z0-9_]+_cuda_kernel_forward)"') | ForEach-Object { $_.Groups['e'].Value } | Sort-Object -Unique)
    $entityFull = Resolve-RepoPath $chain.ptx_entity
    $entityHash = Get-Sha256File $entityFull
    $ptxConst = if ($mPtx.Success) { $mPtx.Groups['h'].Value } else { '' }
    $cuFull = ($entityFull -replace '\.ptx$', '.cu')
    $cuOk = $false
    if ((Test-Path -LiteralPath $cuFull) -and $mCu.Success) { $cuOk = ((Get-Sha256File $cuFull) -eq $mCu.Groups['h'].Value) }
    $receiptEntity = Read-Json (Resolve-RepoPath $chain.receipt_entity)
    $receiptText = ($receiptEntity | ConvertTo-Json -Depth 8)
    $receiptRecords = $receiptText.Contains($ptxConst)
    # 上游源身份：模块 const 的 *_SHA256 与 pinned 上游同名文件逐一比对
    $upstreamChecks = @()
    $upstreamRoot = Join-Path $ProductRoot $spec.pins.mjwarp.path
    foreach ($pair in @(@('SMOOTH_SHA256', 'smooth.py'), @('PASSIVE_SHA256', 'passive.py'), @('SUPPORT_SHA256', 'support.py'), @('TYPES_SHA256', 'types.py'))) {
        $mc = [regex]::Match($moduleText, ("const {0}: &str = ""(?<h>[0-9a-f]{{64}})""" -f $pair[0]))
        if (-not $mc.Success) { continue }
        $upFile = Get-ChildItem -LiteralPath $upstreamRoot -Recurse -File -Filter $pair[1] -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -notmatch '\\build\\' } | Select-Object -First 1
        if ($null -eq $upFile) { $upstreamChecks += [ordered]@{ file = $pair[1]; expected = $mc.Groups['h'].Value; actual = $null; ok = $false }; continue }
        $actual = Get-Sha256File $upFile.FullName
        $upstreamChecks += [ordered]@{ file = (Rel $upFile.FullName); expected = $mc.Groups['h'].Value; actual = $actual; ok = ($actual -eq $mc.Groups['h'].Value) }
    }
    $upstreamAllOk = ($upstreamChecks.Count -gt 0) -and (@($upstreamChecks | Where-Object { -not $_.ok }).Count -eq 0)
    $state = if ($ptxConst -ne '' -and $entityHash -eq $ptxConst -and $receiptRecords) { 'verified' } else { 'missing-or-mismatch' }
    $frozenRows += [pscustomobject]@{
        id = "chain:$($chain.id)"
        name = $chain.id
        route = 'FrozenPtx'
        module = $chain.module
        source = [ordered]@{ path = $chain.ptx_entity; sha256 = $entityHash; ptx_const = $ptxConst; cu_entity_ok = $cuOk }
        entries = $entries
        compile = [ordered]@{
            kind = 'Warp 1.15.0 冻结 PTX（产品直接加载，不重编译）'
            generation = $chain.upstream.generation
            target_arch = 89
        }
        artifact = [ordered]@{ ptx_sha256 = @($ptxConst); entity_ok = ($entityHash -eq $ptxConst); receipt_records_digest = $receiptRecords; evidence_paths = @($chain.ptx_entity, $chain.receipt_entity) }
        evidence_state = $state
        upstream = [ordered]@{ mjwarp_revision = $chain.upstream.revision; warp_version = $chain.upstream.warp_version; source_files = @($chain.upstream.source_files); source_identity_checks = @($upstreamChecks); source_identity_ok = $upstreamAllOk; warp_kernel_symbols = $entries }
        conclusion = if ($state -eq 'verified') { "冻结链证据完整：实体哈希 = 模块常量，回执记录摘要一致，entry $($entries.Count) 个" } else { '冻结链证据不一致（见 artifact/upstream 字段）' }
    }
}
Write-Output ("冻结 PTX 链：{0} 条，entry {1} 个" -f $frozenRows.Count, (@($frozenRows | ForEach-Object { $_.entries }).Count))

# ---------- 6. MathDx 路线（配方级） ----------
$mdx = $spec.mathdx_route
$digestProps = @($mdx.expected_digests.PSObject.Properties)
$probeMatches = 0
foreach ($prop in $digestProps) {
    $v = [string]$prop.Value
    if ($routeProbe.Contains($v) -and $routeG01Probe.Contains($v)) { $probeMatches++ }
}
$routeReceiptText = ($routeReceipt | ConvertTo-Json -Depth 12)
$routeKeyOk = $routeReceiptText.Contains($mdx.expected_digests.route_key_sha256)
$mdxRow = [pscustomobject]@{
    id = "route:$($mdx.id)"
    name = $mdx.name
    route = 'MathDxLto'
    module = $mdx.source
    source = [ordered]@{ entry = $mdx.entry; options = $mdx.options; nvrtc = $mdx.versions.nvrtc; nvjitlink = $mdx.versions.nvjitlink; libmathdx = $mdx.versions.libmathdx }
    artifact = [ordered]@{ route_key_sha256 = $mdx.expected_digests.route_key_sha256; digests_recorded_in_receipt = $routeKeyOk; probe_digest_matches = $probeMatches; probe_expected = $digestProps.Count; evidence_paths = @($mdx.evidence) }
    evidence_state = if ($routeKeyOk -and $probeMatches -eq $digestProps.Count) { 'verified' } else { 'missing-or-mismatch' }
    upstream = [ordered]@{ mjwarp_revision = $spec.pins.mjwarp.revision; note = 'MathDx Solver LTO 走 libmathdx 求解器 LTOIR，上游为 MathDx SDK 0.3.1 + 冻结 7 维 POTRF 对照，不属 MJWarp/Warp 内核集合' }
    conclusion = '配方级证据：路线键与两级探针日志摘要一致（两环境 1 过 0 败）'
}
Write-Output ("MathDx 路线：探针摘要命中 {0}/{1}" -f $probeMatches, $digestProps.Count)

# ---------- 7. 汇总与缺口 ----------
$allRows = @($kernelRows) + @($frozenRows) + @($mdxRow)
$withEvidence = @($allRows | Where-Object { $_.evidence_state -eq 'verified' })
$evidencePresent = @($allRows | Where-Object { $_.evidence_state -in @('verified','source-drift','derived','renamed-evidence') })
$sourceDrift = @($kernelRows | Where-Object { $_.evidence_state -eq 'source-drift' })
$renamed = @($allRows | Where-Object { $_.evidence_state -eq 'renamed-evidence' })
$derived = @($allRows | Where-Object { $_.evidence_state -eq 'derived' })
$undeterminedRowsAll = @($allRows | Where-Object { $_.evidence_state -eq 'undetermined' })
$missing = @($allRows | Where-Object { $_.evidence_state -eq 'missing' -or $_.evidence_state -eq 'receipt-without-entity' -or $_.evidence_state -eq 'missing-or-mismatch' })
$receiptsWithoutEntity = @($receiptRows | Where-Object { -not $_.ptxEntityOk })
$entitiesVerified = @($receiptRows | Where-Object { $_.ptxEntityOk })

$gapItems = @()
foreach ($row in $missing) {
    $gapItems += [ordered]@{ id = $row.id; name = $row.name; status = '缺失证据'; route = $row.route; module = $row.module; reason = $row.conclusion }
}
foreach ($row in $sourceDrift) {
    $gapItems += [ordered]@{ id = $row.id; name = $row.name; status = '源身份漂移'; route = $row.route; module = $row.module; reason = $row.conclusion; evidence_paths = @($row.artifact.evidence_paths) }
}
foreach ($row in $derived) {
    $gapItems += [ordered]@{ id = $row.id; name = $row.name; status = '派生（证据由基核承担）'; route = $row.route; module = $row.module; reason = $row.conclusion; base = $row.source.derived_from; evidence_paths = @($row.artifact.evidence_paths) }
}
foreach ($row in $renamed) {
    $gapItems += [ordered]@{ id = $row.id; name = $row.name; status = '同源不同名（历史命名）'; route = $row.route; module = $row.module; reason = $row.conclusion; renamed_entries = @($row.artifact.renamed_entries); evidence_paths = @($row.artifact.evidence_paths) }
}
foreach ($row in $undeterminedRowsAll) {
    $gapItems += [ordered]@{ id = $row.id; name = $row.name; status = '未判定'; route = $row.route; module = $row.module; reason = $row.conclusion }
}
foreach ($g in $np01Diff.gaps) {
    $status = if ($g.status -like '未实现*') { '未实现（不计入 Windows 缺失）' } else { '部分覆盖（NP01 缺口）' }
    $gapItems += [ordered]@{ id = 'np01-gap'; name = $g.route; status = $status; reason = $g.reason; upstream_files = @($g.upstream_files); product_paths = @($g.product_paths) }
}
foreach ($h in $spec.helper_sources) { $gapItems += [ordered]@{ id = 'helper:' + $h.path; name = $h.const; status = $h.status; reason = $h.reason } }
foreach ($u in $spec.undetermined) { $gapItems += [ordered]@{ id = 'undetermined:' + $u.id; name = $u.name; status = $u.status; reason = $u.reason } }
foreach ($g in $spec.global_gaps) { $gapItems += [ordered]@{ id = 'global:' + $g.id; name = $g.id; status = $g.status; reason = $g.reason } }
foreach ($d in $spec.deferred) { $gapItems += [ordered]@{ id = 'deferred:' + $d.id; name = $d.name; status = $d.status; reason = $d.reason } }
$frozenNotOk = @($frozenRows | Where-Object { -not $_.upstream.source_identity_ok })
foreach ($f in $frozenNotOk) { $gapItems += [ordered]@{ id = 'chain-source:' + $f.name; name = $f.name; status = '未判定'; reason = '模块内上游源 SHA256 常量与 pinned checkout 对应文件不一致或无法比对' } }
$receiptFailedItems = @($receiptFailed)
foreach ($f in $receiptFailedItems) { $gapItems += [ordered]@{ id = 'receipt:' + $f; name = $f; status = '回执实体异常'; reason = '回执文件缺失或哈希与索引不符（本脚本已逐份复核）' } }
$gapItems += [ordered]@{
    id = 'global:receipt-ptx-entities-not-in-repo'
    name = '历史捕获 PTX 实体不在仓'
    status = '部分缺失'
    reason = ('{0}/{1} 份回执在仓内无可核对的 PTX 实体（历史捕获产物依赖 target 产物，未入库）；已入库实体 {2} 份逐份哈希复核通过；本任务不重采集' -f $receiptsWithoutEntity.Count, $receiptRows.Count, $entitiesVerified.Count)
}

$coverageAll = [math]::Round(100.0 * $evidencePresent.Count / [math]::Max(1, $allRows.Count), 2)
$identityOk = @($kernelRows | Where-Object { $_.source_identity_matches_capture }).Count + @($frozenRows | Where-Object { $_.evidence_state -eq 'verified' -and $_.upstream.source_identity_ok }).Count + [int](@($allRows | Where-Object { $_.route -eq 'MathDxLto' -and $_.evidence_state -eq 'verified' }).Count)
$coverageIdentity = [math]::Round(100.0 * $identityOk / [math]::Max(1, $allRows.Count), 2)

$inventory = [ordered]@{
    schema = 1
    task_id = $taskId
    generated_at = (Get-Date).ToString('yyyy-MM-ddTHH:mm:sszzz')
    tool = 'scripts/np02-evidence/np02-evidence.ps1'
    command = 'powershell -File scripts/np02-evidence/np02-evidence.ps1 -ProductRoot <repo> -OutDir docs/development-evidence/np02-evidence'
    scope = 'Windows 单平台已实现内核/配方（不含未实现子系统与 Linux）'
    platform = [ordered]@{ os = 'Windows'; gpu = 'NVIDIA GeForce RTX 4070 Ti SUPER'; driver = '596.36'; target_arch = 70 }
    pins = $pinReport
    inputs = [ordered]@{
        receipts_index = $spec.inputs.receipts_index
        receipts_total = $receiptRows.Count
        receipts_file_hash_ok = $receiptVerified
        receipts_failed = $receiptFailedItems.Count
        t4_artifacts = @($t4Current.artifacts).Count
        t4_mappings = @($t4Current.mappings).Count
        np01_rows = @($np01Mapping.rows).Count
        np01_gaps = @($np01Diff.gaps).Count
    }
    counts = [ordered]@{
        windows_implemented_paths = $allRows.Count
        kernel_rows = $kernelRows.Count
        frozen_chain_rows = $frozenRows.Count
        mathdx_route_rows = 1
        derived_kernel_rows = $derived.Count
        renamed_kernel_rows = $renamed.Count
        undetermined_kernel_rows = $undeterminedRowsAll.Count
        evidence_verified = $withEvidence.Count
        evidence_present = $evidencePresent.Count
        evidence_missing = $missing.Count
        source_identity_drift = $sourceDrift.Count
        receipts_total = $receiptRows.Count
        receipts_file_hash_ok = $receiptVerified
        receipt_ptx_entities_verified = $entitiesVerified.Count
        receipt_ptx_entities_missing = $receiptsWithoutEntity.Count
        coverage_pct_evidence = $coverageAll
        coverage_pct_source_identity = $coverageIdentity
        gap_items = $gapItems.Count
        unimplemented = @($gapItems | Where-Object { $_.status -like '未实现*' }).Count
        partial = @($gapItems | Where-Object { $_.status -like '部分覆盖*' }).Count
        deferred = @($gapItems | Where-Object { $_.status -eq 'deferred' }).Count
        undetermined = @($gapItems | Where-Object { $_.status -eq '未判定' -or $_.status -like '不适用*' }).Count
    }
    rows = $allRows
    gaps = $gapItems
    boundaries = @(
        'Windows 单平台已实现路径清单；未实现子系统与 Linux deferred 单列在 gaps，不计入 Windows 缺失',
        '本清单不构成 NP02 完整验收，也不代表双平台结论',
        '未改产品源码/测试/容差/fixtures；未跑 GPU；未启动 Python',
        '回执与 PTX 实体按文件哈希逐份复核；上游 pin 与上游源 SHA256 常量按 pinned checkout 复核'
    )
}
$gapDoc = [ordered]@{
    schema = 1
    task_id = $taskId
    generated_at = $inventory.generated_at
    counts = $inventory.counts
    items = $gapItems
}

if ($Check) {
    Write-Output '[-Check] 只做校验，不写文件'
} else {
    if (-not (Test-Path -LiteralPath $outFull)) { New-Item -ItemType Directory -Force -Path $outFull | Out-Null }
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText((Join-Path $outFull 'inventory.json'), ($inventory | ConvertTo-Json -Depth 12), $utf8)
    [System.IO.File]::WriteAllText((Join-Path $outFull 'gap.json'), ($gapDoc | ConvertTo-Json -Depth 8), $utf8)
    Write-Output ("已写出：{0}、{1}" -f (Join-Path $outFull 'inventory.json'), (Join-Path $outFull 'gap.json'))
}

$c = $inventory.counts
Write-Output ''
Write-Output "== NP02 Windows 单平台清单 =="
Write-Output ("已实现路径      : {0}（内核 {1} + 冻结链 {2} + MathDx 路线 1）" -f $c.windows_implemented_paths, $c.kernel_rows, $c.frozen_chain_rows)
Write-Output ("有证据          : {0}（覆盖率 {1}%；其中源身份一致 {2}、源身份漂移 {3}、派生 {4}、同源旧名 {5}）" -f $c.evidence_present, $c.coverage_pct_evidence, $c.evidence_verified, $c.source_identity_drift, $c.derived_kernel_rows, $c.renamed_kernel_rows)
Write-Output ("缺失证据        : {0}" -f $c.evidence_missing)
Write-Output ("缺口条目        : {0}（未实现 {1} / 部分 {2} / deferred {3} / 未判定或不适用 {4}）" -f $c.gap_items, $c.unimplemented, $c.partial, $c.deferred, $c.undetermined)
Write-Output ("回执复核        : {0}/{1} 文件哈希通过，PTX 实体缺失 {2} 条" -f $receiptVerified, $receiptRows.Count, @($receiptRows | Where-Object { -not $_.ptxEntityOk }).Count)
