# QA re-check after fix commit 43ca9e9f (CPU only)
$ErrorActionPreference = 'Continue'
$Root = 'D:\Rust\mjwarp-rs'
Set-Location $Root
$Out = Join-Path $Root 'target\qa-primitive-recheck'
New-Item -ItemType Directory -Force $Out | Out-Null
$lines = New-Object System.Collections.ArrayList
function Add-Line([string]$s) { [void]$lines.Add($s); Write-Output $s }
$ev = Join-Path $Root 'docs\development-evidence\g10-p5-primitive-capture'
$t4 = Join-Path $Root 'target\g10-p5-primitive-capture\attempt4'

# ---- provenance / reference ----
$ref = Get-Content (Join-Path $ev 'upstream-primitive-gpu.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$p = $ref.provenance
Add-Line ("captured_at=$($p.captured_at)")
Add-Line ("kernel_cache_parent='$($p.kernel_cache_parent)' relative=" + (-not ($p.kernel_cache_parent -match '^[A-Za-z]:')) + " exists=" + (Test-Path -LiteralPath (Join-Path $Root ($p.kernel_cache_parent -replace '/', '\'))))
Add-Line ("script_sha_recorded='$($p.authoring_script_sha256)' computed='$((Get-FileHash -LiteralPath (Join-Path $Root 'scripts\capture_primitive_gpu.py') -Algorithm SHA256).Hash.ToLower())'")
$capAt = [datetime]::Parse($p.captured_at)
# window bounds: facts + samples + mtimes
$facts = [IO.File]::ReadAllText((Join-Path $t4 'facts.txt'))
$wsMatch = [regex]::Match($facts, 'window_start=([^\s]+)'); $weMatch = [regex]::Match($facts, 'window_end=([^\s]+)')
$windowStart = [datetime]::Parse($wsMatch.Groups[1].Value); $windowEnd = [datetime]::Parse($weMatch.Groups[1].Value)
Add-Line ("window_start=$($windowStart.ToString('o')) window_end=$($windowEnd.ToString('o')) captured_at_in_window=" + ($capAt -ge $windowStart -and $capAt -le $windowEnd) + " captured_at_equals_window_start=" + ($capAt -eq $windowStart))
$samples = @(Import-Csv (Join-Path $t4 'samples.csv'))
$firstTs = [datetime]::Parse($samples[0].timestamp); $lastTs = [datetime]::Parse($samples[-1].timestamp)
Add-Line ("samples=$($samples.Count) first=$($samples[0].timestamp) last=$($samples[-1].timestamp)")
Add-Line ("captured_at_between_first_last_sample=" + ($capAt -ge $firstTs -and $capAt -le $lastTs) + " captured_at_before_first_sample_s=" + [math]::Round(($firstTs - $capAt).TotalSeconds, 3))
Add-Line ("window_bounds_contain_samples=" + ($windowStart -le $firstTs -and $windowEnd -ge $lastTs))
$mt = @{}
foreach ($n in 'baseline.txt', 'run-a.log', 'upstream-primitive-gpu-A.json', 'run-b.log', 'upstream-primitive-gpu-B.json', 'samples.csv', 'facts.txt', 'compare.log') { $mt[$n] = (Get-Item (Join-Path $t4 $n)).LastWriteTime.ToString('o') }
foreach ($k in $mt.Keys) { Add-Line ("mtime $k = $($mt[$k])") }

# ---- two-run ----
$a = Join-Path $t4 'upstream-primitive-gpu-A.json'; $bb = Join-Path $t4 'upstream-primitive-gpu-B.json'
$f = Join-Path $ev 'upstream-primitive-gpu.json'
$shaA = (Get-FileHash $a -Algorithm SHA256).Hash.ToLower(); $shaB = (Get-FileHash $bb -Algorithm SHA256).Hash.ToLower(); $shaF = (Get-FileHash $f -Algorithm SHA256).Hash.ToLower()
$ab = [IO.File]::ReadAllBytes($a); $bb2 = [IO.File]::ReadAllBytes($bb); $eq = $ab.Length -eq $bb2.Length
if ($eq) { for ($i = 0; $i -lt $ab.Length; $i++) { if ($ab[$i] -ne $bb2[$i]) { $eq = $false; break } } }
Add-Line ("two_run shaA=$shaA shaB=$shaB equal=" + ($shaA -eq $shaB) + " byte_equal=$eq final_equals_A=" + ($shaF -eq $shaA) + " size=$($ab.Length)")
Add-Line ("per_case_two_captures_all_true=" + (@($ref.cases.PSObject.Properties | Where-Object { $_.Value.two_captures_bitwise_identical -ne $true }).Count -eq 0))
# data unchanged vs pre-fix reference
$oldRaw = & git show '44a53a79:docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json' 2>$null
$old = ($oldRaw | Out-String) | ConvertFrom-Json
$oldCases = $old.cases | ConvertTo-Json -Depth 20 -Compress
$newCases = $ref.cases | ConvertTo-Json -Depth 20 -Compress
Add-Line ("cases_data_identical_to_prefix_reference=" + ($oldCases -eq $newCases) + " old_captured_at=$($old.provenance.captured_at) new_captured_at=$($p.captured_at)")
# ---- caches ----
foreach ($c in 'warp-cache-a', 'warp-cache-b') {
    $cp = Join-Path $t4 $c
    Add-Line ("cache $c exists=" + (Test-Path $cp) + " files=" + @(Get-ChildItem $cp -Recurse -File -ErrorAction SilentlyContinue).Count)
}
$runA = [IO.File]::ReadAllText((Join-Path $t4 'run-a.log')); $runB = [IO.File]::ReadAllText((Join-Path $t4 'run-b.log'))
Add-Line ("runA_cache=" + ([regex]::Match($runA, 'kernel_cache_dir: (.+)')).Groups[1].Value.Trim())
Add-Line ("runB_cache=" + ([regex]::Match($runB, 'kernel_cache_dir: (.+)')).Groups[1].Value.Trim())
Add-Line ("runA_warp_init=" + $runA.Contains('Warp 1.15.0 initialized') + " runB_warp_init=" + $runB.Contains('Warp 1.15.0 initialized') + " runA_wrote=" + $runA.Contains('wrote ') + " runB_wrote=" + $runB.Contains('wrote ') + " runA_counts=" + @($runA -split "`n" | Where-Object { $_ -match 'count=\d+ two_captures=True' }).Count + " runB_counts=" + @($runB -split "`n" | Where-Object { $_ -match 'count=\d+ two_captures=True' }).Count)
Add-Line ("facts_run_line=" + ((@($facts -split "`n" | Where-Object { $_ -match '^runA_exit=' }) -join '')))
Add-Line ("zero_rust_compile=" + -not (($runA + $runB) -match 'cargo|rustc'))
# ---- window stats ----
$used = @($samples | ForEach-Object { [int]$_.used_mib }); $free = @($samples | ForEach-Object { [int]$_.free_mib }); $util = @($samples | ForEach-Object { [int]$_.util_percent })
$peak = ($used | Measure-Object -Maximum).Maximum; $fmin = ($free | Measure-Object -Minimum).Minimum; $umax = ($util | Measure-Object -Maximum).Maximum
$intervals = @(); for ($i = 1; $i -lt $samples.Count; $i++) { $intervals += ([datetime]::Parse($samples[$i].timestamp) - [datetime]::Parse($samples[$i - 1].timestamp)).TotalMilliseconds }
$sorted = @($intervals | Sort-Object); $median = $sorted[[int]([math]::Floor($sorted.Count / 2))]
$mono = $true; for ($i = 1; $i -lt $samples.Count; $i++) { if ([datetime]::Parse($samples[$i].timestamp) -lt [datetime]::Parse($samples[$i - 1].timestamp)) { $mono = $false } }
Add-Line ("samples_stats peak=$peak free_min=$fmin util_max=$umax monotonic=$mono median_interval_ms=$([math]::Round($median,1))")
Add-Line ("facts_sampler_line=" + ((@($facts -split "`n" | Where-Object { $_ -match '^sampler_samples=' }) -join '')))
$postApps = [regex]::Match($facts, 'post_compute_apps_raw=(.+)').Groups[1].Value
Add-Line ("post_compute_apps_entries=" + @($postApps -split '\|').Count + " task_process_present=" + [bool]($postApps -match 'g10_p5|primitive|capture_primitive|mjwarp-parallel'))

# ---- scope of fix commit ----
$changed = @(git diff --name-only 44a53a79..43ca9e9f)
Add-Line ("fix_commit_files=" + $changed.Count + " : " + ($changed -join ','))
$product = @($changed | Where-Object { $_ -match '^src/|^tests/|^Cargo|^fixtures/|^\.normify/gates\.toml' })
Add-Line ("fix_commit_product_or_fixture_changes=" + $product.Count + " " + ($product -join ','))
$oldRefs = @('docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json', 'docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json', 'docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json')
$oldDiff = @(git diff --name-only 767e420e..43ca9e9f -- $oldRefs)
Add-Line ("old_refs_changed_since_767e420e=" + $oldDiff.Count + " " + ($oldDiff -join ','))
$a1 = git rev-parse 767e420e:src/runtime/collision_gpu/abi.rs; $a2 = git rev-parse 43ca9e9f:src/runtime/collision_gpu/abi.rs
Add-Line ("abi_blob_same=" + ($a1 -eq $a2))

# ---- delivery evidence O1-O8 closure text checks ----
$readme = [IO.File]::ReadAllText((Join-Path $ev 'README.md'))
$tCriterion = -join (0x53C2,0x8003,0x5C1A,0x4E0D,0x662F,0x5224,0x636E | ForEach-Object { [char]$_ })
$tAttempt1 = -join (0x6B8B,0x7559,0x5B57,0x9762 | ForEach-Object { [char]$_ })
Add-Line ("readme_attempt1_line105_syntaxerror=" + (($readme -match 'line 105') -and ($readme -match 'SyntaxError')))
Add-Line ("readme_criterion_literal=" + $readme.Contains($tCriterion))
$tWindowStatus = -join (0x7A97,0x53E3,0x72B6,0x6001 | ForEach-Object { [char]$_ })
$tAttempt3Valid = 'attempt3 ' + (-join (0x4E3A,0x552F,0x4E00,0x6709,0x6548,0x8F6E | ForEach-Object { [char]$_ }))
$tAttempt4Valid = 'attempt4 ' + (-join (0x4E3A,0x66F4,0x6B63,0x7A97,0x53E3,0x552F,0x4E00,0x6709,0x6548,0x8F6E | ForEach-Object { [char]$_ }))
$tSessions2 = 'GPU ' + (-join (0x7A97,0x53E3,0x4F1A,0x8BDD,0x5408,0x8BA1 | ForEach-Object { [char]$_ })) + ' 2'
Add-Line ("readme_header_status=" + ((@($readme -split "`n" | Where-Object { $_ -match $tWindowStatus }) -join ' ')))
Add-Line ("readme_attempt3_sole_valid_claim=" + [bool]($readme.Contains($tAttempt3Valid)) + " readme_attempt4_sole_valid_claim=" + [bool]($readme.Contains($tAttempt4Valid)) + " readme_window_sessions_2=" + [bool]($readme.Contains($tSessions2)))
Add-Line ("readme_kernel_cache_parent_attempt3_mention=" + [bool]($readme -match 'kernel_cache_parent[^\n]*attempt3') + " attempt4_cache_mention=" + [bool]($readme -match 'attempt4/warp-cache-a'))
Add-Line ("readme_runner_independence=" + [bool]($readme -match 'runner[^\n]*O8') + " readme_o7_sessions=" + [bool]($readme -match 'O7[^\n]*8 / 100 / 68'))
Add-Line ("readme_capsule_observation=" + [bool]($readme -match 'capsule-capsule.*nacon=1') + " readme_b1_gpu_reference_note=" + [bool]($readme -match 'B1[^\n]*GPU'))
$precheck = [IO.File]::ReadAllText((Join-Path $ev 'precheck-cpu.md'))
Add-Line ("precheck_table_fixed=" + (-not ($precheck -match '\|\s*---\s*\|\s*---\s*\|\s*---\s*\|\s*---\s*\|\s*---\s*\|\|\s*plane-sphere')) + " precheck_criterion=" + $precheck.Contains($tCriterion))
$receipt = Get-Content (Join-Path $ev 'receipt.json') -Raw -Encoding UTF8 | ConvertFrom-Json
Add-Line ("receipt_head='$($receipt.head)' status='$($receipt.status)' gpu_windows=$($receipt.gpu_windows)")
Add-Line ("receipt_captured_at='$($receipt.window.captured_at)'")
Add-Line ("receipt_attempts=" + (@($receipt.window.attempts) -join ' | '))
Add-Line ("receipt_observation=" + $receipt.window.observation)
Add-Line ("receipt_has_kernel_cache_parent=" + [bool]($receipt.PSObject.Properties.Name -contains 'kernel_cache_parent'))
$rj = $receipt | ConvertTo-Json -Depth 6 -Compress; Add-Line ("receipt_raw=" + $rj.Substring(0, [math]::Min(400, $rj.Length)))
# evidence quality
$bad = 0
foreach ($file in (Get-ChildItem $ev -Recurse -File)) {
    $bytes = [IO.File]::ReadAllBytes($file.FullName)
    $bom = ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF)
    $text = [IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)
    $lc = ($text -split "`n").Count
    if ($bom -or $lc -gt 500) { Add-Line ("BAD $($file.Name) bom=$bom lines=$lc"); $bad++ }
}
Add-Line ("delivery_evidence_files=" + @(Get-ChildItem $ev -Recurse -File).Count + " bad=$bad")

$lines | Set-Content -LiteralPath (Join-Path $Out 'recheck-report.txt') -Encoding utf8
Add-Line 'RECHECK DONE'
