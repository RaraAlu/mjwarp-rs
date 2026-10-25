# G3 SDF sign 修正片 · 反向判别力检查（authoring-only，纯 CPU，不占 GPU）。
#
# 用途：把**改前提交** `23661557` 的 `primitives.rs` 原样嵌入临时 crate 的 `mod old`，
# 用本片新增的上游语义断言复算——若改前实现与上游 wp.sign 一致则应全部通过；
# 实际预期：`-0.0`/`NaN` 三条产品路径断言失败、负值断言通过（证明断言有判别力）。
#
# 用法（仓库根目录）：
#   powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\g10-p5-sdf-primitives-sign\pre_fix_reverse_check.ps1
# 输出：docs/development-evidence/g10-p5-sdf-primitives-sign/pre-fix-check-output.txt
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Set-Location $root
$commit = '236615576b4fd5fa792df82ecc364acb30a9e645'
$oldPath = 'src/physics/collision/sdf/primitives.rs'
$work = Join-Path $root 'target/g10-p5-p61-raw'
$evidence = Join-Path $root 'docs/development-evidence/g10-p5-sdf-primitives-sign'
New-Item -ItemType Directory -Force -Path $work, $evidence | Out-Null

$old = @(git show "${commit}:$oldPath" 2>&1)
if ($LASTEXITCODE -ne 0 -or $old.Count -eq 0) { throw "无法读取 ${commit}:${oldPath}" }
$oldText = ($old -join "`r`n")
if ($oldText -notmatch 'signum' -or $oldText -match 'upstream_sign') {
    throw '改前源与预期不符（应含 f32::signum() 且不含 upstream_sign）'
}
$oldSha = (Get-FileHash -Algorithm SHA256 -InputStream (
    New-Object System.IO.MemoryStream (, [System.Text.Encoding]::UTF8.GetBytes($oldText)))).Hash.ToLower()

$main = @'
// 反向判别力检查：把改前（commit __COMMIT__）primitives.rs 原样嵌入 `mod old`，
// 用本片新增的上游语义断言复算。期望：-0.0/NaN 三条产品路径断言失败（改前 f32::signum()），
// 负值断言通过；若全部通过则说明断言无判别力（脚本按失败退出）。
const MINVAL: f32 = 1.0e-12;

mod old {
__OLD_SOURCE__
}

fn main() {
    let a = [-0.5_f32, -0.5, -0.5];
    let size = [1.0_f32, 1.0, 1.0];
    let positive = old::radial_field(a, [1.0, 1.0, 1.0], size);
    let zero = old::radial_field(a, [-0.0, 0.0, -0.0], size);
    let nan = old::radial_field(a, [f32::NAN, 0.0, -0.0], size);
    let negative = old::radial_field(a, [-1.0, -2.0, -3.0], size);
    let grad = old::grad_cylinder([0.0, 0.0, -0.0], [2.0, 1.0, 1.0]);

    let mut failed = 0;
    let mut check = |name: &str, matched_upstream: bool| {
        println!("{} {} (改前实现符合上游={})", if matched_upstream { "PASS" } else { "FAIL" }, name, matched_upstream);
        if !matched_upstream {
            failed += 1;
        }
    };
    check("radial_field(-0.0/0.0) == +1 版本", zero == positive);
    check("radial_field(NaN) == +1 版本", nan == positive);
    check("grad_cylinder(-0.0) 轴向为 +1", grad == [0.0, 0.0, 1.0]);
    check("radial_field(负值) 仍为 -1（对照项）", negative == [-positive[0], -positive[1], -positive[2]]);
    println!("改前 f32::signum(-0.0) = {}", (-0.0_f32).signum());
    println!("改前 f32::signum(NaN).is_nan() = {}", f32::NAN.signum().is_nan());
    if failed == 3 {
        println!("REVERSE_OK failures={failed}/3（改前源恰在三处上游语义断言上失败：判别力成立）");
    } else {
        println!("REVERSE_UNEXPECTED failures={failed}（预期 3，检查是否拿到了正确的改前源）");
        std::process::exit(2);
    }
}
'@
$source = $main.Replace('__OLD_SOURCE__', $oldText).Replace('__COMMIT__', $commit)
$sourcePath = Join-Path $work 'prefix_check.rs'
[System.IO.File]::WriteAllText($sourcePath, $source, (New-Object System.Text.UTF8Encoding($false)))

$exe = Join-Path $work 'prefix_check.exe'
# 改前源里有未使用函数告警，rustc 的 warning 走 stderr；临时放宽 EAP 以免被 Stop 误判为失败。
$previousEap = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
rustc -O $sourcePath -o $exe 2>&1 | Out-Null
$ErrorActionPreference = $previousEap
if (-not (Test-Path $exe)) { throw 'rustc 编译前缀检查失败' }
# PS 5.1 解码原生子进程 stdout 用 OEM 码页，中文会乱码；改为重定向到文件后按 UTF-8 读原始字节。
$rawPath = Join-Path $work 'prefix_check.stdout.txt'
$process = Start-Process -FilePath $exe -NoNewWindow -Wait -PassThru `
    -RedirectStandardOutput $rawPath -RedirectStandardError ($rawPath + '.err')
$code = $process.ExitCode
$output = @([System.IO.File]::ReadAllLines($rawPath, [System.Text.Encoding]::UTF8))
$header = @(
    "pre_fix_reverse_check schema=1 task=2026-10-25-g10-p5-sdf-primitives-sign",
    "pre_fix_commit=$commit",
    "pre_fix_blob=$(& git rev-parse "${commit}:$oldPath")",
    "pre_fix_source=$oldPath",
    "pre_fix_source_sha256=$oldSha",
    "expected_failures=3 expected_control_pass=1",
    "rustc=$(& rustc --version)"
) + $output
$outPath = Join-Path $evidence 'pre-fix-check-output.txt'
[System.IO.File]::WriteAllLines($outPath, $header, (New-Object System.Text.UTF8Encoding($false)))
Get-Content $outPath
if ($code -ne 0) { throw "逆向检查未得到预期结果 exit=$code" }
