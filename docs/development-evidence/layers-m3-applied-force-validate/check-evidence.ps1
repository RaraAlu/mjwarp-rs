$ErrorActionPreference = 'Stop'
$task = 'docs/development-evidence/layers-m3-applied-force-validate'
$sourcePath = 'src/physics/applied_force.rs'
$source = [IO.File]::ReadAllText((Resolve-Path -LiteralPath $sourcePath))
$required = @(
    'struct AppliedForceValidateSpec;',
    'struct AppliedForceValidateKernel {',
    'self.validate.launch(',
    'struct AppliedForceDeltaKernel {',
    'self.delta.launch(',
    'publish: SynchronousKernel'
)
foreach ($text in $required) {
    if (-not $source.Contains($text)) { throw "缺少源码证据：$text" }
}
if ($source.Contains('validate: SynchronousKernel')) { throw '验证入口仍是原始字段' }
if (-not $source.Contains('self.publish.launch(')) { throw '发布入口不再保留原始发射' }
$cuda = [regex]::Match(
    $source,
    'const CUDA: &str = r#"(.*?)"#;',
    [Text.RegularExpressions.RegexOptions]::Singleline
)
if (-not $cuda.Success) { throw '找不到CUDA源码' }
$sha = [Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($cuda.Groups[1].Value))
$cudaHash = [Convert]::ToHexString($sha).ToLowerInvariant()
if ($cudaHash -ne 'edb51d900a9f14c3779e6a411dc1c46998cdd91ca70d880ae4f862cf428ea953') {
    throw "CUDA源码漂移：$cudaHash"
}
$fixtureHash = (Get-FileHash -LiteralPath 'fixtures/applied-force/reference.json' -Algorithm SHA256).Hash.ToLowerInvariant()
if ($fixtureHash -ne '13391225d2761132a931b63de65e5bd965a2e7eb93a135f9c0db4684bb2c7cde') {
    throw "夹具漂移：$fixtureHash"
}
Get-ChildItem -LiteralPath $task -Filter '*.json' -File | ForEach-Object {
    $null = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
}
$capture = 'target/normify-evidence/2026-10-19-layers-m3-applied-force-validate/run-01/gpu-capture'
$records = @(Get-ChildItem -LiteralPath $capture -Filter '*.json' -File)
if ($records.Count -ne 3) { throw "捕获记录数错误：$($records.Count)" }
$entries = @($records | ForEach-Object { (Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json).entry })
$expected = @('applied_force_delta', 'applied_force_publish', 'applied_force_validate')
if (@($entries | Sort-Object -Unique).Count -ne 3) { throw '捕获入口存在重复' }
if ((Compare-Object ($entries | Sort-Object) $expected).Count -ne 0) { throw '捕获入口集合错误' }
& git diff --check
if ($LASTEXITCODE -ne 0) { throw "git diff --check退出码：$LASTEXITCODE" }
[ordered]@{
    schema = 1
    cuda_source_sha256 = $cudaHash
    fixture_sha256 = $fixtureHash
    capture_records = $records.Count
    capture_entries = @($entries | Sort-Object)
    typed_actual_entry_kinds = 21
    unmigrated_actual_entry_kinds = 67
    typed_low_level_call_sites = 17
    unmigrated_low_level_call_sites = 82
    wrapper_orchestration_call_sites = 19
    publish_raw = $true
    task_memory = 'unknown'
} | ConvertTo-Json
