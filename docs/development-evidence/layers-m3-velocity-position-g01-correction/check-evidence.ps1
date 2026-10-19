$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-velocity-position-g01-correction'
$root = 'docs/development-evidence/layers-m3-velocity-position-g01-correction'
$runs = "$root/runs"
$runtime = "target/normify-evidence/$task"
$base = 'f40a78af3947142bb31d587d097e42312594edf3'
$sourcePath = 'src/physics/resident.rs'
$plain = '788c870d787e06d58bce113eef24846c0b14aa43f524189a3508f58fc940f280'
$mocap = '93318e85811ad7db5a50c3217eb08201ae0d51b649c6d69feec1f5b6355239a4'
$g01 = '58cb789359125491c79f2b208717b69fbcf18ef1f9172fbfad27a7fe9d02cdb7'

function Get-Sha([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

# 归档运行：登记先于执行，日志预建，退出码符合。
function Assert-Run([string]$Id, [int]$ExitCode = 0) {
    $registrationPath = "$runs/registrations/$Id.json"
    $registration = Get-Content -LiteralPath $registrationPath -Raw | ConvertFrom-Json
    $output = ([string]$registration.output_root).Replace("$runtime/", "$runs/")
    $result = Get-Content -LiteralPath "$output/result.json" -Raw | ConvertFrom-Json
    $started = Get-Content -LiteralPath "$output/started.json" -Raw | ConvertFrom-Json
    $hash = Get-Sha $registrationPath
    if ([DateTime]$registration.registered_at_utc -ge [DateTime]$result.started_at_utc -or
        $result.registration_sha256 -cne $hash -or
        $started.registration_sha256 -cne $hash -or
        $started.stdout_precreated -ne $true -or
        $started.stderr_precreated -ne $true -or
        $result.exit_code -ne $ExitCode) {
        throw "运行证据错误：$Id"
    }
    [ordered]@{
        out = Get-Content -LiteralPath "$output/stdout.log" -Raw
        err = Get-Content -LiteralPath "$output/stderr.log" -Raw
        dir = $output
        started = [DateTime]$result.started_at_utc
    }
}

foreach ($id in 'device-preflight', 'repro-format', 'format-apply', 'format-check', 'clippy-default', 'clippy-cuda', 'inventory') {
    Assert-Run $id | Out-Null
}
$repro = Assert-Run 'repro-gpu' 101
$fix = Assert-Run 'gpu-fix'
$hostRun = Assert-Run 'host-resident'
$plainRun = Assert-Run 'ptx-plain'
$lib = Assert-Run 'gpu-lib-regression' 101
$integration = Assert-Run 'gpu-integration'
$g01Run = Assert-Run 'g01'
$baseRuns = @('base-flex-face', 'base-flex-face-retry-01', 'base-flex-face-retry-02', 'base-flex-face-retry-03', 'base-flex-face-retry-04') |
    ForEach-Object { Assert-Run $_ 101 }

# 复现早于修复，且只在修复前失败。
$formatApply = Get-Content -LiteralPath "$runs/check-02-format-apply/result.json" -Raw | ConvertFrom-Json
if ($repro.started -ge [DateTime]$formatApply.started_at_utc -or
    $repro.out -notmatch 'mismatched=280' -or $repro.out -notmatch '1 failed' -or
    $fix.out -notmatch 'slots=504 mismatched=0' -or $fix.out -notmatch 'imported-world-pose=matched' -or
    $fix.out -notmatch '1 passed; 0 failed') {
    throw '复现或修复结果不符'
}
if ($hostRun.out -notmatch '14 passed; 0 failed' -or $plainRun.out -notmatch '1 passed; 0 failed') { throw '宿主或plain捕获结果不符' }
$integrationPassed = [regex]::Matches($integration.out, '(?m)^test result: ok\. (\d+) passed; 0 failed') |
    ForEach-Object { [int]$_.Groups[1].Value } | Measure-Object -Sum
if ($integrationPassed.Count -ne 3 -or $integrationPassed.Sum -ne 16) { throw '集成回归计数不符' }
if ($g01Run.out -notmatch '7 passed; 0 failed') { throw 'G01回归计数不符' }

# 单元回归唯一失败项在基线源码同样失败。
$name = 'physics::resident::tests::flex_face_readback_rejects_guards_nonfinite_and_arithmetic_overflow'
$failures = @([regex]::Matches($lib.out, "(?m)^test (\S+) \.\.\. FAILED$") | ForEach-Object { $_.Groups[1].Value })
if ($lib.out -notmatch '27 passed; 1 failed' -or $failures.Count -ne 1 -or $failures[0] -cne $name) { throw '单元回归失败项不符' }
if ($baseRuns[0].err -notmatch 'manifest path' -or $baseRuns[1].err -notmatch 'g03_first_difference' -or
    $baseRuns[2].err -notmatch 'previous-kernels\.json' -or $baseRuns[3].out -notmatch 'NvrtcUnavailable' -or
    $baseRuns[4].out -notmatch [regex]::Escape("test $name ... FAILED") -or
    $baseRuns[4].out -notmatch 'resident_flex_face_output') {
    throw '基线复验链不符'
}

# PTX：修复前后mocap与G01变体相同，plain等于上一任务基线。
function Get-Ptx([string]$Dir) {
    @(Get-ChildItem -LiteralPath "$Dir/gpu-capture" -Recurse -File -Filter 'rigid_kinematics-*.ptx' | ForEach-Object {
        $hash = Get-Sha $_.FullName
        if ($_.Name -cne "rigid_kinematics-$hash.ptx") { throw "PTX文件名与内容不符：$($_.Name)" }
        $hash
    } | Sort-Object -Unique)
}
$expected = @($g01, $mocap) | Sort-Object
foreach ($dir in $repro.dir, $fix.dir) {
    if ((Compare-Object (Get-Ptx $dir) $expected)) { throw "变体PTX不符：$dir" }
}
if ((Compare-Object (Get-Ptx $plainRun.dir) @($plain))) { throw 'plain PTX不符' }
$baseline = 'docs/development-evidence/layers-m3-rigid-mass-matrix/runs/run-01-gpu/gpu-capture/product-main'
if (-not (Test-Path -LiteralPath "$baseline/rigid_kinematics-$plain.ptx")) { throw 'plain基线缺失' }
foreach ($pair in @(@($g01, 1), @($mocap, 0))) {
    $cu = Get-ChildItem -LiteralPath "$($fix.dir)/gpu-capture" -Recurse -File -Filter "rigid_kinematics-$($pair[0]).cu" | Select-Object -First 1
    $hasG01 = ([IO.File]::ReadAllText($cu.FullName)).Contains('#define MJWARP_G01_EQUIVALENT')
    if ($hasG01 -ne [bool]$pair[1]) { throw "变体宏不符：$($pair[0])" }
}

# 只改resident.rs；复现补丁只改测试模块。
$changed = @(& git diff --name-only $base -- src tests)
if ($changed.Count -ne 1 -or $changed[0] -cne $sourcePath) { throw "源码改动范围不符：$($changed -join ',')" }
$patch = [IO.File]::ReadAllText((Resolve-Path -LiteralPath "$root/repro-test.patch"))
$hunks = @([regex]::Matches($patch, '(?m)^@@ -(\d+)') | ForEach-Object { [int]$_.Groups[1].Value })
if ($hunks.Count -eq 0 -or ($hunks | Where-Object { $_ -lt 6600 })) { throw '复现补丁越出测试模块' }
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
if ($source.Contains('SynchronousKernel::compile(&self.session, &source, "rigid_kinematics")') -or
    -not $source.Contains('rigid.copy_range_from(0, &data.rigid, 0, data.rigid.len())?;') -or
    -not $source.Contains('self.rigid_kernel.launch(')) {
    throw '修复源码锚点不符'
}

$inventory = Get-Content -LiteralPath "$root/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
$rigid = @($inventory.actual_cuda_entry_kinds | Where-Object entry -CEQ 'rigid_kinematics')
if ($inventory.task_id -cne $task -or $rigid.Count -ne 1 -or @($rigid[0].compile_sites).Count -ne 2 -or
    $inventory.counts.compile_call_sites -ne 89 -or $inventory.counts.unmigrated_actual_entry_kinds -ne 56 -or
    $inventory.counts.unmigrated_low_level_call_sites -ne 71 -or
    $inventory.source_file_sha256 -cne (Get-Sha $inventory.source_file)) {
    throw '清单不符'
}

[ordered]@{
    schema = 1
    task_id = $task
    runs = 19
    source_sha256 = Get-Sha $sourcePath
    result = 'pass'
} | ConvertTo-Json
