# Read-only self-checks for QA review card 2026-10-31-render-sdf-host-unlock-redo-check
# Usage: pwsh -File selfcheck-a.ps1   (run from repo root D:\Rust\mjwarp-rs)
$ErrorActionPreference = 'Stop'
$root = 'D:\Rust\mjwarp-rs'
Set-Location $root
$out = New-Object System.Collections.Generic.List[string]
function L($s) { $out.Add($s); Write-Output $s }

L "# selfcheck A: U1b redo (pin 013d8d4a)"
L "generated_at=$(Get-Date -Format o)"
L "head=$(git rev-parse HEAD)"
L ""

L "## 1 scope: slice commit 013d8d4a"
L (git show --stat --format="" 013d8d4a | Out-String).Trim()
L "src_files_only=$(git diff --name-only 1735d062..013d8d4a -- src)"
L "cuda_diff_empty=$([string]::IsNullOrWhiteSpace((git diff --name-only 1735d062..013d8d4a -- src/render/kernel/cuda src/render/bvh)))"
$tfc = (git diff --name-only 1735d062..013d8d4a -- tests fixtures scripts Cargo.toml Cargo.lock docs/development-evidence/render-bvh/sdf-criterion-v2 | Out-String).Trim()
L "tests_fixtures_criterion_diff=[$tfc]"
L ""

L "## 2 line counts / numstat"
$before = @(git show 1735d062:src/render/kernel/megakernel.rs).Count
$after = @(git show 013d8d4a:src/render/kernel/megakernel.rs).Count
L "megakernel_lines_before=$before after=$after"
L (git diff --numstat 1735d062..013d8d4a -- src/render/kernel/megakernel.rs)
$ora = @(git show af96c048:src/render/kernel/megakernel.rs).Count
L "af96c048_megakernel_lines=$ora (original U1b; diff adds 1 unlock comment line)"
L ""

L "## 3 guard removal"
$rs = git grep -n "reject_sdf_geometry" 013d8d4a -- "*.rs"
L "rs_hits=$(@($rs).Count)"
$cu = git grep -n "reject_sdf_geometry" 013d8d4a -- "*.cu"
L "cu_comment_hits=$(@($cu).Count)"
$cu | ForEach-Object { L "  $_" }
$errs = git grep -n "S7b3 不含 SDF 求交" 013d8d4a -- src
L "old_error_string_hits=$(@($errs).Count)"
$tests = git grep -n "sdf_geometry_guard" 013d8d4a -- src tests
L "guard_test_module_hits=$(@($tests).Count)"
$unsup = git grep -n "LauncherError::Unsupported(" 013d8d4a -- src/render/kernel/megakernel.rs
L "other_unsupported_rejection_calls=$(@($unsup).Count)"
$unsup | ForEach-Object { L "  $_" }
L ""

L "## 4 kernel identity (independent recompute from committed blobs)"
$segments = @('src/render/bvh/cuda/scene.cu','src/render/kernel/cuda/render_math.cu','src/render/kernel/cuda/render_common.cu','src/render/kernel/cuda/render_shadow.cu','src/render/kernel/cuda/render_shade.cu','src/render/kernel/cuda/render_megakernel.cu')
$tmp = Join-Path $root 'target\sc-a.bin'
$all = New-Object System.Collections.Generic.List[byte]
foreach ($s in $segments) {
  cmd /c "git cat-file blob 013d8d4a`:$s > target\sc-a.bin" 2>$null
  $b = [System.IO.File]::ReadAllBytes($tmp)
  L "  segment $s bytes=$($b.Length)"
  $all.AddRange($b)
}
$arr = $all.ToArray()
$sha = [System.Security.Cryptography.SHA256]::Create()
$hex = [System.BitConverter]::ToString($sha.ComputeHash($arr)).Replace('-','').ToLower()
L "MEGAKERNEL_SOURCE bytes=$($arr.Length) sha256=$hex"
cmd /c "git cat-file blob 013d8d4a`:src/render/kernel/cuda/aa_resolve.cu > target\sc-aa.bin" 2>$null
$aa = [System.IO.File]::ReadAllBytes((Join-Path $root 'target\sc-aa.bin'))
$sha2 = [System.Security.Cryptography.SHA256]::Create()
L "AA_RESOLVE_SOURCE bytes=$($aa.Length) sha256=$([System.BitConverter]::ToString($sha2.ComputeHash($aa)).Replace('-','').ToLower())"
L "upstream_identity_file=docs/development-evidence/render-bvh/sdf-host-unlock-redo/materials/kernel-identity-unchanged.txt"
Get-Content 'docs\development-evidence\render-bvh\sdf-host-unlock-redo\materials\kernel-identity-unchanged.txt' | ForEach-Object { L "  $_" }
Remove-Item $tmp, (Join-Path $root 'target\sc-aa.bin') -ErrorAction SilentlyContinue
L ""

