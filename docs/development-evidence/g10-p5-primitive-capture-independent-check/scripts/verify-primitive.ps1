# QA independent verification for 2026-10-26-g10-p5-primitive-capture (CPU only, no CUDA context)
$ErrorActionPreference = 'Continue'
$Root = 'D:\Rust\mjwarp-rs'
Set-Location $Root
$Out = Join-Path $Root 'target\qa-primitive'
New-Item -ItemType Directory -Force -Path $Out | Out-Null
$lines = New-Object System.Collections.ArrayList
function Add-Line([string]$s) { [void]$lines.Add($s); Write-Output $s }

# ---------- A. dispatch from frozen source ----------
$fs = Join-Path $Root 'docs\evidence\d04-complex-physics\authoring-runs\run-5\frozen-source\_src'
$driver = [IO.File]::ReadAllLines((Join-Path $fs 'collision_driver.py'))
$prim = [IO.File]::ReadAllLines((Join-Path $fs 'collision_primitive.py'))
function Extract-Block([string[]]$src, [string]$startNeedle, [string]$endNeedle) {
    $inside = $false; $out = @()
    foreach ($ln in $src) {
        if (-not $inside) { if ($ln -match $startNeedle) { $inside = $true; $out += $ln } continue }
        $out += $ln
        if ($ln -match $endNeedle) { break }
    }
    return $out
}
$tableBlock = Extract-Block $driver '^\s*MJ_COLLISION_TABLE\s*=\s*\{' '^\}'
$mapBlock = Extract-Block $prim '^\s*_PRIMITIVE_COLLISIONS\s*=\s*\{' '^\}'
Add-Line ("== frozen source: driver_lines=" + $driver.Count + " primitive_lines=" + $prim.Count)
$table = @{}
foreach ($ln in $tableBlock) {
    if ($ln -match '\(\s*GeomType\.(\w+)\s*,\s*GeomType\.(\w+)\s*\)\s*:\s*CollisionType\.(\w+)') {
        $table["$($Matches[1])-$($Matches[2])"] = $Matches[3]
    }
}
$wrappers = @{}
foreach ($ln in $mapBlock) {
    if ($ln -match '\(\s*GeomType\.(\w+)\s*,\s*GeomType\.(\w+)\s*\)\s*:\s*(\w+)') {
        $wrappers["$($Matches[1])-$($Matches[2])"] = $Matches[3]
    }
}
Add-Line ("table_entries=" + $table.Count + " wrapper_entries=" + $wrappers.Count)
Add-Line ("table_has_BOX-BOX_CONVEX=" + ($table['BOX-BOX'] -eq 'CONVEX') + " override_line_present=" + [bool](@($driver | Where-Object { $_ -match 'NATIVECCD' -and $_ -match 'BOX, GeomType\.BOX' }).Count))
$pairs = [ordered]@{
    'plane-sphere' = @('PLANE', 'SPHERE'); 'plane-capsule' = @('PLANE', 'CAPSULE');
    'plane-ellipsoid' = @('PLANE', 'ELLIPSOID'); 'plane-cylinder' = @('PLANE', 'CYLINDER');
    'plane-box' = @('PLANE', 'BOX'); 'plane-mesh' = @('PLANE', 'MESH');
    'sphere-sphere' = @('SPHERE', 'SPHERE'); 'sphere-capsule' = @('SPHERE', 'CAPSULE');
    'sphere-cylinder' = @('SPHERE', 'CYLINDER'); 'capsule-capsule' = @('CAPSULE', 'CAPSULE')
}
$dispatchOk = 0
$dispatchRows = @()
foreach ($case in $pairs.Keys) {
    $a = $pairs[$case][0]; $b = $pairs[$case][1]
    $tl = "$a-$b"
    $val = $table[$tl]
    $effective = $val
    if ($a -eq 'BOX' -and $b -eq 'BOX') {
        $override = @($driver | Where-Object { $_ -match 'collision_table\[\(GeomType\.BOX, GeomType\.BOX\)\] = CollisionType\.PRIMITIVE' }).Count -gt 0
        if ($override) { $effective = 'PRIMITIVE' }
    }
    $wrap = $wrappers[$tl]
    $ok = ($effective -eq 'PRIMITIVE') -and [bool]$wrap
    if ($ok) { $dispatchOk++ }
    $dispatchRows += [pscustomobject]@{ case = $case; pair = $tl; table = $val; after_override = $effective; wrapper = $wrap; pass = $ok }
    Add-Line ("dispatch {0,-16} table={1,-10} effective={2,-10} wrapper={3,-22} pass={4}" -f $case, $val, $effective, $wrap, $ok)
}
Add-Line ("dispatch_summary ok=$dispatchOk/10")
# compare with delivered audit table
$audit = [IO.File]::ReadAllText((Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\dispatch-audit.md'))
$auditMismatch = 0
foreach ($r in $dispatchRows) {
    $line = @($audit -split "`n" | Where-Object { $_ -match [regex]::Escape($r.pair) -and $_ -match 'PRIMITIVE' }) | Select-Object -First 1
    $hasWrapper = $line -and $line.Contains($r.wrapper)
    $hasPrimitive = $line -and $line.Contains('PRIMITIVE')
    if (-not ($hasWrapper -and $hasPrimitive)) { $auditMismatch++; Add-Line ("audit_row_mismatch " + $r.case + " line=" + $line) }
}
Add-Line ("audit_rows_matching=$($dispatchRows.Count - $auditMismatch)/10")

# ---------- B. fixtures and input shas ----------
$fixDir = Join-Path $Root 'fixtures\g10-p5-primitive'
$inputs = Get-Content (Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\inputs-sha256.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$ref = Get-Content (Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\upstream-primitive-gpu.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$shaMismatch = 0; $bytesMismatch = 0; $xmlFlagMissing = 0; $qposMismatch = 0; $caseMissing = 0
function F32Bits([string]$token) {
    $d = [double]::Parse($token, [System.Globalization.CultureInfo]::InvariantCulture)
    $f = [single]$d
    return [BitConverter]::ToUInt32([BitConverter]::GetBytes($f), 0)
}
foreach ($file in @('plane-sphere.xml', 'plane-capsule.xml', 'plane-ellipsoid.xml', 'plane-cylinder.xml', 'plane-box.xml', 'plane-mesh.xml', 'sphere-sphere.xml', 'sphere-capsule.xml', 'sphere-cylinder.xml', 'capsule-capsule.xml', 'tetra.obj')) {
    $p = Join-Path $fixDir $file
    $sha = (Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLower()
    $rec = $inputs.files.$file
    $bytes = (Get-Item -LiteralPath $p).Length
    if (-not $rec -or $sha -ne $rec.sha256) { $shaMismatch++; Add-Line ("INPUT_SHA_MISMATCH " + $file + " computed=" + $sha) }
    if (-not $rec -or $bytes -ne $rec.bytes) { $bytesMismatch++; Add-Line ("INPUT_BYTES_MISMATCH " + $file + " computed=" + $bytes) }
}
Add-Line ("inputs_files=" + @($inputs.files.PSObject.Properties).Count + " sha_mismatch=$shaMismatch bytes_mismatch=$bytesMismatch")
$caseList = @($ref.cases.PSObject.Properties.Name)
Add-Line ("reference_cases=" + $caseList.Count + " : " + ($caseList -join ','))
foreach ($name in $caseList) {
    $case = $ref.cases.$name
    $xmlPath = Join-Path $Root ($case.xml -replace '/', '\')
    $doc = New-Object System.Xml.XmlDocument
    $doc.Load($xmlPath)
    $flag = $doc.SelectSingleNode('//flag[@nativeccd]')
    $flagValue = if ($flag) { $flag.GetAttribute('nativeccd') } else { '' }
    if ($flagValue -ne 'disable') { $xmlFlagMissing++; Add-Line ("NATIVECCD_MISSING " + $name + " value=" + $flagValue) }
    $key = $doc.SelectSingleNode('//key[@name="capture"]')
    $qposText = if ($key) { $key.GetAttribute('qpos') } else { '' }
    $tokens = @($qposText -split '\s+' | Where-Object { $_ })
    $bits = @($tokens | ForEach-Object { F32Bits $_ })
    $refBits = @($case.qpos_bits | ForEach-Object { [uint32][int64]$_ })
    $bitsEq = ($bits.Count -eq $refBits.Count) -and (@(0..($bits.Count - 1) | Where-Object { $bits[$_] -ne $refBits[$_] }).Count -eq 0)
    $xmlSha = (Get-FileHash -LiteralPath $xmlPath -Algorithm SHA256).Hash.ToLower()
    $xmlShaOk = $xmlSha -eq $case.xml_sha256
    if (-not $bitsEq -or -not $xmlShaOk) { $qposMismatch++; Add-Line ("QPOS_OR_XMLSHA_MISMATCH " + $name + " bitsEq=" + $bitsEq + " xmlShaOk=" + $xmlShaOk) }
    Add-Line ("case {0,-16} sha_ok={1} qpos_bits_ok={2} nativeccd={3} gpu_nacon={4}" -f $name, $xmlShaOk, $bitsEq, $flagValue, $case.output.count)
}

# ---------- C. provenance vs local machine ----------
$smi = & nvidia-smi --query-gpu=name,memory.total,driver_version,compute_cap --format=csv,noheader 2>&1 | Select-Object -First 1
$smiParts = @(([string]$smi) -split ',' | ForEach-Object { $_.Trim() })
Add-Line ("local_gpu name='$($smiParts[0])' mem='$($smiParts[1])' driver='$($smiParts[2])' cc='$($smiParts[3])'")
$p = $ref.provenance
Add-Line ("provenance_gpu name='$($p.gpu.name)' mem='$($p.gpu.memory_total)' driver='$($p.gpu.driver)' cc='$($p.gpu.compute_capability)'")
Add-Line ("gpu_name_match=" + ($p.gpu.name -eq $smiParts[0]) + " driver_match=" + ($p.gpu.driver -eq $smiParts[2]) + " mem_match=" + ($p.gpu.memory_total -eq $smiParts[1]) + " cc_match=" + ($p.gpu.compute_capability -eq $smiParts[3]))
$py = $p.interpreter.path
$pyExists = Test-Path -LiteralPath $py
$pyVersion = if ($pyExists) { (& $py --version 2>&1 | Select-Object -First 1) } else { 'MISSING' }
Add-Line ("interpreter exists=$pyExists version='$pyVersion' recorded='$($p.interpreter.version)'")
$repoScript = Join-Path $Root 'scripts\capture_primitive_gpu.py'
$repoScriptSha = (Get-FileHash -LiteralPath $repoScript -Algorithm SHA256).Hash.ToLower()
Add-Line ("script_sha_recorded='$($p.authoring_script_sha256)' computed='$repoScriptSha' match=" + ($p.authoring_script_sha256 -eq $repoScriptSha))
Add-Line ("kernel_cache_root='$($p.kernel_cache_root)' relative=" + (-not ($p.kernel_cache_root -match '^[A-Za-z]:')) + " exists=" + (Test-Path -LiteralPath (Join-Path $Root ($p.kernel_cache_root -replace '/', '\'))))
$t3 = Join-Path $Root 'target\g10-p5-primitive-capture\attempt3'
foreach ($c in @('warp-cache-a', 'warp-cache-b')) {
    $cp = Join-Path $t3 $c
    Add-Line ("cache $c exists=" + (Test-Path $cp) + " files=" + @(Get-ChildItem $cp -Recurse -File -ErrorAction SilentlyContinue).Count)
}
$capAt = [datetime]::Parse($p.captured_at)
$samples = @(Import-Csv (Join-Path $t3 'samples.csv'))
$firstTs = [datetime]::Parse($samples[0].timestamp); $lastTs = [datetime]::Parse($samples[-1].timestamp)
Add-Line ("captured_at=$($p.captured_at) samples_first=$($samples[0].timestamp) samples_last=$($samples[-1].timestamp)")
Add-Line ("captured_at_within_sampled_window=" + ($capAt -ge $firstTs -and $capAt -le $lastTs) + " delta_vs_first_s=" + [math]::Round(($capAt - $firstTs).TotalSeconds, 1))
Add-Line ("window_related_file_mtimes: baseline=" + (Get-Item (Join-Path $t3 'baseline.txt')).LastWriteTime.ToString('o') + " referenceA=" + (Get-Item (Join-Path $t3 'upstream-primitive-gpu-A.json')).LastWriteTime.ToString('o') + " referenceB=" + (Get-Item (Join-Path $t3 'upstream-primitive-gpu-B.json')).LastWriteTime.ToString('o'))

# ---------- D. two runs ----------
$a = Join-Path $t3 'upstream-primitive-gpu-A.json'; $b = Join-Path $t3 'upstream-primitive-gpu-B.json'
$final = Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\upstream-primitive-gpu.json'
$shaA = (Get-FileHash -LiteralPath $a -Algorithm SHA256).Hash.ToLower()
$shaB = (Get-FileHash -LiteralPath $b -Algorithm SHA256).Hash.ToLower()
$shaF = (Get-FileHash -LiteralPath $final -Algorithm SHA256).Hash.ToLower()
$aBytes = [IO.File]::ReadAllBytes($a); $bBytes = [IO.File]::ReadAllBytes($b)
$sameBytes = ($aBytes.Length -eq $bBytes.Length)
if ($sameBytes) { for ($i = 0; $i -lt $aBytes.Length; $i++) { if ($aBytes[$i] -ne $bBytes[$i]) { $sameBytes = $false; break } } }
Add-Line ("two_run shaA=$shaA shaB=$shaB equal=" + ($shaA -eq $shaB) + " byte_equal=$sameBytes final_equals_A=" + ($shaF -eq $shaA) + " sizeA=$($aBytes.Length) sizeB=$($bBytes.Length) sizeFinal=" + (Get-Item $final).Length)
$runA = [IO.File]::ReadAllText((Join-Path $t3 'run-a.log'))
$runB = [IO.File]::ReadAllText((Join-Path $t3 'run-b.log'))
$cacheA = ([regex]::Match($runA, 'kernel_cache_dir: (.+)')).Groups[1].Value.Trim()
$cacheB = ([regex]::Match($runB, 'kernel_cache_dir: (.+)')).Groups[1].Value.Trim()
Add-Line ("runA_cache='$cacheA' runB_cache='$cacheB' different=" + ($cacheA -ne $cacheB))
Add-Line ("runA_wrote=" + $runA.Contains('wrote ') + " runB_wrote=" + $runB.Contains('wrote ') + " runA_count_lines=" + @($runA -split "`n" | Where-Object { $_ -match 'count=\d+ two_captures=True' }).Count + " runB_count_lines=" + @($runB -split "`n" | Where-Object { $_ -match 'count=\d+ two_captures=True' }).Count)
# attempts
$t2 = Join-Path $Root 'target\g10-p5-primitive-capture\attempt2'
$t1 = Join-Path $Root 'target\g10-p5-primitive-capture\attempt1'
if (Test-Path $t2) {
    $a2 = Get-Content (Join-Path $t2 'upstream-primitive-gpu-A.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    $b2 = Get-Content (Join-Path $t2 'upstream-primitive-gpu-B.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    $shaA2 = (Get-FileHash (Join-Path $t2 'upstream-primitive-gpu-A.json') -Algorithm SHA256).Hash.ToLower()
    $shaB2 = (Get-FileHash (Join-Path $t2 'upstream-primitive-gpu-B.json') -Algorithm SHA256).Hash.ToLower()
    Add-Line ("attempt2 shaA=$shaA2 shaB=$shaB2 equal=" + ($shaA2 -eq $shaB2) + " cacheA='$($a2.provenance.kernel_cache_dir)' cacheB='$($b2.provenance.kernel_cache_dir)' absoluteA=" + ($a2.provenance.kernel_cache_dir -match '^[A-Za-z]:'))
}
if (Test-Path $t1) {
    $t1log = [IO.File]::ReadAllText((Join-Path $t1 'run-a.log'))
    Add-Line ("attempt1_runA_log_first_lines=" + (($t1log -split "`n" | Select-Object -First 6) -join ' / '))
}

# ---------- E. window facts ----------
$count = $samples.Count
$intervals = @()
for ($i = 1; $i -lt $count; $i++) { $intervals += ([datetime]::Parse($samples[$i].timestamp) - [datetime]::Parse($samples[$i - 1].timestamp)).TotalMilliseconds }
$sorted = @($intervals | Sort-Object)
$median = if ($sorted.Count) { $sorted[[int]([math]::Floor($sorted.Count / 2))] } else { 0 }
$used = @($samples | ForEach-Object { [int]$_.used_mib }); $free = @($samples | ForEach-Object { [int]$_.free_mib }); $util = @($samples | ForEach-Object { [int]$_.util_percent })
$peak = ($used | Measure-Object -Maximum).Maximum; $fmin = ($free | Measure-Object -Minimum).Minimum; $umax = ($util | Measure-Object -Maximum).Maximum
$mono = $true
for ($i = 1; $i -lt $count; $i++) { if ([datetime]::Parse($samples[$i].timestamp) -lt [datetime]::Parse($samples[$i - 1].timestamp)) { $mono = $false } }
Add-Line ("samples=$count monotonic=$mono median_interval_ms=$([math]::Round($median,1)) peak=$peak free_min=$fmin util_max=$umax")
$facts = [IO.File]::ReadAllText((Join-Path $t3 'facts.txt'))
Add-Line ("facts_samples_line=" + ((@($facts -split "`n" | Where-Object { $_ -match '^sampler_samples=' }) -join '')))
Add-Line ("facts_run_line=" + ((@($facts -split "`n" | Where-Object { $_ -match '^runA_exit=' }) -join '')))
$base = [IO.File]::ReadAllText((Join-Path $t3 'baseline.txt'))
$appsBefore = ([regex]::Match(([IO.File]::ReadAllText((Join-Path $t3 'facts.txt'))), 'post_compute_apps_raw=(.+)')).Groups[1].Value
Add-Line ("baseline_file_first_line=" + (($base -split "`n")[1]))
Add-Line ("post_compute_apps_contains_task=" + [bool]($appsBefore -match 'g10_p5|primitive|capture_primitive|mjwarp-parallel') + " entry_count=" + (@($appsBefore -split '\|').Count))
$guardLog = [IO.File]::ReadAllText((Join-Path $t3 'guard.log'))
Add-Line ("guard_post_line=" + ((@($guardLog -split "`n" | Where-Object { $_ -match 'guard_sample phase=post' }) -join '')))
Add-Line ("zero_rust_compile_evidence: run logs mention cargo/rustc=" + [bool](($runA + $runB) -match 'cargo|rustc'))

Add-Line ("delivery_evidence_files=" + @(Get-ChildItem (Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture') -Recurse -File).Count)

# ---------- F. scope / old refs / evidence consistency ----------
$diffNames = @(git diff --name-only 767e420e..44a53a79)
Add-Line ("diff_files=" + $diffNames.Count + " non_evidence_scripts_fixture=" + @($diffNames | Where-Object { $_ -match '^scripts/|^fixtures/' }).Count + " evidence=" + @($diffNames | Where-Object { $_ -match '^docs/development-evidence/' }).Count + " normify=" + @($diffNames | Where-Object { $_ -match '^\.normify/' }).Count + " other=" + @($diffNames | Where-Object { $_ -notmatch '^scripts/|^fixtures/|^docs/development-evidence/|^\.normify/' }).Count)
$oldRefs = @('docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json', 'docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json', 'docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json')
$changedOldRefs = @(git diff --name-only 767e420e..44a53a79 -- $oldRefs)
Add-Line ("old_reference_files_changed=" + $changedOldRefs.Count + " " + ($changedOldRefs -join ','))
$abiChanged = @(git diff --name-only 767e420e..44a53a79 -- src/runtime/collision_gpu/abi.rs)
Add-Line ("abi_changed=" + $abiChanged.Count)
$fixtureChanged = @(git diff --name-only 767e420e..44a53a79 -- fixtures) | Where-Object { $_ -notmatch '^fixtures/g10-p5-primitive/' }
Add-Line ("existing_fixtures_changed_outside_new_dir=" + @($fixtureChanged).Count + " " + (@($fixtureChanged) -join ','))
$readme = [IO.File]::ReadAllText((Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\README.md'))
$tNotOpened = -join (0x7A97,0x53E3,0x72B6,0x6001,0xFF1A,0x672A,0x5F00,0x7A97 | ForEach-Object { [char]$_ })
$tCriterion = -join (0x53C2,0x8003,0x5C1A,0x4E0D,0x662F,0x5224,0x636E | ForEach-Object { [char]$_ })
$tNoHostCross = (-join (0x4E0D,0x505A | ForEach-Object { [char]$_ })) + ' host ' + (-join (0x4EA4,0x53C9 | ForEach-Object { [char]$_ }))
$tFixtureBaseline = (-join (0x53EA,0x4F5C | ForEach-Object { [char]$_ })) + ' fixture ' + (-join (0x6709,0x6548,0x6027,0x57FA,0x7EBF | ForEach-Object { [char]$_ }))
$tGpuCaption = 'GPU ' + (-join (0x6355,0x83B7 | ForEach-Object { [char]$_ }))
$tCpuPrecheck = 'CPU ' + (-join (0x9884,0x68C0,0x4E3A | ForEach-Object { [char]$_ }))
$readmeStaleHeader = $readme.Contains($tNotOpened)
$readmeWindow1940 = $readme -match '19:40'
Add-Line ("readme_stale_not_opened=$readmeStaleHeader readme_mentions_1940=$readmeWindow1940 readme_mentions_1923=" + ($readme -match '19:23'))
$readmeBoundaries = "criterion_not_yet=" + ($readme.Contains($tCriterion) -or ($readme -match 'not.*criterion')) + " no_host_cross=" + $readme.Contains($tNoHostCross) + " nacon_only_fixture=" + $readme.Contains($tFixtureBaseline)
Add-Line ($readmeBoundaries)
$readmeCapCapsule = ($readme -match 'capsule-capsule') -and ($readme -match 'nacon=1')
Add-Line ("readme_capsule_capsule_observation=" + $readmeCapCapsule + " text_has_gpu1_cpu2=" + (($readme -match ('nacon=1[^\n]*' + $tCpuPrecheck + ' 2')) -or $readme.Contains($tGpuCaption + ' nacon=1')))
$receipt = Get-Content (Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\receipt.json') -Raw -Encoding UTF8 | ConvertFrom-Json
Add-Line ("receipt_head='$($receipt.head)' receipt_status='$($receipt.status)' receipt_boundary_not_opened=" + [bool]($receipt.boundaries -match 'not opened yet'))
$precheck = [IO.File]::ReadAllText((Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\precheck-cpu.md'))
Add-Line ("precheck_table_separator_merged=" + [bool]($precheck -match '\| --- \| --- \| --- \| --- \| --- \|\| plane-sphere'))
$auditLines = [IO.File]::ReadAllLines((Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture\dispatch-audit.md'))
Add-Line ("audit_line_47_matches=" + [bool](@($auditLines | Where-Object { $_ -match '^47: MJ_COLLISION_TABLE' }).Count))
Add-Line ("audit_nativeccd_override_matches=" + [bool](@($auditLines | Where-Object { $_ -match '910:' -and $_ -match 'DisableBit.NATIVECCD' }).Count) + " override_apply_line=" + [bool](@($auditLines | Where-Object { $_ -match '912:' -and $_ -match 'BOX, GeomType.BOX' }).Count))

# ---------- G. script diff vs existing GPU capture script ----------
$oldCapture = Join-Path $Root 'docs\development-evidence\g10-p5-g2-gpu-reference-capture\capture_mjwarp_f32_gpu.py'
$newCapture = Join-Path $Root 'scripts\capture_primitive_gpu.py'
$oldCmp = Join-Path $Root 'docs\development-evidence\g10-p5-g2-gpu-reference-capture\compare_references.py'
$newCmp = Join-Path $Root 'scripts\compare_primitive_capture.py'
function Diff-Files([string]$x, [string]$y, [string]$outFile) {
    $prev = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
    $d = & git diff --no-index --unified=0 $x $y 2>&1 | ForEach-Object { [string]$_ }
    $ErrorActionPreference = $prev
    $d | Set-Content -LiteralPath $outFile -Encoding utf8
    $added = @($d | Where-Object { $_ -match '^\+' -and $_ -notmatch '^\+\+\+' }).Count
    $removed = @($d | Where-Object { $_ -match '^-' -and $_ -notmatch '^---' }).Count
    return "$added added / $removed removed"
}
if (Test-Path $oldCapture) { Add-Line ("capture_script_diff_vs_g2_gpu: " + (Diff-Files $oldCapture $newCapture (Join-Path $Out 'diff-capture.txt'))) } else { Add-Line 'old_capture_script_missing' }
if (Test-Path $oldCmp) { Add-Line ("compare_script_diff_vs_g2: " + (Diff-Files $oldCmp $newCmp (Join-Path $Out 'diff-compare.txt'))) } else { Add-Line 'old_compare_script_missing' }
function Extract-Function([string]$path, [string]$name) {
    $text = [IO.File]::ReadAllText($path) -split "`n"
    $out = @(); $inside = $false
    foreach ($ln in $text) {
        if (-not $inside) { if ($ln -match "^\s*def\s+$name\(") { $inside = $true; $out += $ln }; continue }
        if ($ln -match '^\S' -and $ln -notmatch '^\s') { break }
        $out += $ln
    }
    return ($out -join "`n")
}
$newCapFn = Extract-Function $newCapture 'capture'
$oldCapFn = Extract-Function $oldCapture 'capture'
Add-Line ("capture_function_text_identical=" + ($newCapFn.Trim() -eq $oldCapFn.Trim()) + " new_len=" + $newCapFn.Length + " old_len=" + $oldCapFn.Length)
$fieldsNew = @(([IO.File]::ReadAllText($newCapture)) -split "`n" | Where-Object { $_ -match '^FLOAT_FIELDS|^INT_FIELDS' })
$fieldsOld = @(([IO.File]::ReadAllText($oldCapture)) -split "`n" | Where-Object { $_ -match '^FLOAT_FIELDS|^INT_FIELDS' })
Add-Line ("field_lists_identical=" + (($fieldsNew -join "`n") -eq ($fieldsOld -join "`n")))

$lines | Set-Content -LiteralPath (Join-Path $Out 'verify-report.txt') -Encoding utf8
Add-Line 'VERIFY DONE'
