# G10@P5 修正片 · GPU 窗口探针构建（authoring-only，不占 GPU）。
# 用法（仓库根目录）：pwsh -NoProfile -File docs/development-evidence/g10-p5-ccd-overflow-bit-fix/build_probes.ps1
#
# 构建 4 个探针（同一批 nvcc 选项：`-std=c++17 -O3 -arch=sm_89 --fmad=true`）：
# 1. overflow_probe_after.exe  ：本片探针 + 当前工作树 device 源（修正后）；
# 2. overflow_probe_before.exe ：本片探针 + HEAD 的 device 源（修正前，从 `git show` 抽出，用于反向判别）；
# 3. g5_probe.exe              ：G5 冻结 `probe.cu` + 当前 device 源（回归：接触/容量判定）；
# 4. g5_witness_probe.exe      ：G5 冻结 `witness_probe.cu` + 当前 device 源（回归：witness 三例）。
#
# 产物与源 sha256 报告：`docs/development-evidence/g10-p5-ccd-overflow-bit-fix/build-report.txt`。
# 不写 G5 冻结证据目录；device-before 源落在 `target/g10-p5-p58/device-before/`。
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Set-Location $root

$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
$nvcc = Join-Path $root 'target/toolchains/cuda-12.8.1/bin/nvcc.exe'
if (-not (Test-Path $nvcc)) { throw "nvcc 不存在: $nvcc" }
$cudartLib = Get-ChildItem (Join-Path $root 'target/toolchains/cuda-12.8.1') -Recurse -Filter 'cudart.lib' |
    Select-Object -First 1
if (-not $cudartLib) { throw 'cudart.lib 不存在' }
$cudartDir = $cudartLib.DirectoryName
$cudartDll = Get-ChildItem $cudartDir -Filter 'cudart64_*.dll' | Select-Object -First 1
if (-not $cudartDll) {
    $cudartDll = Get-ChildItem (Join-Path $root 'target/toolchains/cuda-12.8.1/bin') -Filter 'cudart64_*.dll' |
        Select-Object -First 1
}

$evidence = 'docs/development-evidence/g10-p5-ccd-overflow-bit-fix'
$outDir = 'target/g10-p5-p58'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Copy-Item $cudartDll.FullName (Join-Path $outDir $cudartDll.Name) -Force

function Sha256([string]$path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }

# 1) device-before：从 HEAD 抽出 14 个 device .cu（修正前源），供 before 探针 include。
$beforeDir = Join-Path $outDir 'device-before'
New-Item -ItemType Directory -Force -Path $beforeDir | Out-Null
$deviceSources = git ls-tree -r --name-only HEAD -- src/physics/collision/cuda | Where-Object { $_ -like '*.cu' }
if (-not $deviceSources) { throw 'HEAD 中找不到 device .cu' }
foreach ($source in $deviceSources) {
    $name = Split-Path $source -Leaf
    cmd /c ('git show "HEAD:{0}" > "{1}"' -f $source, (Join-Path $beforeDir $name))
    if ($LASTEXITCODE -ne 0) { throw "git show 失败: $source" }
}
$changed = @('ccd.cu', 'contact.cu', 'hfield.cu') | Where-Object {
    (Sha256 (Join-Path $beforeDir $_)) -ne (Sha256 (Join-Path 'src/physics/collision/cuda' $_))
}
Write-Output ('device-before sources=' + $deviceSources.Count + ' changed_vs_worktree=' + ($changed -join ','))

function Build-Probe([string]$binary, [string]$probeSource, [string]$deviceInclude) {
    $command = '"' + $nvcc + '" -std=c++17 -O3 -arch=sm_89 --fmad=true -Xcompiler /utf-8' +
        ' -I ' + $deviceInclude + ' -L "' + $cudartDir + '" -o "' + $binary + '" ' + $probeSource
    # 构建日志落 target，不混入函数返回值（否则命令记录会被 nvcc 输出污染）。
    $buildLog = Join-Path $outDir ((Split-Path $binary -Leaf) + '.build.log')
    cmd /c ('"' + $vcvars + '" >nul 2>&1 && ' + $command + ' > "' + $buildLog + '" 2>&1')
    if ($LASTEXITCODE -ne 0) { throw "nvcc 构建失败: $binary exit=$LASTEXITCODE（日志 $buildLog）" }
    return $command
}

$deviceDir = 'src/physics/collision/cuda'
$g5Evidence = 'docs/development-evidence/g10-p5-gpu-kernels/gpu-window'
$afterExe = Join-Path $outDir 'overflow_probe_after.exe'
$beforeExe = Join-Path $outDir 'overflow_probe_before.exe'
$g5Exe = Join-Path $outDir 'g5_probe.exe'
$witnessExe = Join-Path $outDir 'g5_witness_probe.exe'

$commandAfter = Build-Probe $afterExe "$evidence/overflow_probe.cu" $deviceDir
$commandBefore = Build-Probe $beforeExe "$evidence/overflow_probe.cu" $beforeDir
$commandG5 = Build-Probe $g5Exe "$g5Evidence/probe.cu" $deviceDir
$commandWitness = Build-Probe $witnessExe "$g5Evidence/witness_probe.cu" $deviceDir
Write-Output 'BUILD_OK 4 probes'

$headRevision = (git rev-parse HEAD)
$lines = @(
    'probe_build schema=1',
    ('head_revision=' + $headRevision),
    ('nvcc=' + ((& $nvcc --version) | Select-String -Pattern 'release' | ForEach-Object { $_.Line.Trim() })),
    ('fmad=true fast_math=false arch=sm_89 opt=-O3 std=c++17'),
    ('device_include_after=' + $deviceDir),
    ('device_include_before=' + $beforeDir),
    ('command_overflow_after=' + $commandAfter),
    ('command_overflow_before=' + $commandBefore),
    ('command_g5_probe=' + $commandG5),
    ('command_g5_witness_probe=' + $commandWitness),
    ('overflow_probe_after_sha256=' + (Sha256 $afterExe)),
    ('overflow_probe_before_sha256=' + (Sha256 $beforeExe)),
    ('g5_probe_sha256=' + (Sha256 $g5Exe)),
    ('g5_witness_probe_sha256=' + (Sha256 $witnessExe)),
    ('probe_source_sha256 ' + (Sha256 "$evidence/overflow_probe.cu") + ' overflow_probe.cu'),
    ('probe_source_sha256 ' + (Sha256 "$g5Evidence/probe.cu") + ' g5_probe.cu(frozen)'),
    ('probe_source_sha256 ' + (Sha256 "$g5Evidence/witness_probe.cu") + ' g5_witness_probe.cu(frozen)')
)
foreach ($source in ($deviceSources | Sort-Object)) {
    $name = Split-Path $source -Leaf
    $lines += ('device_before_sha256 ' + (Sha256 (Join-Path $beforeDir $name)) + ' ' + $name)
    $lines += ('device_after_sha256 ' + (Sha256 (Join-Path $deviceDir $name)) + ' ' + $name)
}
[System.IO.File]::WriteAllLines(
    (Join-Path $root "$evidence/build-report.txt"), $lines,
    (New-Object System.Text.UTF8Encoding($false)))
Write-Output ("BUILD_REPORT_WRITTEN changed_device_sources=" + ($changed -join ','))
