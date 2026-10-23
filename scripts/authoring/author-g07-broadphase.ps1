# Offline authoring wrapper: pinned-environment audit + G07 broadphase references.
# OFFLINE ONLY: never referenced by product build/tests/gates/scripts.
# Product tests only read the static fixture; they never start Python.
param([string]$Venv = 'target/mjwarp-parallel/venv')
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
foreach ($key in $expected.Keys) { if ($versions.$key -ne $expected[$key]) { Fail "version mismatch $key=$($versions.$key)" } }
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
$gpu = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>&1
if ($LASTEXITCODE -ne 0) { Fail 'nvidia-smi failed' }
Write-Host "pinned env OK: python=$($versions.python) warp=$($versions.warp) gpu=$gpu upstream=$head installed_sources=$checked"
& $python -B (Join-Path $PSScriptRoot 'author-g07-broadphase.py')
if ($LASTEXITCODE -ne 0) { throw "authoring generation failed with exit $LASTEXITCODE" }
$fixture = 'fixtures/broadphase/broadphase-cases.json'
$evidence = 'docs/development-evidence/backlog-batch2/2026-10-11-roadmap-g07-p5/authoring.json'
foreach ($path in @($fixture, $evidence)) {
    if (-not (Test-Path $path)) { Fail "expected output missing: $path" }
    Write-Host ("output {0} sha256={1}" -f $path, (Sha256 $path))
}
