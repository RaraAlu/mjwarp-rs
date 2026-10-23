# NP01 逐字段精度映射（只读）
#
# 输入冻结上游源码（MJWarp 71da24d9… + Warp 1.15.0 ffa038b1…）与产品源码，
# 输出 mapping.json（逐字段/内核/编译选项映射）与 diff.json（偏离、未覆盖、缺口）。
# 只读：不修改任何源码、测试、容差或 fixtures；不启动 Python。
[CmdletBinding()]
param(
    [string]$ProductRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)),
    [string]$UpstreamRoot = '',
    [string]$WarpRoot = '',
    [string]$OutDir = '',
    [string]$TaskId = '2026-10-24-np01-field-precision-map',
    [string]$ExpectedUpstreamCommit = '71da24d956378a87a703b6e1442b13aec0c4ac29',
    [string]$ExpectedWarpCommit = 'ffa038b1adf3927a8e893d4536f1a7562f17d749',
    [string]$SpecPath = ''
)

$ErrorActionPreference = 'Stop'
$ProductRoot = (Resolve-Path -LiteralPath $ProductRoot).Path
if (-not $UpstreamRoot) { $UpstreamRoot = Join-Path $ProductRoot 'target\mjwarp-parallel\upstream' }
if (-not $WarpRoot) { $WarpRoot = Join-Path $ProductRoot 'target\research\warp' }
if (-not $OutDir) { $OutDir = Join-Path $ProductRoot 'docs\development-evidence\np01-field-mapping' }
if (-not $SpecPath) { $SpecPath = Join-Path $PSScriptRoot 'spec.json' }

function Fail([string]$Message) {
    throw "NP01 映射失败：$Message（不猜测、不伪造，请修复输入后重跑）"
}

# ---------- 输入校验 ----------
foreach ($item in @(@{ Path = $UpstreamRoot; Name = 'MJWarp 上游' }, @{ Path = $WarpRoot; Name = 'Warp 上游' })) {
    if (-not (Test-Path -LiteralPath $item.Path)) { Fail "$($item.Name)目录缺失：$($item.Path)" }
    $actual = (& git -C $item.Path rev-parse HEAD 2>$null)
    if (-not $actual) { Fail "$($item.Name)不是 git 仓库或无法读取 HEAD：$($item.Path)" }
    $item.Path | Out-Null
    if ($item.Name -eq 'MJWarp 上游' -and $actual.Trim() -ne $ExpectedUpstreamCommit) {
        Fail "MJWarp 上游 pin 不符：期望 $ExpectedUpstreamCommit，实际 $($actual.Trim())"
    }
    if ($item.Name -eq 'Warp 上游' -and $actual.Trim() -ne $ExpectedWarpCommit) {
        Fail "Warp 上游 pin 不符：期望 $ExpectedWarpCommit，实际 $($actual.Trim())"
    }
}
$spec = Get-Content -LiteralPath $SpecPath -Raw -Encoding UTF8 | ConvertFrom-Json
$typesPath = Join-Path $UpstreamRoot 'mujoco_warp\_src\types.py'
if (-not (Test-Path -LiteralPath $typesPath)) { Fail "上游 types.py 缺失：$typesPath" }

# ---------- Warp 类型语义 ----------
$typeMap = @{
    'float' = 'f32'; 'float32' = 'f32'; 'float64' = 'f64'
    'int' = 'i32'; 'int32' = 'i32'; 'int64' = 'i64'; 'bool' = 'bool'
    'vec2' = 'f32[2]'; 'vec2f' = 'f32[2]'; 'vec2i' = 'i32[2]'
    'vec3' = 'f32[3]'; 'vec3f' = 'f32[3]'; 'vec3d' = 'f64[3]'
    'vec4' = 'f32[4]'; 'quat' = 'f32[4]'; 'vec5' = 'f32[5]'; 'vec5f' = 'f32[5]'
    'vec6' = 'f32[6]'; 'spatial_vector' = 'f32[6]'
    'vec8f' = 'f32[8]'; 'mat33' = 'f32[9]'; 'mat43' = 'f32[12]'
    'vec10' = 'f32[10]'; 'mat' = 'f32[9]'
}
function MapWarpType([string]$Token) {
    $t = $Token.Trim()
    $t = $t -replace '^wp\.', ''
    $t = $t -replace '^mujoco\.', ''
    if ($typeMap.ContainsKey($t)) { return $typeMap[$t] }
    return "未识别($t)"
}

