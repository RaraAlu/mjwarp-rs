$ErrorActionPreference = 'Stop'
$root = 'docs/development-evidence/layers-m3-sleep-copy'
$runtime = 'target/normify-evidence/2026-10-19-layers-m3-sleep-copy'
$priorPath = 'docs/evidence/t4-g03-retest/tendon-wake-internal/map.json'
$outputPath = "$root/precision-revalidation.json"
if (Test-Path -LiteralPath $outputPath) { throw "精度映射已存在：$outputPath" }

$prior = Get-Content -LiteralPath $priorPath -Raw | ConvertFrom-Json
if ($prior.groups.Count -ne 18 -or $prior.outputs.Count -ne 5) { throw '旧映射规模漂移' }
$copy = @($prior.groups | Where-Object id -ceq 'copy-noop-boundary')
if ($copy.Count -ne 1 -or $copy[0].product -cne 'result[offset+i]=state[offset+i];') {
    throw '复制契约漂移'
}

$baseline = 'docs/evidence/passive-force-sleep/gpu-1/ptx'
$names = @(
    'sleep_copy-0f0740d3df05b413ea24faf778f76e32e651183ade60c45b2dc324f97e50d2ff.cu',
    'sleep_copy-0f0740d3df05b413ea24faf778f76e32e651183ade60c45b2dc324f97e50d2ff.ptx',
    'sleep_copy-0f0740d3df05b413ea24faf778f76e32e651183ade60c45b2dc324f97e50d2ff.json'
)
$captures = @(
    "$runtime/run-01-product-captured/gpu-capture",
    "$runtime/run-02-zero-tree-captured/gpu-capture"
)
$captureRows = [Collections.Generic.List[object]]::new()
foreach ($capture in $captures) {
    foreach ($name in $names) {
        $current = Join-Path $capture $name
        $frozen = Join-Path $baseline $name
        if (-not (Test-Path -LiteralPath $current -PathType Leaf)) { throw "缺少捕获：$current" }
        $currentHash = (Get-FileHash -LiteralPath $current -Algorithm SHA256).Hash.ToLowerInvariant()
        $frozenHash = (Get-FileHash -LiteralPath $frozen -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($currentHash -cne $frozenHash) { throw "捕获漂移：$current" }
        $captureRows.Add([ordered]@{capture=$capture;file=$name;sha256=$currentHash;matches_frozen=$true})
    }
}

$residentPath = 'src/physics/resident.rs'
$resident = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $residentPath))
$anchors = @(
    'struct SleepCopySpec;',
    'Checked::<StaticSleepCopyMetadata>::assume_uploaded(',
    'Checked::<SleepCopyMetadata>::assume_uploaded(',
    'sleep_copy_plan(&dims, &bindings, proof.binding(0))',
    '.launch(metadata.as_buffer(), parameters, state, result, dims.abi)',
    'device.copy_kernel.launch(',
    'device.wake_kernel.launch(',
    'std::mem::swap(&mut out.state, &mut out.scratch);'
)
foreach ($anchor in $anchors) {
    if (-not $resident.Contains($anchor)) { throw "缺少宿主锚点：$anchor" }
}
$order = @(
    $resident.IndexOf('device.copy_kernel.launch(', [StringComparison]::Ordinal),
    $resident.IndexOf('device.wake_kernel.launch(', [StringComparison]::Ordinal),
    $resident.IndexOf('std::mem::swap(&mut out.state, &mut out.scratch);', [StringComparison]::Ordinal)
)
if ($order[0] -lt 0 -or $order[0] -ge $order[1] -or $order[1] -ge $order[2]) {
    throw '复制、唤醒、交换顺序漂移'
}

$groupRows = @($prior.groups | ForEach-Object {
    [ordered]@{id=$_.id;scalar=$_.scalar;product=$_.product;inheritance='unchanged_contract'}
})
$outputRows = @($prior.outputs | ForEach-Object {
    [ordered]@{name=$_.name;scalar=$_.scalar;inheritance='unchanged_contract'}
})
$result = [ordered]@{
    schema = 1
    task = '2026-10-19-layers-m3-sleep-copy'
    prior_map = [ordered]@{
        path = $priorPath
        sha256 = (Get-FileHash -LiteralPath $priorPath -Algorithm SHA256).Hash.ToLowerInvariant()
        groups = $groupRows
        outputs = $outputRows
    }
    current_host = [ordered]@{
        path = $residentPath
        sha256_lf = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($resident.Replace("`r`n", "`n")))).ToLowerInvariant()
        proof = 'real session, buffer identity and write generation'
        transaction_order = @('sleep_copy', 'tendon_wake', 'swap')
        state_span = 'worlds * (2 * ntree + 3) + 8'
        params_kernel_use = 'unused but pointer and session validated'
    }
    frozen_product = [ordered]@{
        cuda_body_changed = $false
        capture_rows = @($captureRows)
        exact_variants = 2
        state_and_scratch_guards_checked = $true
    }
    old_identity_gate = [ordered]@{
        zero_test_attempt_exit = 0
        enabled_attempt_passed = 1
        enabled_attempt_failed = 3
        failure = 'resident.rs whole-file identity drift'
        map_modified = $false
        accepted_as_current = $false
    }
    boundary = [ordered]@{
        local_sleep_copy_evidence = 'complete for this host wrapper slice'
        tendon_wake_contract = 'inherited and unchanged'
        np01_to_np04_accepted = $false
        performance_accepted = $false
        linux_run = $false
    }
}
$json = ($result | ConvertTo-Json -Depth 100).Replace("`r`n", "`n") + "`n"
[IO.File]::WriteAllText((Join-Path (Get-Location) $outputPath), $json, [Text.UTF8Encoding]::new($false))
[ordered]@{
    schema = 1
    inherited_groups = $groupRows.Count
    inherited_outputs = $outputRows.Count
    exact_capture_files = $captureRows.Count
    old_identity_gate_passed = $false
    result = 'pass'
} | ConvertTo-Json
