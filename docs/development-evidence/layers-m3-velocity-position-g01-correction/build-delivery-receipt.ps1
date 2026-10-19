$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-velocity-position-g01-correction'
$evidence = 'docs/development-evidence/layers-m3-velocity-position-g01-correction'
$runs = "$evidence/runs"
$source = 'src/physics/resident.rs'
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
$test = 'physics::resident::tests::g01_velocity_position_matches_equivalent_rigid_for_degenerate_quaternions'
$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $task
    milestones = [ordered]@{ implementation = 'complete'; validation = 'passed'; delivery_record = 'pending_commit' }
    files = $files
    validation = @(
        [ordered]@{ command = "cargo test --locked --features cuda-probe --lib $test -- --ignored --exact --nocapture --test-threads=1 (before fix)"; result = 'expected fail: 280 of 504 rigid slots differed, strict variant produced NaN'; evidence = "$runs/run-01-repro/result.json" },
        [ordered]@{ command = "cargo test --locked --features cuda-probe --lib $test -- --ignored --exact --nocapture --test-threads=1"; result = 'pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 0 of 504 slots differed; imported non-unit world pose matched; mocap and mocap+G01 PTX equal the pre-fix capture'; evidence = "$runs/run-02-fix/result.json" },
        [ordered]@{ command = 'cargo test --locked --features cuda-probe --lib physics::mass_matrix::tests::runs_rigid_mass_matrix_wrapper_paths_and_recovers -- --ignored --exact --test-threads=1'; result = 'pass: plain rigid_kinematics PTX equals 788c870d baseline'; evidence = "$runs/run-03-ptx-plain/result.json" },
        [ordered]@{ command = 'cargo fmt --check'; result = 'pass'; evidence = "$runs/check-03-format/result.json" },
        [ordered]@{ command = 'cargo clippy --locked --all-targets -- -D warnings'; result = 'pass'; evidence = "$runs/check-04-clippy-default/result.json" },
        [ordered]@{ command = 'cargo clippy --locked --all-targets --features cuda-probe -- -D warnings'; result = 'pass'; evidence = "$runs/check-05-clippy-cuda/result.json" },
        [ordered]@{ command = 'cargo test --locked --lib -- --test-threads=1 physics::resident:: physics::com_velocity:: physics::passive_force:: physics::fluid_force::'; result = 'pass: 14 passed'; evidence = "$runs/check-06-host/result.json" },
        [ordered]@{ command = 'cargo test --locked --features cuda-probe --lib -- --ignored --test-threads=1 physics::resident:: physics::com_velocity:: physics::passive_force:: physics::fluid_force::'; result = 'fail: 27 passed, 1 failed; the failing flex_face overflow test fails identically at base f40a78a'; evidence = "$runs/run-04-gpu-lib/result.json" },
        [ordered]@{ command = 'cargo test --locked --manifest-path target/claude/base-src/Cargo.toml --features cuda-probe --lib physics::resident::tests::flex_face_readback_rejects_guards_nonfinite_and_arithmetic_overflow -- --ignored --exact --test-threads=1'; result = 'fail at base after four setup corrections: same resident_flex_face_output assertion'; evidence = "$runs/run-07-base-flex-face-retry-04/result.json" },
        [ordered]@{ command = 'cargo test --locked --features cuda-probe --test resident_kinematics --test resident_com_velocity --test resident_mocap -- --ignored --test-threads=1'; result = 'pass: 16 passed'; evidence = "$runs/run-05-gpu-integration/result.json" },
        [ordered]@{ command = 'cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe,cuda-probe --test g01 -- --include-ignored --test-threads=1 --nocapture'; result = 'pass: 7 passed'; evidence = "$runs/run-06-g01/result.json" },
        [ordered]@{ command = 'pwsh -NoProfile -File docs/development-evidence/layers-m3-velocity-position-g01-correction/build-inventory.ps1'; result = 'pass: scanner reproduced base rows; 89 compile sites, 56 entry kinds and 71 low-level sites remain'; evidence = "$runs/check-08-inventory/result.json" },
        [ordered]@{ command = 'pwsh -NoProfile -File docs/development-evidence/layers-m3-velocity-position-g01-correction/check-evidence.ps1'; result = 'pass: 19 runs, repro order, PTX variants, source scope and inventory'; evidence = "$runs/check-09-evidence/result.json" }
    )
    preserved_findings = @(
        [ordered]@{ finding = 'pre-existing failure: flex_face_readback_rejects_guards_nonfinite_and_arithmetic_overflow fails at base f40a78a'; runs = @('gpu-lib-regression', 'base-flex-face-retry-04') },
        [ordered]@{ finding = 'base rerun needed four setup corrections: long-path worktree, missing tests, missing docs evidence, relative CUDA PATH'; runs = @('base-flex-face', 'base-flex-face-retry-01', 'base-flex-face-retry-02', 'base-flex-face-retry-03') },
        [ordered]@{ finding = 'G01 world_pose import writes body0, so the private workspace copies data.rigid instead of seeding identity'; action = 'acceptance item 4 updated before implementation' }
    )
    remaining = @(
        'G01 fluid path through PassiveForcePlan::with_fluid_force was not run end to end',
        'frozen MJWarp static comparison was not run for degenerate quaternions',
        "$($inventory.counts.unmigrated_actual_entry_kinds) actual CUDA entry kinds remain unmigrated",
        "$($inventory.counts.unmigrated_low_level_call_sites) product low-level call sites remain unmigrated",
        'pre-existing flex_face overflow test failure remains',
        'historical flg_acc=false still has 1042 failed blocks',
        'NP01 through NP04 remain unaccepted',
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
