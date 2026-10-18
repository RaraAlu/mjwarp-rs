$ErrorActionPreference = 'Stop'
$task = '2026-10-19-layers-m3-rne-bias'
$evidence = "docs/development-evidence/layers-m3-rne-bias"
$runtime = "target/normify-evidence/$task"
$source = 'src/physics/rne_bias.rs'
$currentSource = "$evidence/frozen/current-rne-bias.rs"
$receiptPath = "docs/development-evidence/changes/$task.json"

function Copy-New([string]$Source, [string]$Destination) {
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "缺少交付原件：$Source" }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent | Out-Null }
    $bytes = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Source))
    $stream = [IO.FileStream]::new(
        (Join-Path (Get-Location) $Destination),
        [IO.FileMode]::CreateNew,
        [IO.FileAccess]::Write,
        [IO.FileShare]::Read
    )
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
}

$postRoot = "$evidence/post-freeze-check"
foreach ($item in @(
    @{source="$runtime/check-13-evidence-final/result.json";destination="$postRoot/evidence-final.run.json"},
    @{source="$runtime/check-13-evidence-final/stdout.log";destination="$postRoot/evidence-final.stdout.log"},
    @{source="$runtime/check-13-evidence-final/stderr.log";destination="$postRoot/evidence-final.stderr.log"}
)) {
    Copy-New $item.source $item.destination
}
$postMembers = @(Get-ChildItem -LiteralPath $postRoot -Recurse -File |
    Where-Object Name -ne 'manifest.json' | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            path=$_.FullName.Substring((Resolve-Path -LiteralPath $postRoot).Path.Length + 1).Replace('\', '/')
            bytes=$_.Length
            sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    })
$postManifest = [ordered]@{
    schema=1
    root=$postRoot
    self_excluded=$true
    member_count=$postMembers.Count
    members=$postMembers
}
$postJson = ($postManifest | ConvertTo-Json -Depth 6).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path $postRoot 'manifest.json'), $postJson, [Text.UTF8Encoding]::new($false))

if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -cne
    (Get-FileHash -LiteralPath $currentSource -Algorithm SHA256).Hash) {
    throw '当前源码与冻结原件不一致'
}
& git add -- $source $evidence
if ($LASTEXITCODE -ne 0) { throw "git add退出码：$LASTEXITCODE" }
$sourceBlob = @(& git rev-parse ":$source")[0].Trim()
$currentBlob = @(& git rev-parse ":$currentSource")[0].Trim()
if ($LASTEXITCODE -ne 0 -or $sourceBlob -cne $currentBlob) {
    throw '暂存源码与冻结原件不一致'
}
$staged = @(& git diff --cached --name-only --diff-filter=ACDMRTUXB | Where-Object { $_ -ne $receiptPath })
if ($LASTEXITCODE -ne 0 -or $staged.Count -eq 0) { throw '暂存区读取失败或为空' }
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
$receipt = [ordered]@{
    schema=1
    kind='delivery'
    task_id=$task
    milestones=[ordered]@{
        implementation='complete'
        validation='passed'
        delivery_record='pending_commit'
    }
    files=$files
    validation=@(
        [ordered]@{command='cargo test --locked --lib rne_bias_plan -- --test-threads=1';result='pass: 2 passed';evidence="$evidence/formal-run/runs/host-final.run.json"},
        [ordered]@{command='cargo test --locked --lib -- --test-threads=1';result='pass: 248 passed';evidence="$evidence/formal-run/runs/default-lib.run.json"},
        [ordered]@{command='cargo clippy --locked --all-targets --features cuda-probe -- -D warnings';result='pass after registered repair';evidence="$evidence/formal-run/runs/clippy-final.run.json"},
        [ordered]@{command='cargo fmt --check';result='pass after registered repair';evidence="$evidence/formal-run/runs/format-final.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --test resident_rne_bias reference_hashes_freeze_native_bias_inputs_gravities_and_tolerances -- --exact --test-threads=1';result='pass: 1 input and tolerance test';evidence="$evidence/formal-run/runs/input-tolerance-gate.run.json"},
        [ordered]@{command='cargo test --locked --features cuda-probe --lib physics::rne_bias::tests::runs_rne_bias_wrapper_and_recovers_after_real_metadata_write -- --ignored --exact --test-threads=1';result='pass on local NVIDIA GeForce RTX 4070 Ti SUPER: 1 passed';evidence="$evidence/formal-run/runs/gpu-product.run.json"},
        [ordered]@{command='pwsh -NoProfile -File docs/development-evidence/layers-m3-rne-bias/check-formal-evidence.ps1';result='pass: 167 members';evidence="$evidence/post-freeze-check/evidence-final.run.json"}
    )
    preserved_failures=@(
        [ordered]@{gate='format';exit_code=1;evidence="$evidence/formal-run/runs/format-first.run.json";reason='首次rustfmt检查发现差异'},
        [ordered]@{gate='clippy';exit_code=101;evidence="$evidence/formal-run/runs/clippy-first.run.json";reason='首次检查发现参数数量诊断'},
        [ordered]@{gate='precision-map';exit_code=1;evidence="$evidence/formal-run/runs/precision-first.run.json";reason='首次脚本使用了错误JSON键'},
        [ordered]@{gate='evidence';exit_code=1;evidence="$evidence/formal-run/runs/evidence-first.run.json";reason='首次脚本误把PTX寄存器编号判为f64'},
        [ordered]@{gate='delivery-preflight';exit_code=1;evidence="$evidence/formal-run/runs/delivery-preflight-first.run.json";reason='首次预检缺少回执路径绑定'}
    )
    preserved_findings=@(
        [ordered]@{
            finding='PTX register names contain percent-f64 and do not denote f64 instructions'
            initial="$evidence/precision-revalidation.json"
            corrected="$evidence/precision-revalidation-v2.json"
            verification="$evidence/formal-run/runs/evidence-final.run.json"
        }
    )
    remaining=@(
        "$($inventory.counts.unmigrated_actual_entry_kinds) actual CUDA entry kinds remain unmigrated",
        "$($inventory.counts.unmigrated_low_level_call_sites) product low-level call sites remain unmigrated",
        'historical flg_acc=false still has 1042 failed blocks',
        'NP01 through NP04 remain unaccepted',
        'performance remains unaccepted',
        'release and Linux were not run',
        'task GPU memory usage remains unknown',
        'parent M3 remains incomplete'
    )
}
$json = ($receipt | ConvertTo-Json -Depth 12).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $receiptPath), $json, [Text.UTF8Encoding]::new($false))
& git add -- $receiptPath
if ($LASTEXITCODE -ne 0) { throw "回执暂存失败：$LASTEXITCODE" }
[ordered]@{
    schema=1
    task=$task
    covered_files=$files.Count
    receipt=$receiptPath
    result='staged'
} | ConvertTo-Json
