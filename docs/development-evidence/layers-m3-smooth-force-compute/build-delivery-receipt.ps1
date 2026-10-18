$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-smooth-force-compute'
$evidence = 'docs/development-evidence/layers-m3-smooth-force-compute'
$runtime = "target/normify-evidence/$task"
$source = 'src/physics/smooth_force.rs'
$receiptPath = "docs/development-evidence/changes/$task.json"

function Copy-New([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少交付原件：$Source" }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    if (Test-Path -LiteralPath $Destination) {
        if ((Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash -cne
            (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash) {
            throw "交付归档冲突：$Destination"
        }
        return
    }
    [IO.File]::WriteAllBytes($Destination, [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source)))
}

$postRoot = "$evidence/post-freeze-check"
foreach ($item in @(
    @{source="$runtime/check-06-evidence-final/result.json";destination="$postRoot/evidence-final.run.json"},
    @{source="$runtime/check-06-evidence-final/stdout.log";destination="$postRoot/evidence-final.stdout.log"},
    @{source="$runtime/check-06-evidence-final/stderr.log";destination="$postRoot/evidence-final.stderr.log"}
)) {
    Copy-New $item.source $item.destination
}
$indexReady = Test-Path -LiteralPath "$runtime/check-09-index-bytes/result.json" -PathType Leaf
if ($indexReady) {
    $registration = "$runtime/registrations/index-bytes.json"
    foreach ($item in @(
        @{source=$registration;destination="$evidence/index-check/registration.json"},
        @{source="$runtime/check-09-index-bytes/started.json";destination="$evidence/index-check/started.json"},
        @{source="$runtime/check-09-index-bytes/result.json";destination="$evidence/index-check/result.json"},
        @{source="$runtime/check-09-index-bytes/stdout.log";destination="$evidence/index-check/stdout.log"},
        @{source="$runtime/check-09-index-bytes/stderr.log";destination="$evidence/index-check/stderr.log"}
    )) {
        Copy-New $item.source $item.destination
    }
}
$postMembers = @(Get-ChildItem -LiteralPath $postRoot -Recurse -File |
    Where-Object Name -ne 'manifest.json' | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            path = $_.FullName.Substring((Resolve-Path -LiteralPath $postRoot).Path.Length + 1).Replace('\', '/')
            bytes = $_.Length
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    })
$postManifest = [ordered]@{schema=1;root=$postRoot;self_excluded=$true;member_count=$postMembers.Count;members=$postMembers}
$postJson = ($postManifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'manifest.json'), $postJson, [Text.UTF8Encoding]::new($false))

& git add -- $source $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0) { throw "git diff退出码：$LASTEXITCODE" }
if ($staged.Count -eq 0) { throw '暂存区为空' }
foreach ($path in $staged) {
    if ($path -ne $source -and -not $path.StartsWith("$evidence/")) {
        throw "暂存范围外文件：$path"
    }
}
$files = @($staged | Sort-Object | ForEach-Object {
    $blob = @(& git rev-parse ":$_")[0]
    if ($LASTEXITCODE -ne 0 -or $blob -notmatch '^[0-9a-f]{40}$') { throw "对象解析失败：$_" }
    [ordered]@{path=$_;blob=$blob}
})
$inventory = Get-Content -LiteralPath "$evidence/unmigrated-launch-inventory.json" -Raw | ConvertFrom-Json
$next = $inventory.acceptance_boundary.next_candidate
$validation = [Collections.Generic.List[object]]::new()
$validation.Add([ordered]@{command='cargo test --locked --features cuda-probe --lib physics::smooth_force::tests::compute_wrapper_rejects_capacity_context_alias_span_dimension_and_proof_faults -- --exact --test-threads=1';result='pass: 1 passed';evidence="$evidence/formal-run/host.run.json"})
$validation.Add([ordered]@{command='cargo test --locked --features cuda-probe --lib physics::smooth_force::tests::gpu_m3_compute_wrapper_sleep_disabled_preserves_failure_boundaries -- --ignored --exact --test-threads=1 --nocapture';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/gpu-off.run.json"})
$validation.Add([ordered]@{command='cargo test --locked --features cuda-probe --lib physics::smooth_force::tests::gpu_m3_compute_wrapper_sleep_enabled_covers_sources_and_ntree_zero -- --ignored --exact --test-threads=1 --nocapture';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/gpu-on.run.json"})
$validation.Add([ordered]@{command='cargo fmt --check';result='first failed and preserved; registered format apply then final pass';evidence="$evidence/formal-run/fmt-final.run.json"})
$validation.Add([ordered]@{command='cargo clippy --locked --features cuda-probe --lib -- -D warnings';result='pass';evidence="$evidence/formal-run/clippy.run.json"})
$validation.Add([ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-smooth-force-compute/check-formal-evidence.ps1';result='pass: 64 manifest members plus infrastructure runs';evidence="$evidence/post-freeze-check/result.json"})
if ($indexReady) {
    $validation.Add([ordered]@{command='pwsh -NoProfile -File scripts/check-evidence-blobs.ps1 -Manifest docs/development-evidence/layers-m3-smooth-force-compute/formal-run/manifest.json -Map docs/development-evidence/layers-m3-smooth-force-compute/source-map.json -Revision INDEX';result='pass';evidence="$evidence/index-check/result.json"})
}
$receipt = [ordered]@{
    schema = 1
    kind = 'delivery'
    task_id = $task
    milestones = [ordered]@{
        implementation = 'complete'
        validation = 'passed'
        delivery_record = 'pending_commit'
    }
    process_compliance = [ordered]@{
        current_task_run_registration = 'complete'
        current_task_failed_runs_preserved = $true
        predecessor_missing_registrations = 6
        predecessor_gap_record = 'docs/development-evidence/layers-m3-smooth-force-validate/retry-registration-gap.json'
        predecessor_gap_closed = $false
        full_m3_process_compliance = $false
        parent_m3_passed = $false
    }
    files = $files
    validation = @($validation)
    preserved_failures = @(
        [ordered]@{
            gate = 'format'
            evidence = "$evidence/formal-run/fmt-first.run.json"
            exit_code = 1
            reason = '新增Rust代码尚未应用rustfmt排版'
        }
    )
    remaining = @(
        "64 actual CUDA entry kinds and 79 product low-level call sites remain unmigrated",
        "next raw entry is $next",
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'U016 and NP01 through NP04 remain unaccepted',
        'parent M3 remains proposed and incomplete',
        'the predecessor task retains six missing retry preregistrations'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{schema=1;task=$task;covered_files=$files.Count;receipt=$receiptPath;index_evidence=$indexReady;result='staged'} | ConvertTo-Json