L "## 5 upstream evidence manifest vs committed blobs"
$man = 'docs/development-evidence/render-bvh/sdf-host-unlock-redo/manifest.txt'
$tmp2 = Join-Path $root 'target\sc-m.bin'
$bad = 0; $n = 0
Get-Content $man -Encoding UTF8 | Where-Object { $_ -and -not $_.StartsWith('#') } | ForEach-Object {
  $p = $_.Split(' '); $rel = $p[2]; $n++
  cmd /c "git cat-file blob 013d8d4a`:$rel > target\sc-m.bin" 2>$null
  $b = [System.IO.File]::ReadAllBytes($tmp2)
  $h = [System.BitConverter]::ToString(([System.Security.Cryptography.SHA256]::Create()).ComputeHash($b)).Replace('-','').ToLower()
  if ($h -ne $p[0] -or $b.Length -ne [int]$p[1]) { $bad++; L "  MISMATCH $rel" }
}
L "manifest_entries=$n mismatches=$bad"
Remove-Item $tmp2 -ErrorAction SilentlyContinue
$patch = (Get-Content 'docs\development-evidence\render-bvh\sdf-host-unlock-redo\materials\src-diff.patch' -Raw) -replace "`r`n","`n"
$real = (git diff 1735d062..013d8d4a -- src/render/kernel/megakernel.rs) -join "`n"
L "src_diff_patch_matches_real=$($patch.TrimEnd() -eq $real.TrimEnd())"
L ""

L "## 6 gate provenance: committed runs/ vs evidence logs/"
foreach ($g in @('fmt','clippy','clippy-cuda','unit')) {
  $j = Get-Content ".normify\runs\2026-10-31-render-sdf-host-unlock-redo\$g.json" -Raw -Encoding UTF8 | ConvertFrom-Json
  $e = Get-Content "docs\development-evidence\render-bvh\sdf-host-unlock-redo\logs\gate-$g.txt" -Raw -Encoding UTF8 | ConvertFrom-Json
  L "  $g runs_at=$($j.at) runs_wall=$($j.wall_ms) runs_logsha=$($j.log_sha256.Substring(0,12)) | evidence_at=$($e.at) evidence_wall=$($e.wall_ms) evidence_logsha=$($e.log_sha256.Substring(0,12)) same_run=$($j.at -eq $e.at -and $j.log_sha256 -eq $e.log_sha256)"
}
$u = Get-Content ".normify\runs\2026-10-31-render-sdf-host-unlock-redo\unit.json" -Raw -Encoding UTF8 | ConvertFrom-Json
L "unit runs: passed=$($u.tests.passed) failed=$($u.tests.failed) ignored=$($u.tests.ignored) exit=$($u.exit)"
L "no_run_log=$((Get-Content 'docs\development-evidence\render-bvh\sdf-host-unlock-redo\logs\no-run.txt' -Raw).Trim())"
L ""

L "## 7 stacked state (attempt4 prerequisites)"
$sd = (git diff --name-only fa604008..HEAD -- src | Out-String).Trim()
L "src_diff_fa604008_to_head=[$sd]"
L "megakernel_lines_at_head=$(@(git show HEAD:src/render/kernel/megakernel.rs).Count)"
$guardHead = git grep -n "reject_sdf_geometry" HEAD -- "*.rs"
L "guard_rs_hits_at_head=$(@($guardHead).Count)"
$ao = git grep -n "float a = dot3_ba" HEAD -- src/render/kernel/cuda
L "device_a_dot3_ba_at_head=$(@($ao).Count)"
L "  $ao"
L ""

L "## 8 claims cross-check (upstream evidence text)"
$rp = 'docs\development-evidence\render-bvh\sdf-host-unlock-redo\README.md'
L "readme_claims_440_to_405=$((Select-String -Path $rp -Pattern '440 → 405').Count -gt 0)"
L "readme_claims_zero_cu=$((Select-String -Path $rp -Pattern '零 `.cu` 改动').Count -gt 0)"
L "readme_mentions_residual_cu_comments=$((Select-String -Path $rp -Pattern 'render_common|注释仍有|注释 2 行').Count -gt 0)"
L "same_shape_note=af96c048 added 1 unlock comment line (406 lines); this redo omits it (405 lines), guard removal identical"
$cardStale = Select-String -Path .normify\changes\2026-10-31-render-sdf-validate.json -Pattern "U1b 重做暂缓"
L "u4_card_note_still_says_u1b_deferred=$(@($cardStale).Count -gt 0)"
$attempt4 = Select-String -Path .normify\changes\2026-10-31-render-sdf-validate-window-material-v2.json -Pattern "host 解锁|10070226"
L "attempt4_material_v2_prepare_checklist_refs_host_unlock_and_identity=$(@($attempt4).Count -gt 0)"
L ""

L "## 9 validation re-run (read-only)"
$v = normify validate --fingerprints 2>&1
$verr = ($v | Where-Object { $_ -match '错误' })
L "validate=$($verr -join ' ')"
L ""

$dst = Join-Path $root 'docs\development-evidence\render-bvh\sdf-host-unlock-redo-check\materials\selfcheck.txt'
New-Item -ItemType Directory -Force (Split-Path $dst) | Out-Null
[System.IO.File]::WriteAllText($dst, ($out -join "`n") + "`n", (New-Object System.Text.UTF8Encoding($false)))
Write-Output "written: $dst"
