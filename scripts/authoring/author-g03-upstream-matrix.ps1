# Offline authoring pinned-environment audit and generation wrapper.
# OFFLINE ONLY: never referenced by the product build, tests, gates or scripts.
param(
    [string]$Venv = 'target/mjwarp-parallel/venv',
    [string]$OutputDir = 'docs/development-evidence/g03-upstream-matrix'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root
$revision = '71da24d956378a87a703b6e1442b13aec0c4ac29'
$expected = @{ python = '3.12.13'; mujoco = '3.15.0'; warp = '1.15.0'; numpy = '2.5.4'; 'mujoco-warp' = '3.15.0' }

function Fail([string]$message) { throw "authoring pinned-env check failed: $message" }
function HexToBase64Url([string]$hex) {
    $bytes = New-Object byte[] ($hex.Length / 2)
    for ($i = 0; $i -lt $bytes.Length; $i++) { $bytes[$i] = [Convert]::ToByte($hex.Substring($i * 2, 2), 16) }
    [Convert]::ToBase64String($bytes).TrimEnd('=').Replace('+', '-').Replace('/', '_')
}

$venvPath = (Resolve-Path $Venv).Path
$python = Join-Path $venvPath 'Scripts/python.exe'
if (-not (Test-Path $python)) { Fail 'pinned python.exe missing' }
$upstream = (Resolve-Path 'target/mjwarp-parallel/upstream').Path
$site = Join-Path $venvPath 'Lib/site-packages'
$outAbs = [IO.Path]::GetFullPath((Join-Path $root $OutputDir))
New-Item -ItemType Directory -Force $outAbs | Out-Null

# 1) Python and package versions (single interpreter start, read-only).
$pyJson = & $python -B -c "import json,sys,platform,importlib.metadata as md,mujoco,warp,numpy,mujoco_warp; print(json.dumps({'python':platform.python_version(),'mujoco':mujoco.__version__,'warp':warp.__version__,'numpy':numpy.__version__,'mujoco-warp':md.version('mujoco-warp'),'mujoco_warp_file':mujoco_warp.__file__,'prefix':sys.prefix}))"
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
$dirty = & git -C $upstream status --porcelain
if ($dirty) { Fail 'upstream checkout is dirty' }

# 3) Installed mujoco_warp sources must equal the pinned upstream sources.
$upSrc = Join-Path $upstream 'mujoco_warp'
$instSrc = Join-Path $site 'mujoco_warp'
$checked = 0
Get-ChildItem $upSrc -Recurse -File -Filter '*.py' | ForEach-Object {
    $rel = $_.FullName.Substring($upSrc.Length + 1)
    $inst = Join-Path $instSrc $rel
    if (-not (Test-Path $inst)) { Fail "installed source missing: $rel" }
    $a = (Get-FileHash $_.FullName -Algorithm SHA256).Hash
    $b = (Get-FileHash $inst -Algorithm SHA256).Hash
    if ($a -ne $b) { Fail "installed source differs from pinned upstream: $rel" }
    $checked++
}
if ($checked -lt 70) { Fail "unexpected source count $checked" }

# 4) warp.dll must match the pinned wheel RECORD.
$record = Import-Csv (Join-Path $site 'warp_lang-1.15.0.dist-info/RECORD') -Header path, hash, size |
    Where-Object path -eq 'warp/bin/warp.dll'
$dll = Join-Path $site 'warp/bin/warp.dll'
$dllSha = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLowerInvariant()
$recordHash = 'sha256=' + (HexToBase64Url $dllSha)
if (-not $record -or $record.hash -ne $recordHash -or [long]$record.size -ne (Get-Item $dll).Length) {
    Fail 'warp.dll differs from pinned wheel RECORD'
}

# 5) Authoring cache must match the frozen reference manifest.
$manifest = Get-Content 'fixtures/mjwarp-mass-solve/manifest.json' -Raw | ConvertFrom-Json
$cacheRoot = 'target/g03-block-reference'
$cacheChecked = 0
foreach ($property in $manifest.cache_hashes.PSObject.Properties) {
    $path = Join-Path $cacheRoot $property.Name
    if (-not (Test-Path $path)) { Fail "authoring cache missing: $($property.Name)" }
    if ((Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $property.Value) {
        Fail "authoring cache hash mismatch: $($property.Name)"
    }
    $cacheChecked++
}
$gpu = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader
if ($LASTEXITCODE -ne 0 -or -not $gpu) { Fail 'GPU/driver query failed' }

$audit = [ordered]@{
    schema = 1
    kind = 'pinned-authoring-environment-audit'
    offline_only = $true
    product_chain_reads = $false
    venv = $venvPath
    venv_isolated = $true
    versions = $versions
    upstream = [ordered]@{ path = $upstream; head = $head; pinned = $revision; clean = $true; python_files_checked = $checked }
    warp_dll = [ordered]@{ path = $dll; sha256 = $dllSha; wheel_record_match = $true }
    authoring_cache = [ordered]@{ root = (Resolve-Path $cacheRoot).Path; checked = $cacheChecked; manifest_match = $true }
    gpu = @($gpu)
}
$auditPath = Join-Path $outAbs 'env-audit.json'
$audit | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $auditPath -Encoding utf8
Write-Host "env audit passed; venv-only python; versions=$($versions.python)/$($versions.mujoco)/$($versions.warp)/$($versions.numpy); sources=$checked; cache=$cacheChecked; gpu=$($gpu -join ';')"

# 6) Pin the CUDA toolkit used by warp (offline local toolchain) and run the
# offline generator (no network; bytecode disabled).
$cuda = Join-Path $root 'target/toolchains/cuda-12.8.1'
if (Test-Path $cuda) {
    $env:CUDA_PATH = $cuda
    $env:PATH = (Join-Path $cuda 'bin') + ';' + $env:PATH
}
& $python -B (Join-Path $root 'scripts/authoring/author-g03-upstream-matrix.py') $outAbs
if ($LASTEXITCODE -ne 0) { Fail "generation failed with exit code $LASTEXITCODE" }
Write-Host "authoring outputs written under $outAbs"
