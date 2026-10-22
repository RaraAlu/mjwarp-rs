# 生成 M3 刚体运动学迁移后的未迁移发射清单。
# 基于 v5 清单（layers-m3-velocity-position-g01-correction），把全部行重定位到
# 61 片拆分后的布局，并记录本任务迁移的三处 rigid_kinematics 发射。
# 不覆盖旧清单：旧文件在另一目录；本目录清单不存在时才写入。
$ErrorActionPreference = 'Stop'
Push-Location (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)))

$root = (Get-Location).Path
$task = '2026-10-20-layers-m3-rigid-kinematics'
$base = (git rev-parse HEAD).Trim()
$priorRel = 'docs/development-evidence/layers-m3-velocity-position-g01-correction/unmigrated-launch-inventory.json'
$outRel = 'docs/development-evidence/layers-m3-rigid-kinematics/unmigrated-launch-inventory.json'
if (Test-Path -LiteralPath $outRel) { throw "清单已存在：$outRel" }

$LaunchRe = [regex]'((?:[A-Za-z_][A-Za-z0-9_]*\.)*[A-Za-z_][A-Za-z0-9_]*)((?:\[[0-9]+\])*)\.(launch(?:_[a-z0-9_]+)?)\('

# 只扫描测试模块之前的产品前缀，并去掉行注释。
function Get-Rows([string[]]$Lines) {
    $end = $Lines.Length
    for ($i = 0; $i -lt $Lines.Length - 1; $i++) {
        if ($Lines[$i] -ceq '#[cfg(test)]' -and $Lines[$i + 1] -ceq 'mod tests {') { $end = $i; break }
    }
    $launch = [Collections.Generic.List[object]]::new()
    $compile = [Collections.Generic.List[object]]::new()
    for ($i = 0; $i -lt $end; $i++) {
        $code = $Lines[$i] -replace '//.*$', ''
        if ($code -match 'SynchronousKernel::compile\(') {
            $k = $i; $limit = [Math]::Min($i + 12, $end)
            while ($k -lt $limit -and ($Lines[$k] -replace '//.*$', '') -notmatch '"([a-z0-9_]+)"') { $k++ }
            if ($k -lt $limit) { $compile.Add([pscustomobject]@{ line = $k + 1; entry = $Matches[1] }) }
            else { $compile.Add([pscustomobject]@{ line = $i + 1; entry = $null }) }
        }
        if ($code -match '^\s*\.(launch(?:_[a-z0-9_]+)?)\(') {
            $j = $i - 1
            while ($j -ge 0 -and $Lines[$j].Trim() -eq '') { $j-- }
            $launch.Add([pscustomobject]@{ line = $i + 1; receiver = $Lines[$j].Trim(); method = $Matches[1] })
        }
        else {
            $mm = $LaunchRe.Match($code)
            if ($mm.Success -and $code -notmatch '\bfn\s+launch') {
                $launch.Add([pscustomobject]@{ line = $i + 1; receiver = $mm.Groups[1].Value + $mm.Groups[2].Value; method = $mm.Groups[3].Value })
            }
        }
    }
    [pscustomobject]@{ launch = $launch; compile = $compile }
}

# 旧文件 -> 拆分后同组文件（根文件加产品子模块，排除测试文件）。
$groupMap = @{}
$splitTasks = Get-ChildItem .normify/changes -Filter '2026-10-21-split-*.json' | ForEach-Object { Get-Content $_.FullName -Raw -Encoding utf8 | ConvertFrom-Json } | Where-Object { $_.id -ne '2026-10-21-split-program-reconcile' }
foreach ($t in $splitTasks) { $groupMap[$t.schedule.files[0]] = @($t.schedule.files[0]) + @($t.schedule.files | Select-Object -Skip 1) }
# 拆分程序之前已完成的两个特例：resident 根拆分子模块，physics/mod.rs 拆出五个产品文件。
$groupMap['src/physics/resident.rs'] = @('src/physics/resident.rs', 'src/physics/resident')
$groupMap['src/physics/mod.rs'] = @('src/physics/mod.rs', 'src/physics/kinematics.rs', 'src/physics/com_position.rs', 'src/physics/device_checks.rs', 'src/physics/history.rs')

