# Offline authoring wrapper: pinned-environment audit + upstream G26 energy_vel
# reference generation. OFFLINE ONLY: never referenced by product build/tests/
# gates/scripts; product tests still run cargo only and never start Python.
param(
    [string]$Venv = 'target/mjwarp-parallel/venv',
    [string]$OutputDir = 'docs/development-evidence/g26-complete'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root
$revision = '71da24d956378a87a703b6e1442b13aec0c4ac29'
$expected = @{ python = '3.12.13'; mujoco = '3.15.0'; warp = '1.15.0'; numpy = '2.5.4'; 'mujoco-warp' = '3.15.0' }
$warpDllSha = '7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6'

function Fail([string]$message) { throw "authoring pinned-env check failed: $message" }
function Sha256([string]$path) { (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() }

$venvPath = (Resolve-Path $Venv).Path
$python = Join-Path $venvPath 'Scripts/python.exe'
if (-not (Test-Path $python)) { Fail 'pinned python.exe missing' }
$upstream = (Resolve-Path 'target/mjwarp-parallel/upstream').Path
$site = Join-Path $venvPath 'Lib/site-packages'

# 1) Versions and interpreter isolation (single interpreter start, read-only).
$pyJson = & $python -B -c "import json,sys,platform,importlib.metadata as md,mujoco,warp,numpy,mujoco_warp; print(json.dumps({'python':platform.python_version(),'mujoco':mujoco.__version__,'warp':warp.__version__,'numpy':numpy.__version__,'mujoco-warp':md.version('mujoco-warp'),'prefix':sys.prefix}))"
if ($LASTEXITCODE -ne 0) { Fail 'python version probe failed' }
$versions = $pyJson | ConvertFrom-Json
foreach ($key in $expected.Keys) {
    if ($versions.$key -ne $expected[$key]) { Fail "version mismatch $key=$($versions.$key) expected $($expected[$key])" }
}
if ($versions.prefix -ne $venvPath) { Fail "python prefix $($versions.prefix) is not the pinned venv" }
$cfg = Get-Content (Join-Path $venvPath 'pyvenv.cfg') -ErrorAction Stop
if (-not ($cfg -match 'include-system-site-packages = false')) { Fail 'venv is not isolated from system site-packages' }

# 2) Upstream checkout pinned commit and clean worktree.
$head = (& git -C $upstream rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $head -ne $revision) { Fail "upstream HEAD $head is not pinned $revision" }
if (& git -C $upstream status --porcelain) { Fail 'upstream checkout is dirty' }

# 3) Installed mujoco_warp sources must equal pinned upstream sources.
$upSrc = Join-Path $upstream 'mujoco_warp'
$instSrc = Join-Path $site 'mujoco_warp'
$checked = 0
Get-ChildItem $upSrc -Recurse -File -Filter '*.py' | ForEach-Object {
    $rel = $_.FullName.Substring($upSrc.Length + 1)
    $inst = Join-Path $instSrc $rel
    if (-not (Test-Path $inst)) { Fail "installed source missing: $rel" }
    if ((Sha256 $_.FullName) -ne (Sha256 $inst)) { Fail "installed source differs from pinned upstream: $rel" }
    $checked++
}
if ($checked -lt 70) { Fail "unexpected source count $checked" }

# 4) warp.dll must match the pinned wheel artifact.
$warpDll = Join-Path $site 'warp/bin/warp.dll'
if (-not (Test-Path $warpDll)) { Fail 'warp.dll missing' }
if ((Sha256 $warpDll) -ne $warpDllSha) { Fail 'warp.dll does not match the pinned wheel RECORD' }

# 5) Reference GPU/driver.
$gpu = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>&1
if ($LASTEXITCODE -ne 0) { Fail 'nvidia-smi failed' }
Write-Host "pinned env OK: python=$($versions.python) mujoco=$($versions.mujoco) warp=$($versions.warp) mujoco-warp=$($versions.'mujoco-warp')"
Write-Host "gpu=$gpu upstream=$head installed_sources=$checked"

$outAbs = [IO.Path]::GetFullPath((Join-Path $root $OutputDir))
& $python -B (Join-Path $PSScriptRoot 'author-g26-energy-vel.py') $outAbs
if ($LASTEXITCODE -ne 0) { throw "authoring generation failed with exit $LASTEXITCODE" }
