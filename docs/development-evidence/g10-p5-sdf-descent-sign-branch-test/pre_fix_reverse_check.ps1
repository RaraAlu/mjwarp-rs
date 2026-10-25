# G3 descent sign 分支测试 · 反向判别检查（authoring-only，纯 CPU）。
#
# 做法：把 `src/physics/collision/sdf/descent.rs` 里的
#   `let sign = primitives::upstream_sign(max_val);`
# 临时替换为修正前三分支（`>0→1, <0→-1, else→0`），跑本片新增的两个分支测试，
# 预期两者都 FAIL（旧实现把 0/-0.0 的增量抹成 0）；然后 **在 finally 中还原源码**，
# 校验 sha256 与还原前一致，并复跑确认 PASS。
#
# 用法（仓库根目录）：
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-sdf-descent-sign-branch-test\pre_fix_reverse_check.ps1
# 输出：docs/development-evidence/g10-p5-sdf-descent-sign-branch-test/pre-fix-reverse-output.txt
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Set-Location $root
$source = Join-Path $root 'src/physics/collision/sdf/descent.rs'
$evidence = Join-Path $root 'docs/development-evidence/g10-p5-sdf-descent-sign-branch-test'
New-Item -ItemType Directory -Force -Path $evidence | Out-Null

function Sha256([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }
function Run-BranchTests {
    $previousEap = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $output = @(& cargo test --locked --lib `
            physics::collision::sdf::descent::tests::compute_grad -- --test-threads=1 2>&1)
    $code = $LASTEXITCODE
    $ErrorActionPreference = $previousEap
    return @{ Code = $code; Output = $output }
}

if (-not (Test-Path $source)) { throw "源码不存在: $source" }
$original = [System.IO.File]::ReadAllText($source)
$originalSha = Sha256 $source
$needle = '    let sign = primitives::upstream_sign(max_val);'
if (-not $original.Contains($needle)) { throw '未找到 sign 行（源码版本不符）' }
$oldBranch = @'
    let sign = if max_val > 0.0 {
        1.0
    } else if max_val < 0.0 {
        -1.0
    } else {
        0.0
    };
'@
$patched = $original.Replace($needle, $oldBranch.TrimEnd())

$lines = @(
    'descent_sign_branch_reverse_check schema=1 task=2026-10-25-g10-p5-sdf-descent-sign-branch-test',
    "source=src/physics/collision/sdf/descent.rs",
    "source_sha256_before=$originalSha",
    'reverted_branch=old three-branch (>0->1, <0->-1, else->0)',
    'tests=compute_grad_zero_max_val_uses_upstream_sign,compute_grad_negative_zero_max_val_uses_upstream_sign',
    'expectation=both tests FAIL under old branch'
)
$restoredOk = $false
try {
    [System.IO.File]::WriteAllText($source, $patched, (New-Object System.Text.UTF8Encoding($false)))
    $reverted = Run-BranchTests
    $lines += '--- run under old branch ---'
    $lines += $reverted.Output
    $lines += ("old_branch_exit_code=" + $reverted.Code + " (expect non-zero)")
    $failCount = @($reverted.Output | Select-String -Pattern '^test .* \.\.\. FAILED$' -AllMatches).Count
    $lines += ("old_branch_failed_tests=" + $failCount + " (expect 2)")
    if ($reverted.Code -eq 0 -or $failCount -ne 2) {
        throw "旧分支下测试未按预期失败（exit=$($reverted.Code) failed=$failCount）"
    }
} finally {
    [System.IO.File]::WriteAllText($source, $original, (New-Object System.Text.UTF8Encoding($false)))
    $restoredSha = Sha256 $source
    $restoredOk = ($restoredSha -eq $originalSha)
    $lines += '--- restore ---'
    $lines += "source_sha256_after=$restoredSha restored_ok=$restoredOk"
}
if (-not $restoredOk) { throw '源码还原失败：sha256 不一致，立即停止并人工检查' }

$after = Run-BranchTests
$lines += '--- rerun after restore ---'
$lines += $after.Output
$lines += ("restored_exit_code=" + $after.Code + " (expect 0)")
$passCount = @($after.Output | Select-String -Pattern '^test .* \.\.\. ok$' -AllMatches).Count
$lines += ("restored_passed_tests=" + $passCount + " (expect 2)")
if ($after.Code -ne 0 -or $passCount -ne 2) {
    throw "还原后测试未通过（exit=$($after.Code) passed=$passCount）"
}
$lines += 'REVERSE_OK old_branch_fails_2/2 restored_passes_2/2'

$outPath = Join-Path $evidence 'pre-fix-reverse-output.txt'
[System.IO.File]::WriteAllLines($outPath, $lines, (New-Object System.Text.UTF8Encoding($false)))
Get-Content $outPath | Select-Object -Last 20
