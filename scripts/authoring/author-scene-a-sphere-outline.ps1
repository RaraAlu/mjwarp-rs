# Offline authoring wrapper: pinned-environment audit + scene-a sphere-outline lists.
# OFFLINE ONLY: never referenced by the product build, product tests, cargo gates or the
# product scripts; product tests only read the frozen fixtures and never start Python.
#
# Boundary basis: user-approved offline authoring exception (2026-10-22). Any mismatch in
# the pinned environment or the upstream identity below aborts before the helper runs.
param(
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Venv = 'target/mjwarp-parallel/venv'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root
$revision = '71da24d956378a87a703b6e1442b13aec0c4ac29'
$expected = @{ python = '3.12.13'; mujoco = '3.15.0'; warp = '1.15.0'; numpy = '2.5.4'; 'mujoco-warp' = '3.15.0' }

function Fail([string]$message) { throw "authoring pinned-env check failed: $message" }
function Sha256([string]$path) { (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() }

$venvPath = (Resolve-Path $Venv).Path
$python = Join-Path $venvPath 'Scripts/python.exe'
if (-not (Test-Path $python)) { Fail 'pinned python.exe missing' }
$upstream = (Resolve-Path 'target/mjwarp-parallel/upstream').Path
$site = Join-Path $venvPath 'Lib/site-packages'

$pyJson = & $python -B -c "import json,sys,platform,importlib.metadata as md,mujoco,warp,numpy,mujoco_warp; print(json.dumps({'python':platform.python_version(),'mujoco':mujoco.__version__,'warp':warp.__version__,'numpy':numpy.__version__,'mujoco-warp':md.version('mujoco-warp'),'prefix':sys.prefix}))"
if ($LASTEXITCODE -ne 0) { Fail 'python version probe failed' }
$versions = $pyJson | ConvertFrom-Json
foreach ($key in $expected.Keys) {
    if ($versions.$key -ne $expected[$key]) { Fail "version mismatch $key=$($versions.$key)" }
}
if ($versions.prefix -ne $venvPath) { Fail 'python prefix is not the pinned venv' }

$head = (& git -C $upstream rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $head -ne $revision) { Fail "upstream HEAD $head is not pinned" }
if (& git -C $upstream status --porcelain) { Fail 'upstream checkout is dirty' }
$upSrc = Join-Path $upstream 'mujoco_warp'
$instSrc = Join-Path $site 'mujoco_warp'
$checked = 0
Get-ChildItem $upSrc -Recurse -File -Filter '*.py' | ForEach-Object {
    $rel = $_.FullName.Substring($upSrc.Length + 1)
    $inst = Join-Path $instSrc $rel
    if (-not (Test-Path $inst)) { Fail "installed source missing: $rel" }
    if ((Sha256 $_.FullName) -ne (Sha256 $inst)) { Fail "installed source differs: $rel" }
    $checked++
}
if ($checked -lt 70) { Fail "unexpected source count $checked" }

$gpu = (& nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>&1)
if ($LASTEXITCODE -ne 0) { Fail 'nvidia-smi failed' }
Write-Host "pinned env OK: python=$($versions.python) numpy=$($versions.numpy) mujoco=$($versions.mujoco) warp=$($versions.warp) gpu=$gpu upstream=$head installed_sources=$checked"

$helper = Join-Path $PSScriptRoot 'author-scene-a-sphere-outline.py'
$replay = Join-Path $PSScriptRoot 'scene_a_outline_replay.py'
if (-not (Test-Path $helper)) { Fail 'helper missing' }
if (-not (Test-Path $replay)) { Fail 'replay module missing' }
Write-Host ("author-scene-a-sphere-outline: helper sha256={0}" -f (Sha256 $helper))
Write-Host ("author-scene-a-sphere-outline: replay sha256={0}" -f (Sha256 $replay))

$outPath = $Out
if (-not [System.IO.Path]::IsPathRooted($outPath)) { $outPath = Join-Path $root $outPath }
$outPath = [System.IO.Path]::GetFullPath($outPath)
$fixtures = (Resolve-Path 'fixtures').Path
if ($outPath.StartsWith($fixtures, [System.StringComparison]::OrdinalIgnoreCase)) {
    Fail "output must not be written under fixtures/: $outPath"
}
$run2 = $outPath + '.run2'

function Invoke-Helper([string]$target) {
    Write-Host ("author-scene-a-sphere-outline: run -> {0}" -f $target)
    & $python -B $helper --repo $root --out $target | Out-Host
    return $LASTEXITCODE
}

if (Test-Path -LiteralPath $run2) { Remove-Item -LiteralPath $run2 -Recurse -Force }
$code1 = Invoke-Helper $outPath
if ($code1 -ne 0 -and $code1 -ne 4) { Fail "helper failed with exit code $code1" }
$code2 = Invoke-Helper $run2
if ($code2 -ne $code1) { Fail "helper exit code is not deterministic ($code1 vs $code2)" }

$files1 = @(Get-ChildItem -LiteralPath $outPath -File | Sort-Object Name)
$files2 = @(Get-ChildItem -LiteralPath $run2 -File | Sort-Object Name)
if ($files1.Count -ne $files2.Count) { Fail 'determinism: output file count differs' }
foreach ($file in $files1) {
    $other = Join-Path $run2 $file.Name
    if (-not (Test-Path -LiteralPath $other)) { Fail "determinism: missing $($file.Name) in run2" }
    if ((Sha256 $file.FullName) -ne (Sha256 $other)) { Fail "determinism: $($file.Name) differs between runs" }
}
Write-Host ("author-scene-a-sphere-outline: determinism=two-runs-byte-identical files={0}" -f $files1.Count)

$sums = Join-Path $outPath 'sha256sums.txt'
if (-not (Test-Path -LiteralPath $sums)) { Fail 'sha256sums.txt missing' }
$verified = 0
foreach ($line in Get-Content -LiteralPath $sums) {
    if ($line -notmatch '^([0-9a-f]{64})  (.+)$') { continue }
    $want = $Matches[1]
    $path = Join-Path $outPath $Matches[2]
    if (-not (Test-Path -LiteralPath $path)) { Fail "listed file missing: $($Matches[2])" }
    $got = Sha256 $path
    if ($got -ne $want) { Fail "sha mismatch $($Matches[2]): want $want got $got" }
    $verified++
    Write-Host ("  sha256 {0}  {1}" -f $got, $Matches[2])
}
Write-Host ("author-scene-a-sphere-outline: sha256 verified={0}" -f $verified)
Remove-Item -LiteralPath $run2 -Recurse -Force
Write-Host ("author-scene-a-sphere-outline: helper exit code {0} (0=freeze ok, 4=criterion verdict fails)" -f $code1)
if ($code1 -eq 4) {
    Write-Host 'author-scene-a-sphere-outline: criterion-v2 freeze verdict FAILS - see b-recompute.txt; the window must not proceed'
}
exit $code1
