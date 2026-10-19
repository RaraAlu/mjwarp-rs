$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rigid-mass-matrix'
$evidence = 'docs/development-evidence/layers-m3-rigid-mass-matrix'
$runs = "$evidence/runs"
$source = 'src/physics/mass_matrix.rs'
$receiptPath = "docs/development-evidence/changes/$task.json"
if (Test-Path -LiteralPath $receiptPath) { throw "回执已存在：$receiptPath" }

& pwsh -NoProfile -File "$evidence/check-evidence.ps1" | Out-Null
if ($LASTEXITCODE -ne 0) { throw '证据核验失败' }
& git add -- $source $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0 -or $staged.Count -eq 0) { throw '暂存区读取失败或为空' }
foreach ($path in $staged) {
    if ($path -ne $source -and -not $path.StartsWith("$evidence/")) { throw "暂存范围外文件：$path" }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{ path = $_; blob = $blob }
})
$inventory = Get-Content -LiteralPath "$evidence/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $task
    milestones = [ordered]@{ implementation = 'complete'; validation = 'passed'; delivery_record = 'pending_commit' }
    files = $files
    validation = @(
        [ordered]@{ command = 'cargo fmt --check'; result = 'pass'; evidence = "$runs/check-02-format/result.json" },
        [ordered]@{ command = 'cargo test --locked --lib physics::mass_matrix::tests::rigid_mass_matrix_plan -- --test-threads=1'; result = 'pass: 2 passed'; evidence = "$runs/check-03-host-wrapper/result.json" },
        [ordered]@{ command = 'cargo test --locked --lib physics::mass_matrix:: -- --test-threads=1'; result = 'pass: 12 passed'; evidence = "$runs/check-04-host-module/result.json" },
        [ordered]@{ command = 'cargo clippy --locked --all-targets -- -D warnings'; result = 'pass'; evidence = "$runs/check-05-clippy-default/result.json" },
        [ordered]@{ command = 'cargo clippy --locked --all-targets --features cuda-probe -- -D warnings'; result = 'pass'; evidence = "$runs/check-06-clippy-cuda/result.json" },
        [ordered]@{ command = 'cargo test --locked --features cuda-probe --lib physics::mass_matrix::tests::runs_rigid_mass_matrix_wrapper_paths_and_recovers -- --ignored --exact --test-threads=1'; result = 'pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed; raw and typed bitwise equal for 257 worlds, zero DOF and a 3-world product model; four PTX captures equal f695b0e8'; evidence = "$runs/run-01-gpu/result.json" },
        [ordered]@{ command = 'cargo test --locked --features cuda-probe --lib physics::mass_ -- --ignored --test-threads=1'; result = 'fail: 41 passed, 2 failed because report path variables were unset after all assertions'; evidence = "$runs/run-02-gpu-regression/result.json" },
        [ordered]@{ command = 'cargo test --locked --features cuda-probe --lib -- --ignored --exact --test-threads=1 <two failed tests> with MJWARP_MUL_M_REPORT and MJWARP_RESIDENT_MIXED_REPORT'; result = 'pass: 2 passed'; evidence = "$runs/run-03-gpu-report/result.json" },
        [ordered]@{ command = 'pwsh -NoProfile -File docs/development-evidence/layers-m3-rigid-mass-matrix/build-inventory.ps1'; result = 'pass: prior counts reproduced; 56 entry kinds and 71 low-level sites remain'; evidence = "$runs/check-08-inventory/result.json" },
        [ordered]@{ command = 'pwsh -NoProfile -File docs/development-evidence/layers-m3-rigid-mass-matrix/check-evidence.ps1'; result = 'pass: 11 runs, 4 PTX captures, source identity and inventory'; evidence = "$runs/check-09-evidence/result.json" }
    )
    preserved_findings = @(
        [ordered]@{ finding = 'first GPU regression failed two tests on unset report path variables'; historical_run = 'gpu-mass-regression'; authoritative_run = 'gpu-report-regression' },
        [ordered]@{ finding = 'inventory next candidate rigid_kinematics has three preprocessor-variant compile sites'; action = 'not started; requires separate scoping' }
    )
    remaining = @(
        "$($inventory.counts.unmigrated_actual_entry_kinds) actual CUDA entry kinds remain unmigrated",
        "$($inventory.counts.unmigrated_low_level_call_sites) product low-level call sites remain unmigrated",
        'historical flg_acc=false still has 1042 failed blocks',
        'NP01 through NP04 remain unaccepted',
        'frozen MJWarp static comparison was not run for this entry',
        'performance remains unaccepted',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'parent M3 remains incomplete'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
$bytes = [Text.UTF8Encoding]::new($false).GetBytes($json)
$stream = [IO.File]::Open((Join-Path (Get-Location) $receiptPath), [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{ schema = 1; task = $task; covered_files = $files.Count; receipt = $receiptPath; result = 'staged' } | ConvertTo-Json
