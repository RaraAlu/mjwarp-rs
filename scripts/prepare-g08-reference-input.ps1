#Requires -Version 7.0
[CmdletBinding()]
param(
    [string]$RepositoryRoot = (Split-Path -Parent $PSScriptRoot),
    [string]$OutputDirectory = 'target/d04-g08-input/native-v1',
    [switch]$SelfTest
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = [IO.Path]::GetFullPath($RepositoryRoot)
function Sha([string]$Path) { return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
function Assert-Hash([string]$Path, [string]$Expected) {
    if ((Sha $Path) -ne $Expected) { throw "Pinned input/source mismatch: $Path" }
}
function Assert-Catalog($Catalog) {
    if ($Catalog.schema -ne 1 -or $Catalog.scalar -ne 'f32' -or $Catalog.native_model_version -ne '3.12.0') { throw 'Input version mismatch' }
    if ($Catalog.nq -ne 7 -or $Catalog.nv -ne 6 -or $Catalog.states.Count -ne 7) { throw 'Input coverage mismatch' }
    $ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($state in $Catalog.states) {
        if (-not $ids.Add($state.id)) { throw 'Duplicate state ID' }
        if ($state.qpos.Count -ne 7 -or $state.qvel.Count -ne 6) { throw 'State width mismatch' }
        foreach ($v in @($state.qpos) + @($state.qvel)) {
            if (-not [single]::IsFinite([single]$v)) { throw 'Nonfinite f32 input' }
        }
        if (($state.qpos[3..6] -join ',') -ne '1,0,0,0') { throw 'This input recipe requires identity quaternion' }
    }
}
function Expect-Rejection([scriptblock]$Action, [string]$Pattern) {
    try { & $Action } catch { if ($_.Exception.Message -notmatch $Pattern) { throw }; return }
    throw "Expected input rejection did not occur: $Pattern"
}
function Clone($Value) { return ($Value | ConvertTo-Json -Depth 30 | ConvertFrom-Json -AsHashtable) }

$xml = Join-Path $root 'fixtures/collision-reference-input/plane-sphere.xml'
$statesPath = Join-Path $root 'fixtures/collision-reference-input/plane-sphere-states.json'
# 源文本允许Git换行转换；其规范内容保持固定。
foreach ($pin in @(
    @($xml, '0f44dd9c2a05f5c9017ca22446cc0aacb0474997b71f734d18b0ea02bff1d165'),
    @($statesPath, '8777184b3d77cb21b183bbc89cbcf6b02500091c52c82e07e055caf191fcc3ab')
)) {
    $bytes = [Text.Encoding]::UTF8.GetBytes([IO.File]::ReadAllText($pin[0]).Replace("`r`n", "`n"))
    if ([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant() -ne $pin[1]) { throw 'Input recipe drift' }
}
$catalog = Get-Content -LiteralPath $statesPath -Raw | ConvertFrom-Json -AsHashtable
Assert-Catalog $catalog
$upstream = Join-Path $root 'target/mjwarp-parallel/upstream/mujoco_warp/_src'
Assert-Hash (Join-Path $upstream 'collision_primitive.py') 'a5573347f51e4e3b308049bcb665bf656c10d583faedb419467262816853cf60'
Assert-Hash (Join-Path $upstream 'collision_primitive_core.py') '2f1cb43cea2d790540e75231297abc829acab5bd892971625464358b3b2757ee'
$toolRoot = Join-Path $root 'target/toolchains/mujoco-3.12.0'
$archive = Join-Path $toolRoot 'mujoco-3.12.0-windows-x86_64.zip'
Assert-Hash $archive 'ffe071c2747dd9513a1c59e7d2428bb678d887f9edec9eb9674b4288a248a8e9'
$compiler = Join-Path $toolRoot 'package/bin/compile.exe'
$library = Join-Path $toolRoot 'package/bin/mujoco.dll'
# 比对官方冻结包内字节；不信任同名外部可执行文件。
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::OpenRead($archive)
try {
    foreach ($pair in @(@('bin/compile.exe', $compiler), @('bin/mujoco.dll', $library))) {
        $entries = @($zip.Entries | Where-Object { $_.FullName.Replace('\', '/') -eq $pair[0] })
        if ($entries.Count -ne 1) { throw 'Official package entry mismatch' }
        $stream = $entries[0].Open()
        try { $expected = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream)).ToLowerInvariant() }
        finally { $stream.Dispose() }
        Assert-Hash $pair[1] $expected
    }
} finally { $zip.Dispose() }

$targetBase = [IO.Path]::GetFullPath((Join-Path $root 'target')).TrimEnd([char[]]@('/', '\')) + [IO.Path]::DirectorySeparatorChar
$output = [IO.Path]::GetFullPath((Join-Path $root $OutputDirectory))
if (-not $output.StartsWith($targetBase, [StringComparison]::OrdinalIgnoreCase)) { throw 'Authoring output must remain below target' }
if (Test-Path -LiteralPath $output) { throw 'Refuse to overwrite existing authoring output' }
New-Item -ItemType Directory -Path $output | Out-Null
$binary = Join-Path $output 'plane-sphere.mjb'
# 离线输入制作，不调用mj_forward、GPU或产品测试。
& $compiler $xml $binary 2>&1 | Tee-Object -FilePath (Join-Path $output 'compiler.log') | Out-Host
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw 'Native model compilation failed' }
if ((Get-Item -LiteralPath $binary).Length -eq 0) { throw 'Native model is empty' }
$dump = Join-Path $output 'plane-sphere-native.txt'
& $compiler $binary $dump 2>&1 | Tee-Object -FilePath (Join-Path $output 'reload.log') | Out-Host
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $dump -PathType Leaf)) { throw 'Native MJB reload failed' }
$text = [IO.File]::ReadAllText($dump)
if (-not $text.StartsWith('MuJoCo version 3.12.0')) { throw 'Loaded native model version mismatch' }
$dimensions = [ordered]@{}
foreach ($pair in @(@('nq', 7), @('nv', 6), @('nbody', 2), @('ngeom', 2))) {
    $match = [regex]::Match($text, '(?m)^\s*' + $pair[0] + '\s+(\d+)\s*$')
    if (-not $match.Success -or [int]$match.Groups[1].Value -ne $pair[1]) { throw "Native dimension mismatch: $($pair[0])" }
    $dimensions[$pair[0]] = [int]$match.Groups[1].Value
}
$normalized = @($catalog.states | ForEach-Object {
    $state = $_
    $qpos = @($state.qpos | ForEach-Object { [single]$_ })
    $qvel = @($state.qvel | ForEach-Object { [single]$_ })
    [ordered]@{ id = $state.id; qpos = $qpos; qvel = $qvel; qpos_bits = @($qpos | ForEach-Object { [BitConverter]::SingleToUInt32Bits($_) }); qvel_bits = @($qvel | ForEach-Object { [BitConverter]::SingleToUInt32Bits($_) }) }
})
$preparedStates = Join-Path $output 'states-f32.json'
[IO.File]::WriteAllText($preparedStates, (ConvertTo-Json -InputObject $normalized -Depth 10), [Text.UTF8Encoding]::new($false))
$passed = 0
if ($SelfTest) {
    $bad = Clone $catalog; $bad.states = @()
    Expect-Rejection { Assert-Catalog $bad } 'coverage'
    $bad = Clone $catalog; $bad.states[0].qpos = @(0, 0, 0, 1, 0, 0)
    Expect-Rejection { Assert-Catalog $bad } 'width'
    $bad = Clone $catalog; $bad.states[1].id = $bad.states[0].id
    Expect-Rejection { Assert-Catalog $bad } 'Duplicate'
    $bad = Clone $catalog; $bad.states[0].qpos[0] = [double]::PositiveInfinity
    Expect-Rejection { Assert-Catalog $bad } 'Nonfinite'
    $bad = Clone $catalog; $bad.native_model_version = '3.15.0'
    Expect-Rejection { Assert-Catalog $bad } 'version'
    $passed = 5
}
$receipt = [ordered]@{
    schema = 1; scope = 'G08 native reference inputs only; not a GPU reference'
    native_version_pin = '3.12.0'; expected_nq = 7; expected_nv = 6; states = $normalized.Count
    upstream_revision = '71da24d956378a87a703b6e1442b13aec0c4ac29'; upstream_function = 'plane_sphere_wrapper'
    model_xml_sha256_raw = Sha $xml; input_recipe_sha256_raw = Sha $statesPath
    model_binary_sha256 = Sha $binary; prepared_states_sha256 = Sha $preparedStates
    native_dump_sha256 = Sha $dump; native_dimensions = $dimensions
    native_compiler_sha256 = Sha $compiler; native_library_sha256 = Sha $library
    state_scalar = 'f32'; native_model_num_scalar = 'f64'; gpu_model_converted = $false
    native_layout_loaded_for_verification = $true; quaternion_order = 'wxyz'; native_compiler_invocations = 2
    negative_tests_passed = $passed; python_invocations = 0; gpu_sessions = 0; cpu_physics_steps = 0
    upstream_cuda_entry = $null; gpu_artifact_captured = $false; contact_output_available = $false; reference_accepted = $false
    producer_requirements = @('verify native binding 3.12.0 and loaded nq/nv', 'resolve plane/sphere geom IDs by name', 'launch frozen collision pipeline', 'capture actual CUDA/PTX and typed contact outputs')
}
$json = $receipt | ConvertTo-Json -Depth 6
[IO.File]::WriteAllText((Join-Path $output 'receipt.json'), $json, [Text.UTF8Encoding]::new($false))
$json
