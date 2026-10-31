# Read-only self-checks for QA review card 2026-10-31-render-sdf-sphere-dot-a-order-refix-check
# Usage: pwsh -File selfcheck-b.ps1   (run from repo root D:\Rust\mjwarp-rs)
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
Set-Location $root
$out = New-Object System.Collections.Generic.List[string]
function L($s) { $out.Add($s); Write-Output $s }

L "# selfcheck B: sphere a-order refix (pin fa604008)"
L "generated_at=$(Get-Date -Format o)"
L "head=$(git rev-parse HEAD)"
L ""

L "## 1 scope: slice commit fa604008"
L (git show --stat --format="" fa604008 | Out-String).Trim()
$sf = (git diff --name-only 8bc0f2b3..fa604008 -- src | Out-String).Trim()
$sf2 = $sf -replace "`n", ", "
L "src_files=[$sf2]"
$tfc = (git diff --name-only 8bc0f2b3..fa604008 -- tests fixtures scripts Cargo.toml Cargo.lock docs/development-evidence/render-bvh/sdf-criterion-v2 | Out-String).Trim()
L "tests_fixtures_criterion_diff=[$tfc]"
L ""

L "## 2 render_math.cu shape / line counts"
foreach ($rev in @('8bc0f2b3','fa604008')) {
  $t = @(git show "${rev}:src/render/kernel/cuda/render_math.cu")
  $blank = @($t | Where-Object { $_.Trim() -eq '' }).Count
  $cmt = @($t | Where-Object { $_.Trim().StartsWith('//') }).Count
  L "  $rev render_math_lines=$($t.Count) blank=$blank comment_only=$cmt code=$($t.Count - $blank - $cmt)"
}
L "dot3_ch0_refs_at_pin:"
git grep -n "dot3_ch0" fa604008 -- src | ForEach-Object { L "  $_" }
L "a_dot3_ba_at_pin:"
git grep -n "float a = dot3_ba" fa604008 -- src/render/kernel/cuda | ForEach-Object { L "  $_" }
L "b_dd_det_kept:"
git grep -n "float b = dot3_ba\|float dd = dot3_ba\|__fmaf_rn(b, b" fa604008 -- src/render/kernel/cuda/render_math.cu | ForEach-Object { L "  $_" }
L ""

L "## 3 render_shade.cu comment-only change"
L (git diff 8bc0f2b3..fa604008 -- src/render/kernel/cuda/render_shade.cu | Out-String).Trim()
$sh = @(git show fa604008:src/render/kernel/cuda/render_shade.cu)
L "render_shade_lines=$($sh.Count) line10=$($sh[9].Trim()) line14=$($sh[13].Trim())"
L ""

L "## 4 same diff as 8126868c (file-byte level)"
L "blob_8126868c=$(git rev-parse 8126868c:src/render/kernel/cuda/render_math.cu)"
L "blob_fa604008=$(git rev-parse fa604008:src/render/kernel/cuda/render_math.cu)"
L "blob_before_8bc0f2b3=$(git rev-parse 8bc0f2b3:src/render/kernel/cuda/render_math.cu)"
$p1 = (git diff 8126868c^..8126868c -- src/render/kernel/cuda/render_math.cu) -join "`n"
$p2 = (git diff 8bc0f2b3..fa604008 -- src/render/kernel/cuda/render_math.cu) -join "`n"
L "render_math_patch_identical_to_original_fix=$($p1 -eq $p2)"
L ""