# 解析上游 types.py：仅 Model/Data 两个 dataclass 的 typed 字段（字段名可含大写，如 M/ten_J）
$upstreamTypes = @{}
$typeLines = @(Get-Content -LiteralPath $typesPath -Encoding UTF8)
$currentClass = ''
for ($i = 0; $i -lt $typeLines.Count; $i++) {
    $line = $typeLines[$i]
    $cm = [regex]::Match($line, '^class ([A-Za-z0-9_]+)')
    if ($cm.Success) { $currentClass = $cm.Groups[1].Value; continue }
    if ($currentClass -notin @('Model', 'Data')) { continue }
    $m = [regex]::Match($line, '^  ([A-Za-z][A-Za-z0-9_]*): (array\([^)]*\)|[A-Za-z_][A-Za-z0-9_.]*)\s*$')
    if (-not $m.Success) { continue }
    $field = $m.Groups[1].Value
    $expr = $m.Groups[2].Value
    $token = ''
    if ($expr.StartsWith('array(')) {
        $inner = $expr.Substring(6, $expr.Length - 7)
        $parts = $inner -split ','
        $token = $parts[$parts.Count - 1].Trim()
    } else {
        $token = $expr.Trim()
    }
    if (-not $upstreamTypes.ContainsKey($field)) {
        $upstreamTypes[$field] = @{ token = $token; type = (MapWarpType $token); line = ($i + 1); class = $currentClass }
    }
}

# 上游字段引用位置（跨全部 _src/*.py）
$upstreamFiles = @(Get-ChildItem -LiteralPath (Join-Path $UpstreamRoot 'mujoco_warp\_src') -Filter *.py -File)
$refMap = @{}
foreach ($file in $upstreamFiles) {
    $text = Get-Content -LiteralPath $file.FullName -Encoding UTF8
    for ($i = 0; $i -lt $text.Count; $i++) {
        foreach ($mm in [regex]::Matches($text[$i], '\b[dm]\.([A-Za-z][A-Za-z0-9_]*)')) {
            $field = $mm.Groups[1].Value
            if (-not $refMap.ContainsKey($field)) { $refMap[$field] = New-Object System.Collections.ArrayList }
            if ($refMap[$field].Count -lt 3) {
                [void]$refMap[$field].Add(("mujoco_warp/_src/{0}:{1}" -f $file.Name, ($i + 1)))
            }
        }
    }
}

# f64 分类（路径前缀）
function ClassifyF64([string]$RelativePath) {
    foreach ($entry in $spec.f64_allowlist) {
        if ($RelativePath.StartsWith($entry.path)) { return $entry }
    }
    return $null
}

# f64 扫描：产品物理非测试源码
$testPathPattern = '(^|/)(tests?)(/|$)|tests\.rs$'
$physicsDir = Join-Path $ProductRoot 'src\physics'
$f64Hits = New-Object System.Collections.ArrayList
$f64Unclassified = New-Object System.Collections.ArrayList
$f64Classes = @{}
foreach ($file in @(Get-ChildItem -LiteralPath $physicsDir -Recurse -Filter *.rs -File)) {
    $rel = $file.FullName.Substring($ProductRoot.Length + 1).Replace('\', '/')
    if ($rel -match $testPathPattern) { continue }
    $lines = @(Get-Content -LiteralPath $file.FullName -Encoding UTF8)
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -notmatch '\bf64\b') { continue }
        $entry = ClassifyF64 $rel
        [void]$f64Hits.Add([ordered]@{
            path = $rel; line = ($i + 1); text = $lines[$i].Trim()
            class = if ($entry) { $entry.class } else { '未分类' }
            reason = if ($entry) { $entry.reason } else { '未在 allowlist 中登记，需人工判定是否偏离' }
        })
        if ($entry) {
            $key = "$rel|$($entry.class)"
            if (-not $f64Classes.ContainsKey($key)) { $f64Classes[$key] = 0 }
            $f64Classes[$key]++
        } else {
            [void]$f64Unclassified.Add([ordered]@{ path = $rel; line = ($i + 1); text = $lines[$i].Trim() })
        }
    }
}
$f64Summary = New-Object System.Collections.ArrayList
foreach ($key in ($f64Classes.Keys | Sort-Object)) {
    $parts = $key -split '\|', 2
    [void]$f64Summary.Add([ordered]@{ path = $parts[0]; class = $parts[1]; hits = $f64Classes[$key] })
}

