# Offline authoring wrapper: native MuJoCo f64 CPU reference + U038 two-reference verdict.
# OFFLINE ONLY: never referenced by product build/tests/gates/scripts; CPU only (no GPU).
param([string]$Venv = 'target/mjwarp-parallel/venv')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root
$expected = @{ python = '3.12.13'; mujoco = '3.15.0'; numpy = '2.5.4' }
function Fail([string]$message) { throw "native authoring pinned-env check failed: $message" }
$venvPath = (Resolve-Path $Venv).Path
$python = Join-Path $venvPath 'Scripts/python.exe'
if (-not (Test-Path $python)) { Fail 'pinned python.exe missing' }
$pyJson = & $python -B -c "import json,sys,platform,mujoco,numpy; print(json.dumps({'python':platform.python_version(),'mujoco':mujoco.__version__,'numpy':numpy.__version__,'prefix':sys.prefix}))"
if ($LASTEXITCODE -ne 0) { Fail 'python version probe failed' }
$versions = $pyJson | ConvertFrom-Json
foreach ($key in $expected.Keys) { if ($versions.$key -ne $expected[$key]) { Fail "version mismatch $key=$($versions.$key)" } }
if ($versions.prefix -ne $venvPath) { Fail 'python prefix is not the pinned venv' }
Write-Host "pinned native env OK: python=$($versions.python) mujoco=$($versions.mujoco) numpy=$($versions.numpy)"
& $python -B (Join-Path $PSScriptRoot 'author-u038-native.py')
if ($LASTEXITCODE -ne 0) { throw "native reference generation failed with exit $LASTEXITCODE" }
& $python -B (Join-Path $PSScriptRoot 'judge-u038-native-two-reference.py')
if ($LASTEXITCODE -ne 0) { throw "native two-reference judgment failed with exit $LASTEXITCODE" }