function Expand-Scopes([string[]]$scopes) {
    $out = [Collections.Generic.List[string]]::new()
    foreach ($s in $scopes) {
        if (Test-Path -LiteralPath $s -PathType Container) {
            Get-ChildItem -LiteralPath $s -Recurse -File -Filter '*.rs' | Where-Object {
                $_.FullName -notmatch '\\tests\\' -and $_.Name -ne 'tests.rs' -and $_.Name -notlike '*_tests.rs'
            } | ForEach-Object { [void]$out.Add($_.FullName.Substring($root.Length + 1).Replace('\', '/')) }
        } elseif (Test-Path -LiteralPath $s -PathType Leaf) { [void]$out.Add($s) }
    }
    @($out)
}

$inv = Get-Content $priorRel -Raw -Encoding utf8 | ConvertFrom-Json
$launchRows = [Collections.Generic.List[object]]::new()
foreach ($r in @($inv.product_low_level_call_sites) + @($inv.wrapper_orchestration_call_sites) + @($inv.excluded_non_kernel_helpers)) { $launchRows.Add($r) }
$compileRows = @($inv.compile_sites)

# 重定位发射行：组内按 (receiver, method) 优先顺序匹配。
$reloc = @{}
$byFile = @{}
foreach ($r in $launchRows) {
    if (-not $byFile.ContainsKey($r.file)) { $byFile[$r.file] = [Collections.Generic.List[object]]::new() }
    $byFile[$r.file].Add($r)
}
$unmatched = [Collections.Generic.List[string]]::new()
foreach ($file in $byFile.Keys) {
    $rows = @($byFile[$file] | Sort-Object { [int]$_.line })
    $scopes = if ($groupMap.ContainsKey($file)) { $groupMap[$file] } else { @($file) }
    $pool = [Collections.Generic.List[object]]::new()
    foreach ($gf in (Expand-Scopes $scopes)) {
        $scan = Get-Rows ([IO.File]::ReadAllLines((Resolve-Path -LiteralPath $gf)))
        foreach ($r in $scan.launch) { $pool.Add([pscustomobject]@{ file = $gf; line = $r.line; receiver = $r.receiver; method = $r.method }) }
    }
    $used = New-Object 'bool[]' $pool.Count
    foreach ($row in $rows) {
        # 探针 rigid 发射的接收者文本在迁移后改变，单独定位，先跳过以免抢占 COM 探针候选。
        if ($file -ceq "src/physics/mod.rs" -and [int]$row.line -eq 374) { continue }
        $found = -1
        for ($k = 0; $k -lt $pool.Count; $k++) {
            if (-not $used[$k] -and $pool[$k].receiver -ceq $row.receiver -and $pool[$k].method -ceq $row.method) { $found = $k; break }
        }
        if ($found -ge 0) {
            $used[$found] = $true
            $reloc["$file|$($row.line)"] = [pscustomobject]@{ file = $pool[$found].file; line = $pool[$found].line }
        } else {
            $unmatched.Add("$file|$($row.line)")
        }
    }
}
# 本任务只应有一处旧发射无法按原文本重定位：探针从裸内核改为类型包装链式发射。
if ($unmatched.Count -ne 0) { throw "旧发射重定位异常：$($unmatched -join ', ')" }

# 探针新位置：kinematics.rs 中 RigidKernel::new(...).launch( 的 .launch 行。
$kinLines = [IO.File]::ReadAllLines((Resolve-Path -LiteralPath 'src/physics/kinematics.rs'))
$probeLine = -1
for ($i = 0; $i -lt $kinLines.Length; $i++) {
    if ($kinLines[$i] -match 'RigidKernel::new\(') {
        for ($j = $i; $j -lt [Math]::Min($i + 4, $kinLines.Length); $j++) {
            if ($kinLines[$j] -match '\.launch\(') { $probeLine = $j + 1; break }
        }
        if ($probeLine -ge 0) { break }
    }
}
if ($probeLine -lt 0) { throw '未找到探针类型化发射行' }
$reloc['src/physics/mod.rs|374'] = [pscustomobject]@{ file = 'src/physics/kinematics.rs'; line = $probeLine }

# 重定位编译行：组内按 entry 名称顺序匹配。
$cReloc = @{}
$cByFile = @{}
foreach ($r in $compileRows) {
    if (-not $cByFile.ContainsKey($r.file)) { $cByFile[$r.file] = [Collections.Generic.List[object]]::new() }
    $cByFile[$r.file].Add($r)
}
foreach ($file in $cByFile.Keys) {
    $rows = @($cByFile[$file] | Sort-Object { [int]$_.line })
    $scopes = if ($groupMap.ContainsKey($file)) { $groupMap[$file] } else { @($file) }
    $pool = [Collections.Generic.List[object]]::new()
    foreach ($gf in (Expand-Scopes $scopes)) {
        $scan = Get-Rows ([IO.File]::ReadAllLines((Resolve-Path -LiteralPath $gf)))
        foreach ($r in $scan.compile) { $pool.Add([pscustomobject]@{ file = $gf; line = $r.line; entry = $r.entry }) }
    }
    $used = New-Object 'bool[]' $pool.Count
    foreach ($row in $rows) {
        $found = -1
        for ($k = 0; $k -lt $pool.Count; $k++) {
            if (-not $used[$k] -and $pool[$k].entry -ceq $row.entry) { $found = $k; break }
        }
        if ($found -lt 0) { throw "编译点重定位失败：$file|$($row.line) entry=$($row.entry)" }
        $used[$found] = $true
        $cReloc["$file|$($row.line)"] = [pscustomobject]@{ file = $pool[$found].file; line = $pool[$found].line }
    }
}

function Locate([string]$oldFile, [int]$oldLine, $table) {
    $key = "$oldFile|$oldLine"
    if (-not $table.ContainsKey($key)) { throw "缺少重定位：$key" }
    $table[$key]
}
function Ref($loc) { "$($loc.file):$($loc.line)" }

# 三处迁移：探针与常驻两处 rigid 发射改为类型包装。
# 探针来自 src/physics/mod.rs:374；两处常驻来自 resident.rs 的 self.rigid_kernel。
$migrated = [Collections.Generic.List[object]]::new()
$lowLevel = [Collections.Generic.List[object]]::new()
foreach ($r in $inv.product_low_level_call_sites) {
    $loc = Locate $r.file ([int]$r.line) $reloc
    $row = [ordered]@{
        file = $loc.file
        line = $loc.line
        receiver = $r.receiver
        method = $r.method
        classification = $r.classification
        entry = $r.entry
    }
    $isProbe = ($r.file -ceq 'src/physics/mod.rs' -and [int]$r.line -eq 374)
    $isPlan = ($r.file -ceq 'src/physics/resident.rs' -and $r.receiver -ceq 'self.rigid_kernel' -and $r.classification -ceq 'unmigrated_raw')
    if ($isProbe -or $isPlan) {
        $row.classification = 'typed_wrapper_low_level'
        $row.entry = 'rigid_kinematics'
        $migrated.Add([pscustomobject]$row)
    }
    $lowLevel.Add([pscustomobject]$row)
}
if ($migrated.Count -ne 3) { throw "迁移行数量异常：$($migrated.Count)" }

$orchestration = [Collections.Generic.List[object]]::new()
foreach ($r in $inv.wrapper_orchestration_call_sites) {
    $loc = Locate $r.file ([int]$r.line) $reloc
    $orchestration.Add([pscustomobject][ordered]@{ file = $loc.file; line = $loc.line; receiver = $r.receiver; method = $r.method; classification = $r.classification; entry = $r.entry })
}
$excluded = [Collections.Generic.List[object]]::new()
foreach ($r in $inv.excluded_non_kernel_helpers) {
    $loc = Locate $r.file ([int]$r.line) $reloc
    $row = [ordered]@{ file = $loc.file; line = $loc.line; receiver = $r.receiver; method = $r.method; reason = $r.reason }
    if ($null -ne $r.correction) { $row.correction = $r.correction }
    $excluded.Add([pscustomobject]$row)
}

# 编译点：直接扫描当前产品前缀。v5 的入口编译点行号历史漂移，
# 本清单以当前扫描为准，入口表由同一份扫描派生。
$compiles = [Collections.Generic.List[object]]::new()
$productFiles = Get-ChildItem src/physics -Recurse -File -Filter '*.rs' | Where-Object {
    $_.FullName -notmatch '\\tests\\' -and $_.Name -ne 'tests.rs' -and $_.Name -notlike '*_tests.rs'
} | ForEach-Object { $_.FullName.Substring($root.Length + 1).Replace('\', '/') } | Sort-Object
foreach ($f in $productFiles) {
    $scan = Get-Rows ([IO.File]::ReadAllLines((Resolve-Path -LiteralPath $f)))
    foreach ($c in $scan.compile) { $compiles.Add([pscustomobject][ordered]@{ file = $f; line = $c.line; entry = $c.entry; expression = $null }) }
}
if ($compiles.Count -ne 89) { throw "编译点数量异常：$($compiles.Count)" }

$entries = [Collections.Generic.List[object]]::new()
foreach ($r in $inv.actual_cuda_entry_kinds) {
    $sites = [Collections.Generic.List[string]]::new()
    foreach ($c in $compiles) {
        if ($c.entry -ceq $r.entry) { [void]$sites.Add("$($c.file):$($c.line)") }
    }
    if ($sites.Count -eq 0) {
        # 非字面量入口共用动态编译点（selector 在固定 match 分支上穷举）。
        if ($r.selection -ceq 'literal') { throw "入口无编译点：$($r.entry)" }
        foreach ($c in $compiles) { if ($null -eq $c.entry) { [void]$sites.Add("$($c.file):$($c.line)") } }
        if ($sites.Count -eq 0) { throw "动态入口无编译点：$($r.entry)" }
    }
    $status = $r.migration_status
    $evidence = @($r.kernel_abi_evidence)
    if ($r.entry -ceq 'rigid_kinematics') {
        $status = 'typed_safe_wrapper'
        $rigidLines = [IO.File]::ReadAllLines((Resolve-Path -LiteralPath 'src/physics/rigid_kinematics/mod.rs'))
        $specLine = -1
        for ($i = 0; $i -lt $rigidLines.Length; $i++) { if ($rigidLines[$i] -match '^struct RigidSpec;') { $specLine = $i + 1; break } }
        if ($specLine -lt 0) { throw '未找到 RigidSpec 定义行' }
        $evidence = @("RigidSpec at src/physics/rigid_kinematics/mod.rs:$specLine")
    }
    $entries.Add([pscustomobject][ordered]@{
        entry = $r.entry
        compile_sites = @($sites)
        selection = $r.selection
        migration_status = $status
        kernel_abi_evidence = $evidence
    })
}

$families = [Collections.Generic.List[object]]::new()
foreach ($r in $inv.kernel_abi_families) {
    $parts = $r.definition.Split(':')
    $oldFile = $parts[0]; $oldLine = [int]$parts[1]
    $scopes = if ($groupMap.ContainsKey($oldFile)) { $groupMap[$oldFile] } else { @($oldFile) }
    $defLine = -1; $defFile = $null
    foreach ($gf in (Expand-Scopes $scopes)) {
        $lines = [IO.File]::ReadAllLines((Resolve-Path -LiteralPath $gf))
        for ($i = 0; $i -lt $lines.Length; $i++) {
            if ($lines[$i].Trim() -match ("(^|\s)struct " + [regex]::Escape($r.type) + "\b")) { $defFile = $gf; $defLine = $i + 1; break }
        }
        if ($defLine -ge 0) { break }
    }
    if ($defLine -lt 0) { throw "规格定义重定位失败：$($r.type) @ $oldFile" }
    $calls = [Collections.Generic.List[string]]::new()
    foreach ($s in @($r.typed_low_level_calls)) {
        $p = $s.Split(':')
        $key = "$($p[0])|$([int]$p[1])"
        if ($reloc.ContainsKey($key)) { [void]$calls.Add((Ref $reloc[$key])) }
    }
    if ($calls.Count -eq 0) {
        # 个别旧规格行号在 v5 中已漂移，按实际入口名从重建的低层行派生。
        foreach ($name in @($r.actual_cuda_entries)) {
            $hit = $lowLevel | Where-Object { $_.classification -ceq 'typed_wrapper_low_level' -and $_.entry -ceq $name } | Select-Object -First 1
            if ($null -ne $hit) { [void]$calls.Add((Ref $hit)) }
        }
    }
    if ($calls.Count -eq 0) { throw "规格发射点缺失：$($r.type)" }
    $families.Add([pscustomobject][ordered]@{
        type = $r.type
        logical_name = $r.logical_name
        definition = "$($defFile):$($defLine)"
        actual_cuda_entries = $r.actual_cuda_entries
        typed_low_level_calls = @($calls)
    })
}

$dynamic = [Collections.Generic.List[object]]::new()
foreach ($r in $inv.dynamic_compile_resolutions) {
    $p = $r.site.Split(':')
    $loc = Locate $p[0] ([int]$p[1]) $cReloc
    $dynamic.Add([pscustomobject][ordered]@{ site = (Ref $loc); selector = $r.selector; candidates = @($r.candidates); certainty = $r.certainty })
}

function Get-Counts($low, $orch, $exc, $comp, $ent, $dyn) {
    [ordered]@{
        compile_call_sites = @($comp).Count
        actual_cuda_entry_kinds = @($ent).Count
        kernel_abi_families = @($families).Count
        typed_actual_entry_kinds = @($ent | Where-Object migration_status -CNE 'unmigrated_or_unwrapped').Count
        unmigrated_actual_entry_kinds = @($ent | Where-Object migration_status -CEQ 'unmigrated_or_unwrapped').Count
        product_low_level_call_sites = @($low).Count
        typed_low_level_call_sites = @($low | Where-Object classification -CEQ 'typed_wrapper_low_level').Count
        unmigrated_low_level_call_sites = @($low | Where-Object classification -CEQ 'unmigrated_raw').Count
        wrapper_orchestration_call_sites = @($orch).Count
        excluded_non_kernel_helpers = @($exc).Count
        dynamic_compile_sites = @($dyn).Count
        unresolved_dynamic_compile_sites = 0
    }
}

$counts = Get-Counts $lowLevel $orchestration $excluded $compiles $entries $dynamic
if ($counts.unmigrated_low_level_call_sites -ne 68 -or $counts.typed_low_level_call_sites -ne 31 -or $counts.unmigrated_actual_entry_kinds -ne 55) {
    throw "计数不符预期：$($counts | ConvertTo-Json -Compress)"
}
if ($counts.compile_call_sites -ne 89 -or $counts.product_low_level_call_sites -ne 99 -or $counts.wrapper_orchestration_call_sites -ne 30) {
    throw "结构计数不符预期：$($counts | ConvertTo-Json -Compress)"
}

$priorBytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $priorRel))
$priorHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($priorBytes)).ToLowerInvariant()
$priorBlob = (& git rev-parse "HEAD:$priorRel").Trim()

$inv.task_id = $task
$inv.base_commit = $base
$inv.source_file = $priorRel
$inv.source_file_sha256 = $priorHash
$inv.source_file_blob = $priorBlob
$inv.scope = 'full prior inventory relocated to the post-split layout; three rigid_kinematics launches moved to the typed wrapper'
$inv.methodology = [pscustomobject][ordered]@{
    source = 'committed v5 inventory plus current post-split src/physics product prefix'
    raw_scan = 'the same scanner reproduced every prior launch row in the pre-split groups before relocating rows'
    prior_inventory = '../layers-m3-velocity-position-g01-correction/unmigrated-launch-inventory.json'
    strings_comments_and_cfg_test_modules_excluded = $true
    low_level_calls_separated_from_wrapper_orchestration = $true
    logical_abi_names_separated_from_actual_cuda_entries = $true
    subtraction_from_historical_counts = $false
    python_executed = $false
    limitations = @(
        'lexical scan does not perform Rust type inference',
        'rows located through the split file groups (root file plus product submodules)',
        'the probe row was relocated to the typed wrapper chain whose receiver text changed',
        'all summary counts are derived from the relocated arrays'
    )
}
$inv.manual_cross_checks = @(
    'all 99 low-level rows relocated once',
    'all 30 orchestration rows relocated once',
    'all 89 compile rows relocated once',
    'three rigid_kinematics launches moved from unmigrated_raw to typed_wrapper_low_level',
    'rigid_kinematics entry state moved to typed_safe_wrapper',
    'split layout rows rebuilt from the repository current product prefix',
    'prior inventory path, hash and blob are recorded'
)
$inv.next_candidate = [pscustomobject][ordered]@{
    selected = $false
    entry = 'rigid_com_position'
    migration_status = 'unmigrated_or_unwrapped'
    compile_sites = @()
    reason = 'rigid_kinematics is migrated; the remaining rigid launches are the COM-position plan kernel and the bare probes'
}
$inv.acceptance_boundary | Add-Member -NotePropertyName rigid_kinematics_migrated -NotePropertyValue $true -Force
$inv.acceptance_boundary | Add-Member -NotePropertyName rigid_kinematics_raw -NotePropertyValue $false -Force
$inv.acceptance_boundary | Add-Member -NotePropertyName velocity_position_launch_raw -NotePropertyValue $false -Force
$inv.acceptance_boundary | Add-Member -NotePropertyName next_candidate -NotePropertyValue 'rigid_com_position' -Force
$inv.inventory_version = 6
$inv.supersedes = $priorRel
$inv.uncertain = @()
$inv.counts = [pscustomobject]$counts
$inv.product_low_level_call_sites = @($lowLevel)
$inv.wrapper_orchestration_call_sites = @($orchestration)
$inv.excluded_non_kernel_helpers = @($excluded)
$inv.compile_sites = @($compiles)
$inv.actual_cuda_entry_kinds = @($entries)
$inv.kernel_abi_families = @($families)
$inv.dynamic_compile_resolutions = @($dynamic)

$json = ($inv | ConvertTo-Json -Depth 50).Replace("`r`n", "`n") + "`n"
$bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
$stream = [IO.File]::Open($outRel, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
[ordered]@{ inventory = $outRel; sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant(); counts = $inv.counts } | ConvertTo-Json -Depth 5
Pop-Location