# ---------- 产品字段与 CUDA 内核 ----------
$rows = New-Object System.Collections.ArrayList
$kernelRows = New-Object System.Collections.ArrayList
$productFieldIndex = @{}
$rowKeys = @{}
function Get-Accum([string]$upstreamField, [string]$productField) {
    $key = $null
    if ($spec.accumulations.PSObject.Properties.Name -contains $upstreamField) { $key = $upstreamField }
    elseif ($spec.accumulations.PSObject.Properties.Name -contains $productField) { $key = $productField }
    if ($key) { return $spec.accumulations.$key }
    return '同字段（逐元素 f32，无显式 f64 累加）'
}
function Get-UpstreamAlias([string]$field) {
    if ($spec.aliases.PSObject.Properties.Name -contains $field) { return $spec.aliases.$field }
    return $field
}
function Get-ProductOnlyNote([string]$field) {
    if ($spec.product_only.PSObject.Properties.Name -contains $field) { return $spec.product_only.$field }
    return $null
}
function Get-TypeConclusion([string]$upstreamType, [string]$productType) {
    if ($upstreamType -like '未识别*' -or $productType -like '未识别*') { return "偏离（类型未识别：上游 $upstreamType / 产品 $productType）" }
    if ($upstreamType -eq $productType) { return "一致（$upstreamType）" }
    if ($productType -eq 'f32' -and $upstreamType -match '^f32\[\d+\]$') { return "一致（f32；上游 $upstreamType 展平为产品 [f32] 布局）" }
    if ($productType -eq 'i32' -and $upstreamType -match '^i32\[\d+\]$') { return "一致（i32；上游 $upstreamType 展平为产品 [i32] 布局）" }
    if ($upstreamType -match '^f32\[\d+\]$' -and $productType -eq "f32[]") { return "一致（f32；上游 $upstreamType 展平为产品 [f32] 布局）" }
    if ($upstreamType -eq 'f64' -and $productType -eq 'f32') { return '偏离（上游 f64 → 产品 f32，降精度）' }
    if ($upstreamType -eq 'f32' -and $productType -eq 'f64') { return '偏离（上游 f32 → 产品 f64，升精度）' }
    return "偏离（类型不一致：上游 $upstreamType / 产品 $productType）"
}