L "## 5 kernel identity (independent recompute from committed blobs)"
$segments = @('src/render/bvh/cuda/scene.cu','src/render/kernel/cuda/render_math.cu','src/render/kernel/cuda/render_common.cu','src/render/kernel/cuda/render_shadow.cu','src/render/kernel/cuda/render_shade.cu','src/render/kernel/cuda/render_megakernel.cu')
foreach ($rev in @('8bc0f2b3','fa604008')) {
  $tmp = Join-Path $root 'target\sc-b.bin'
  $all = New-Object System.Collections.Generic.List[byte]
  foreach ($s in $segments) {
    cmd /c "git cat-file blob ${rev}:$s > target\sc-b.bin" 2>$null
    $all.AddRange([System.IO.File]::ReadAllBytes($tmp))
  }
  $arr = $all.ToArray()
  $hex = [System.BitConverter]::ToString(([System.Security.Cryptography.SHA256]::Create()).ComputeHash($arr)).Replace('-','').ToLower()
  L "  $rev MEGAKERNEL_SOURCE bytes=$($arr.Length) sha256=$hex"
  Remove-Item $tmp -ErrorAction SilentlyContinue
}
cmd /c "git cat-file blob fa604008`:src/render/kernel/cuda/aa_resolve.cu > target\sc-b-aa.bin" 2>$null
$aa = [System.IO.File]::ReadAllBytes((Join-Path $root 'target\sc-b-aa.bin'))
L "  fa604008 AA_RESOLVE_SOURCE bytes=$($aa.Length) sha256=$([System.BitConverter]::ToString(([System.Security.Cryptography.SHA256]::Create()).ComputeHash($aa)).Replace('-','').ToLower())"
Remove-Item (Join-Path $root 'target\sc-b-aa.bin') -ErrorAction SilentlyContinue
$refixId = 'docs\development-evidence\render-bvh\sdf-sphere-a-order-refix\materials\kernel-identity.txt'
L "refix_identity_file=$refixId"
Get-Content $refixId | Where-Object { $_ -match 'source_sha256|note:' } | ForEach-Object { L "  $_" }
$origId = 'docs\development-evidence\render-bvh\sphere-dot-a-order-fix\logs\kernel-identity.txt'
L "original_fix_identity_file=$origId"
Get-Content $origId | Where-Object { $_ -match 's7b3 megakernel' } | ForEach-Object { L "  $_" }
$afterLine = (Select-String -Path $refixId -Pattern 'source_sha256=10070226').Line
$refixPtx = [regex]::Match($afterLine, 'ptx_sha256=([0-9a-f]{64})').Groups[1].Value
$origLine = (Select-String -Path $origId -Pattern 'source_sha256=eb0e717f').Line
$origPtx = [regex]::Match($origLine, 'ptx_sha256=([0-9a-f]{64})').Groups[1].Value
L "ptx_identical_across_two_records=$($refixPtx -eq $origPtx) ptx=$refixPtx ptx_bytes_expected=268589"
L ""

L "## 6 upstream evidence manifest / patch"
$man = 'docs/development-evidence/render-bvh/sdf-sphere-a-order-refix/manifest.txt'
$tmp2 = Join-Path $root 'target\sc-bm.bin'
$bad = 0; $n = 0
Get-Content $man -Encoding UTF8 | Where-Object { $_ -and -not $_.StartsWith('#') } | ForEach-Object {
  $p = $_.Split(' '); $rel = $p[2]; $n++
  cmd /c "git cat-file blob fa604008`:$rel > target\sc-bm.bin" 2>$null
  $b = [System.IO.File]::ReadAllBytes($tmp2)
  $h = [System.BitConverter]::ToString(([System.Security.Cryptography.SHA256]::Create()).ComputeHash($b)).Replace('-','').ToLower()
  if ($h -ne $p[0] -or $b.Length -ne [int]$p[1]) { $bad++; L "  MISMATCH $rel" }
}
L "manifest_entries=$n mismatches=$bad"
Remove-Item $tmp2 -ErrorAction SilentlyContinue
$patch = (Get-Content 'docs\development-evidence\render-bvh\sdf-sphere-a-order-refix\materials\src-diff.patch' -Raw) -replace "`r`n","`n"
$real = (git diff 8bc0f2b3..fa604008 -- src) -join "`n"
L "src_diff_patch_matches_real=$($patch.TrimEnd() -eq $real.TrimEnd())"
L ""

