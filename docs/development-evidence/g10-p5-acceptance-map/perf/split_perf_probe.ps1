# G6-2 性能探针 · 从冻结 G5 探针机械拆分（authoring-only，不占 GPU、不改冻结证据）。
#
# 用法（仓库根目录）：
#   pwsh -NoProfile -File docs/development-evidence/g10-p5-acceptance-map/perf/split_perf_probe.ps1
#
# 输入：docs/development-evidence/g10-p5-gpu-kernels/gpu-window/probe.cu（冻结，只读）
# 输出：perf/perf_probe_common.cu（源 1-426 行：设备 include/结构体/kernel）
#       perf/perf_probe.cu（源 427-743 行：main + 插入 `--perf` 计时分支）
# 说明：除插入 `--perf` 分支外，不改内核、加载逻辑与正常 out1/out2 路径；源 sha256 记入两文件头。
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Set-Location $root

$source = Join-Path $root 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window/probe.cu'
$outDir = Join-Path $root 'docs/development-evidence/g10-p5-acceptance-map/perf'
if (-not (Test-Path $source)) { throw "冻结探针源不存在: $source" }
if (-not (Test-Path $outDir)) { throw "输出目录不存在: $outDir" }

$lines = [System.IO.File]::ReadAllLines($source, [System.Text.Encoding]::UTF8)
$sha = (Get-FileHash -Algorithm SHA256 $source).Hash.ToLower()
# 边界：`int main` 之前是 common（含匿名命名空间收尾 `}  // namespace` 与一个空行）。
$split = -1
for ($index = 0; $index -lt $lines.Count; $index++) {
    if ($lines[$index] -match '^int main\(') { $split = $index; break }
}
if ($split -lt 1) { throw '找不到 `int main` 边界' }
$before = $split - 1
while ($before -ge 0 -and $lines[$before].Trim() -eq '') { $before-- }
if ($before -lt 0 -or $lines[$before] -notmatch '^\}\s+// namespace') {
    throw "拆分边界不符：$($lines[$before])"
}
$commonEnd = $split - 1

$utf8 = New-Object System.Text.UTF8Encoding($false)
$common = @(
    '// G6-2 性能探针 · 冻结 G5 探针源第 1-426 行（设备 include/结构体/kernel；authoring-only，不进产品构建）。',
    "// 来源：docs/development-evidence/g10-p5-gpu-kernels/gpu-window/probe.cu（sha256=$sha）。",
    '// 由 split_perf_probe.ps1 机械拆分生成；除 perf_probe.cu 的 --perf 分支外未改内核与加载逻辑。',
    ''
) + $lines[0..$commonEnd]
[System.IO.File]::WriteAllText((Join-Path $outDir 'perf_probe_common.cu'), (($common -join "`r`n") + "`r`n"), $utf8)

$marker = '  if (kind == "sdf") kernel = kernel_sdf;'
$branch = @(
    $marker,
    '',
    '  // G6-2 perf 模式：`perf_probe <case_dir> --perf [warmup] [reps]`，只计量、不写 out1/out2。',
    '  if (argc > 2 && std::string(argv[2]) == "--perf") {',
    '    const int warmup = argc > 3 ? std::atoi(argv[3]) : 5;',
    '    const int reps = argc > 4 ? std::atoi(argv[4]) : 50;',
    '    return run_perf_mode(directory, host, args, kernel, meta, properties, warmup, reps);',
    '  }'
)
$patched = New-Object System.Collections.Generic.List[string]
$inserted = $false
foreach ($line in $lines[$split..($lines.Count - 1)]) {
    if (-not $inserted -and $line -eq $marker) {
        $patched.AddRange([string[]]$branch)
        $inserted = $true
    } else {
        $patched.Add($line)
    }
}
if (-not $inserted) { throw "未找到 --perf 插入锚点: $marker" }

$main = @(
    '// G6-2 性能探针主入口 · 冻结 G5 探针源第 427-743 行（main）+ 插入的 `--perf` 计时分支。',
    "// 来源：docs/development-evidence/g10-p5-gpu-kernels/gpu-window/probe.cu（sha256=$sha）。",
    '// 正常模式（`perf_probe <case_dir> [repeats]`）与冻结探针完全一致；`--perf` 只做计量。',
    '',
    '#include "perf_probe_common.cu"',
    '#include "perf_probe_timing.cu"',
    ''
) + $patched
[System.IO.File]::WriteAllText((Join-Path $outDir 'perf_probe.cu'), (($main -join "`r`n") + "`r`n"), $utf8)

Write-Output ("SPLIT_OK source_sha256={0} common_lines={1} main_lines={2}" -f $sha, $common.Count, $main.Count)