foreach ($moduleEntry in $spec.product_modules) {
    foreach ($relFile in $moduleEntry.files) {
        $full = Join-Path $ProductRoot ($relFile.Replace('/', '\'))
        if (-not (Test-Path -LiteralPath $full)) { continue }
        $lines = @(Get-Content -LiteralPath $full -Encoding UTF8)
        $struct = ''
        for ($i = 0; $i -lt $lines.Count; $i++) {
            $line = $lines[$i]
            $sm = [regex]::Match($line, 'pub struct ([A-Za-z0-9_]+)')
            if ($sm.Success) { $struct = $sm.Groups[1].Value; continue }
            $fm = [regex]::Match($line, 'pub ([a-z_][a-z0-9_]*):\s*&''a \[([a-z0-9]+)\]')
            $fmNested = [regex]::Match($line, 'pub ([a-z_][a-z0-9_]*):\s*&''a \[\[([a-z0-9]+); (\d+)\]\]')
            if (-not $fm.Success -and -not $fmNested.Success) { continue }
            if ($fmNested.Success) {
                $field = $fmNested.Groups[1].Value
                $elem = $fmNested.Groups[2].Value
                $productType = "$elem[$($fmNested.Groups[3].Value)]"
            } else {
                $field = $fm.Groups[1].Value
                $elem = $fm.Groups[2].Value
                $productType = if ($elem -in @('f32', 'f64', 'i32', 'i64', 'u32', 'u64', 'bool')) { $elem } else { "未识别($elem)" }
            }
            $upField = Get-UpstreamAlias $field
            $upType = '未命中'
            $upRefs = @()
            if ($upstreamTypes.ContainsKey($upField)) {
                $upType = $upstreamTypes[$upField].type
                $upRefs = @($refMap[$upField])
            }
            $productType = if ($elem -in @('f32', 'f64', 'i32', 'i64', 'u32', 'u64', 'bool')) { $elem } else { "未识别($elem)" }
            $obj = if ($struct) { "$struct.$field" } else { $field }
            $rowKey = "$($moduleEntry.module)|$obj"
            if ($rowKeys.ContainsKey($rowKey)) { continue }
            $rowKeys[$rowKey] = $true
            $note = Get-ProductOnlyNote $field
            $f64Class = $null
            foreach ($entry in $spec.f64_allowlist) {
                if ($relFile.StartsWith($entry.path)) { $f64Class = $entry; break }
            }
            $f64Flag = if ($f64Class) { "分类：$($f64Class.class)（非物理状态）" } else { '无（本文件未命中 f64）' }
            $conclusion = if ($upType -eq '未命中' -and $note) {
                "产品内部字段（上游无同名 Data/Model 字段；语义映射待下一轮）"
            } elseif ($upType -eq '未命中') {
                '未映射（上游 Data/Model 无同名字段）'
            } else {
                Get-TypeConclusion $upType $productType
            }
            $evidence = "$relFile#L$($i + 1)"
            if ($upRefs.Count -gt 0) { $evidence += '; ' + ($upRefs -join '; ') }
            if ($upstreamTypes.ContainsKey($upField)) { $evidence += "; mujoco_warp/_src/types.py#L$($upstreamTypes[$upField].line)" }
            $row = [ordered]@{
                module = $moduleEntry.module
                object = $obj
                upstream_type = $upType
                upstream_accum_type = (Get-Accum $upField $field)
                product_type = $productType
                product_accum_type = (Get-Accum $upField $field)
                f64_mixed_flag = $f64Flag
                conclusion = $conclusion
                evidence_path = $evidence
            }
            [void]$rows.Add($row)
            if (-not $productFieldIndex.ContainsKey($obj)) { $productFieldIndex[$obj] = $row }
        }
        # CUDA 内核常量（输入/输出/关键累加量的标量类型扫描）
        $text = ($lines -join "`n")
        foreach ($km in [regex]::Matches($text, 'const ([A-Z][A-Z0-9_]+): &str = r#"(.*?)"#;', 'Singleline')) {
            $name = $km.Groups[1].Value
            $body = $km.Groups[2].Value
            $floatCount = ([regex]::Matches($body, '\bfloat\b')).Count
            $doubleCount = ([regex]::Matches($body, '\bdouble\b')).Count
            $productType = "CUDA 模板：float $floatCount 处 / double $doubleCount 处"
            $conclusion = if ($doubleCount -eq 0) { '一致（f32 内核，无 double 标记）' } else { "偏离（内核含 double $doubleCount 处，需核）" }
            [void]$kernelRows.Add([ordered]@{
                module = $moduleEntry.module
                object = "内核 $name"
                upstream_type = '（按模块归类，见模块字段映射）'
                upstream_accum_type = 'f32 累加（内核内累加量按内核标量类型）'
                product_type = $productType
                product_accum_type = 'f32（无 double 时）'
                f64_mixed_flag = if ($doubleCount -eq 0) { '无' } else { '偏离（double）' }
                conclusion = $conclusion
                evidence_path = "$relFile（$name）"
            })
        }
    }
}

# ---------- 编译选项 ----------
$compileRows = New-Object System.Collections.ArrayList
$compileDiffs = New-Object System.Collections.ArrayList
foreach ($opt in $spec.compile_options) {
    $pfile = Join-Path $ProductRoot ($opt.product_file.Replace('/', '\'))
    $ufile = Join-Path $ProductRoot ($opt.upstream_file.Replace('/', '\'))
    if (-not (Test-Path -LiteralPath $pfile)) { Fail "编译选项锚点文件缺失：$($opt.product_file)" }
    if (-not (Test-Path -LiteralPath $ufile)) { Fail "上游编译选项锚点文件缺失：$($opt.upstream_file)" }
    $pText = Get-Content -LiteralPath $pfile -Raw -Encoding UTF8
    $uText = Get-Content -LiteralPath $ufile -Raw -Encoding UTF8
    $pOk = $pText.Contains($opt.product_anchor)
    $uOk = $uText.Contains($opt.upstream_anchor)
    if (-not $pOk) { Fail "产品锚点未命中：$($opt.product_file) :: $($opt.product_anchor)" }
    if (-not $uOk) { Fail "上游锚点未命中：$($opt.upstream_file) :: $($opt.upstream_anchor)" }
    [void]$compileRows.Add([ordered]@{
        module = 'runtime'
        object = "编译选项 $($opt.option)"
        upstream_type = $opt.upstream_value
        upstream_accum_type = '不适用'
        product_type = $opt.product_value
        product_accum_type = '不适用'
        f64_mixed_flag = '不适用'
        conclusion = $opt.conclusion
        evidence_path = "$($opt.product_file)（锚点：$($opt.product_anchor)）；$($opt.upstream_file)（锚点：$($opt.upstream_anchor)）"
    })
    if ($opt.conclusion -like '差异观察*') {
        [void]$compileDiffs.Add([ordered]@{ option = $opt.option; conclusion = $opt.conclusion; product = $opt.product_value; upstream = $opt.upstream_value })
    }
}

# ---------- diff：偏离/未覆盖/缺口 ----------
$diffItems = New-Object System.Collections.ArrayList
foreach ($row in $rows) {
    if ($row.conclusion -like '偏离*') {
        [void]$diffItems.Add([ordered]@{ kind = 'type_mismatch'; module = $row.module; object = $row.object; detail = $row.conclusion; evidence = $row.evidence_path })
    } elseif ($row.conclusion -like '未映射*') {
        [void]$diffItems.Add([ordered]@{ kind = 'unmapped_product_field'; module = $row.module; object = $row.object; detail = '上游 Data/Model 无同名字段，需语义映射'; evidence = $row.evidence_path })
    } elseif ($row.conclusion -like '产品内部字段*') {
        [void]$diffItems.Add([ordered]@{ kind = 'internal_layout_field'; module = $row.module; object = $row.object; detail = $row.conclusion; evidence = $row.evidence_path })
    }
}
foreach ($row in $kernelRows) {
    if ($row.conclusion -like '偏离*') {
        [void]$diffItems.Add([ordered]@{ kind = 'kernel_type_mismatch'; module = $row.module; object = $row.object; detail = $row.conclusion; evidence = $row.evidence_path })
    }
}
foreach ($hit in $f64Unclassified) {
    [void]$diffItems.Add([ordered]@{ kind = 'f64_unclassified'; module = 'physics'; object = $hit.path; detail = "未分类 f64：$($hit.text)"; evidence = "$($hit.path)#L$($hit.line)" })
}
$gapItems = New-Object System.Collections.ArrayList
foreach ($gap in $spec.gaps) {
    $present = @()
    foreach ($p in $gap.product_paths) {
        $full = Join-Path $ProductRoot ($p.Replace('/', '\'))
        if ((Test-Path -LiteralPath $full) -or (Test-Path -LiteralPath ($full + '.rs'))) { $present += $p }
    }
    $status = if ($gap.PSObject.Properties.Name -contains 'status') { $gap.status } elseif ($present.Count -eq 0) { '未实现（产品无对应模块）' } elseif ($gap.route -like '*部分覆盖*') { '部分覆盖（见缺口说明）' } else { '路径存在但本轮未映射（需下一轮补充）' }
    $upExists = $true
    foreach ($u in $gap.upstream_files) {
        if (-not (Test-Path -LiteralPath (Join-Path $UpstreamRoot ($u.Replace('/', '\'))))) { $upExists = $false }
    }
    if (-not $upExists) { Fail "缺口上游文件缺失，规格与实际不符：$($gap.route)" }
    [void]$gapItems.Add([ordered]@{ route = $gap.route; status = $status; reason = $gap.reason; upstream_files = $gap.upstream_files; product_paths = $gap.product_paths })
    [void]$diffItems.Add([ordered]@{ kind = 'gap'; module = ($gap.upstream_files -join ','); object = $gap.route; detail = "$status：$($gap.reason)"; evidence = 'scripts/np01-field-map/spec.json' })
}

# 上游未被本规格引用的模块（附录，不冒充覆盖）
$coveredUpstream = @()
foreach ($gap in $spec.gaps) { $coveredUpstream += $gap.upstream_files }
if ($spec.PSObject.Properties.Name -contains 'referenced_extra') { $coveredUpstream += @($spec.referenced_extra) }
$referencedFiles = @()
foreach ($field in $refMap.Keys) { foreach ($r in $refMap[$field]) { $referencedFiles += ($r -split ':')[0] } }
$unreferenced = New-Object System.Collections.ArrayList
foreach ($file in $upstreamFiles) {
    $rel = "mujoco_warp/_src/$($file.Name)"
    if ($coveredUpstream -contains $rel) { continue }
    if ($referencedFiles -contains $rel) { continue }
    if ($file.Name -match '_test\.py$|^conftest') { continue }
    [void]$unreferenced.Add($rel)
}

# ---------- 输出 ----------
$mapping = [ordered]@{
    schema = 1
    task_id = $TaskId
    generated_at = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ')
    tool = 'scripts/np01-field-map/np01-field-map.ps1'
    command = "pwsh -NoProfile -File scripts/np01-field-map/np01-field-map.ps1 -ProductRoot <repo> -OutDir docs/development-evidence/np01-field-mapping"
    inputs = [ordered]@{
        product_root = 'D:\Rust\mjwarp-rs'
        product_commit = (& git -C $ProductRoot rev-parse HEAD 2>$null)
        upstream_root = 'target/mjwarp-parallel/upstream'
        upstream_commit = $ExpectedUpstreamCommit
        warp_root = 'target/research/warp'
        warp_commit = $ExpectedWarpCommit
        spec = 'scripts/np01-field-map/spec.json'
    }
    type_semantics = $spec.notes.warp_type_mapping
    counts = [ordered]@{
        field_rows = $rows.Count
        kernel_rows = $kernelRows.Count
        compile_option_rows = $compileRows.Count
        total_rows = ($rows.Count + $kernelRows.Count + $compileRows.Count)
        f64_hits = $f64Hits.Count
        f64_classified = ($f64Hits.Count - $f64Unclassified.Count)
        f64_unclassified = $f64Unclassified.Count
    }
    rows = @($rows + $kernelRows + $compileRows)
    f64_scan = [ordered]@{
        note = '扫描 src/physics 非测试源码中的 f64；分类见 spec.f64_allowlist'
        summary = @($f64Summary)
        hits = @($f64Hits)
    }
}
$diff = [ordered]@{
    schema = 1
    task_id = $TaskId
    generated_at = $mapping.generated_at
    counts = [ordered]@{
        type_mismatch = @($diffItems | Where-Object { $_.kind -eq 'type_mismatch' }).Count
        kernel_type_mismatch = @($diffItems | Where-Object { $_.kind -eq 'kernel_type_mismatch' }).Count
        unmapped_product_field = @($diffItems | Where-Object { $_.kind -eq 'unmapped_product_field' }).Count
        internal_layout_field = @($diffItems | Where-Object { $_.kind -eq 'internal_layout_field' }).Count
        f64_unclassified = @($diffItems | Where-Object { $_.kind -eq 'f64_unclassified' }).Count
        compile_option_difference = $compileDiffs.Count
        gaps = $gapItems.Count
        unreferenced_upstream_modules = $unreferenced.Count
    }
    items = @($diffItems + @($compileDiffs | ForEach-Object { [ordered]@{ kind = 'compile_option_difference'; module = 'runtime'; object = $_.option; detail = $_.conclusion; evidence = 'scripts/np01-field-map/spec.json' } }))
    gaps = @($gapItems)
    unreferenced_upstream_modules = @($unreferenced)
    statement = '本文件只登记偏离/缺口，不构成 NP01 通过或完整验收。'
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$utf8 = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText((Join-Path $OutDir 'mapping.json'), ($mapping | ConvertTo-Json -Depth 8), $utf8)
[System.IO.File]::WriteAllText((Join-Path $OutDir 'diff.json'), ($diff | ConvertTo-Json -Depth 8), $utf8)

Write-Host "NP01 映射完成（只读）："
Write-Host ("  字段行 = {0}；内核行 = {1}；编译选项行 = {2}；合计 = {3}" -f $rows.Count, $kernelRows.Count, $compileRows.Count, ($rows.Count + $kernelRows.Count + $compileRows.Count))
Write-Host ("  上游字段类型命中 = {0}/{1}；产品内部/未映射 = {2}" -f (@($rows | Where-Object { $_.upstream_type -ne '未命中' }).Count), $rows.Count, (@($rows | Where-Object { $_.upstream_type -eq '未命中' }).Count))
Write-Host ("  f64 命中 = {0}（已分类 {1} / 未分类 {2}）" -f $f64Hits.Count, ($f64Hits.Count - $f64Unclassified.Count), $f64Unclassified.Count)
Write-Host ("  diff：类型偏离 = {0}；内核偏离 = {1}；未映射产品字段 = {2}；内部布局字段 = {3}；编译选项差异 = {4}；缺口 = {5}；未引用上游模块 = {6}" -f `
        $diff.counts.type_mismatch, $diff.counts.kernel_type_mismatch, $diff.counts.unmapped_product_field, $diff.counts.internal_layout_field, $diff.counts.compile_option_difference, $diff.counts.gaps, $diff.counts.unreferenced_upstream_modules)
Write-Host ("  输出：{0}" -f $OutDir)