L "## 7 gate provenance: committed runs/ vs evidence logs/"
foreach ($g in @('fmt','clippy','clippy-cuda','unit')) {
  $j = Get-Content ".normify\runs\2026-10-31-render-sdf-sphere-dot-a-order-refix\$g.json" -Raw -Encoding UTF8 | ConvertFrom-Json
  $e = Get-Content "docs\development-evidence\render-bvh\sdf-sphere-a-order-refix\logs\gate-$g.txt" -Raw -Encoding UTF8 | ConvertFrom-Json
  L "  $g runs_at=$($j.at) runs_wall=$($j.wall_ms) runs_logsha=$($j.log_sha256.Substring(0,12)) | evidence_at=$($e.at) evidence_wall=$($e.wall_ms) evidence_logsha=$($e.log_sha256.Substring(0,12)) same_run=$($j.at -eq $e.at -and $j.log_sha256 -eq $e.log_sha256)"
}
$u = Get-Content ".normify\runs\2026-10-31-render-sdf-sphere-dot-a-order-refix\unit.json" -Raw -Encoding UTF8 | ConvertFrom-Json
L "unit runs: passed=$($u.tests.passed) failed=$($u.tests.failed) ignored=$($u.tests.ignored) exit=$($u.exit)"
L "no_run_log=$((Get-Content 'docs\development-evidence\render-bvh\sdf-sphere-a-order-refix\logs\no-run.txt' -Raw).Trim())"
L ""

L "## 8 claims cross-check (upstream evidence text)"
$rpb = 'docs\development-evidence\render-bvh\sdf-sphere-a-order-refix\README.md'
L "readme_claims_290_lines=$((Select-String -Path $rpb -Pattern '290 行').Count -gt 0)"
L "receipt_change_render_math_lines_290=$((Select-String -Path 'docs\development-evidence\render-bvh\sdf-sphere-a-order-refix\receipt.json' -Pattern '\"lines\": 290').Count -gt 0)"
L "measured_total_lines=308 (309 before this slice); claim 290 does not match any line metric"
L "readme_wording_pending_attempt4=$((Select-String -Path $rpb -Pattern '待 attempt4 验证').Count -gt 0)"
L "receipt_claim_scope_pending_attempt4=$((Select-String -Path 'docs\development-evidence\render-bvh\sdf-sphere-a-order-refix\receipt.json' -Pattern 'TO BE VERIFIED IN ATTEMPT4').Count -gt 0)"
L "readme_no_sdf_pass_claim=$((Select-String -Path $rpb -Pattern '不声明 SDF 通过').Count -gt 0)"
L ""

L "## 9 stacked state (attempt4 prerequisites)"
$sd = (git diff --name-only fa604008..HEAD -- src | Out-String).Trim()
L "src_diff_fa604008_to_head=[$sd]"
L "host_guard_rs_hits_at_head=$(@(git grep -n 'reject_sdf_geometry' HEAD -- '*.rs').Count)"
L "megakernel_lines_at_head=$(@(git show HEAD:src/render/kernel/megakernel.rs).Count)"
$ao = git grep -n "float a = dot3_ba" HEAD -- src/render/kernel/cuda
L "device_a_dot3_ba_at_head=$(@($ao).Count)"
L "criterion_v2_exemption_objects (spec section 2):"
Select-String -Path docs\development-evidence\render-bvh\sdf-criterion-v2\spec.md -Pattern "受控豁免|清单外 1 ULP|B=128" | Select-Object -First 4 | ForEach-Object { L "  $($_.LineNumber): $($_.Line.Trim())" }
L ""

$dst = Join-Path $root 'docs\development-evidence\render-bvh\sdf-sphere-a-order-refix-check\materials\selfcheck.txt'
New-Item -ItemType Directory -Force (Split-Path $dst) | Out-Null
[System.IO.File]::WriteAllText($dst, ($out -join "`n") + "`n", (New-Object System.Text.UTF8Encoding($false)))
Write-Output "written: $dst"
