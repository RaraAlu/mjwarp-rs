# Offline natural validation: generated upstream matrix -> MathDx potrf_11 ->
# bit-exact comparison against the frozen upstream factor (mixed-joints world=4).
# OFFLINE ONLY: never referenced by the product build, tests, gates or scripts.
param(
    [string]$OutputDir = 'docs/development-evidence/g03-upstream-matrix',
    [string]$Cubin = 'target/mathdx11/out/factor-cdd46856c1496194/potrf_11_11_1_89_32_1_1_5_x_x_0.cubin',
    [string]$Oracle = 'target/mathdx11/oracle.exe'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $root
function Fail([string]$message) { throw "upstream-matrix validation failed: $message" }

$out = [IO.Path]::GetFullPath((Join-Path $root $OutputDir))
$matrixPath = Join-Path $out 'matrix-world4.f32'
$provenancePath = Join-Path $out 'provenance.json'
if (-not (Test-Path $matrixPath)) { Fail 'generated matrix-world4.f32 missing; run author-g03-upstream-matrix.ps1 first' }
if (-not (Test-Path $provenancePath)) { Fail 'provenance.json missing' }
if (-not (Test-Path $Cubin)) {
    Fail "MathDx potrf_11 cubin missing: $Cubin (rebuild instructions in docs/development-evidence/g03-mathdx-alignment/README.md)"
}
if (-not (Test-Path $Oracle)) {
    Fail "oracle.exe missing: $Oracle (compile docs/development-evidence/g03-mathdx-alignment/tools/oracle.cpp as documented)"
}
$cubinSha = (Get-FileHash $Cubin -Algorithm SHA256).Hash.ToLowerInvariant()
$expectedCubinSha = '36e7f49ef7ff7900a639bd464c8db559771f3a50884e97842b407e3e96d2c765'
if ($cubinSha -ne $expectedCubinSha) { Fail "potrf_11 cubin hash $cubinSha is not the phase-1 verified artifact" }

$outPath = Join-Path $out 'potrf-world4.out'
if (Test-Path $outPath) { Remove-Item $outPath -Force }
& $Oracle $Cubin factor $matrixPath $outPath 'mathdx11_factor'
if ($LASTEXITCODE -ne 0) { Fail "oracle run failed with exit code $LASTEXITCODE" }
$raw = [IO.File]::ReadAllBytes($outPath)
if ($raw.Length -ne 4 * 122) { Fail "oracle output size $($raw.Length) unexpected" }
$factor = New-Object 'float[]' 121
[Buffer]::BlockCopy($raw, 0, $factor, 0, 4 * 121)
$status = [BitConverter]::ToInt32($raw, 4 * 121)

$reference = Get-Content 'fixtures/mjwarp-mass-solve/reference.json' -Raw | ConvertFrom-Json
$model = $reference.models | Where-Object { $_.name -eq 'mixed-joints' }
$row = $model.rows | Where-Object { $_.worlds -eq 8 -and $_.rhs_count -eq 3 -and $_.world -eq 4 }
if (-not $row) { Fail 'frozen reference row mixed-joints world=4 not found' }
$expected = @($row.factor)

$diff = 0
$maxAbs = 0.0
$details = @()
for ($i = 0; $i -lt 121; $i++) {
    $actualBit = [BitConverter]::GetBytes([single]$factor[$i])
    $expectedValue = [single][double]$expected[$i]
    $expectedBit = [BitConverter]::GetBytes($expectedValue)
    $same = $true
    for ($b = 0; $b -lt 4; $b++) { if ($actualBit[$b] -ne $expectedBit[$b]) { $same = $false; break } }
    if (-not $same) {
        $diff++
        $abs = [Math]::Abs([double]$factor[$i] - [double]$expectedValue)
        if ($abs -gt $maxAbs) { $maxAbs = $abs }
        if ($details.Count -lt 32) {
            $details += [ordered]@{
                index = $i
                actual = [double]$factor[$i]
                expected = [double]$expectedValue
                abs = $abs
            }
        }
    }
}

$result = [ordered]@{
    schema = 1
    kind = 'upstream-matrix-natural-validation'
    offline_only = $true
    product_chain_reads = $false
    status = $status
    compared = 121
    bit_differences = $diff
    max_absolute = $maxAbs
    passed = ($status -eq 0 -and $diff -eq 0)
    inputs = [ordered]@{
        generated_matrix = (Get-FileHash $matrixPath -Algorithm SHA256).Hash.ToLowerInvariant()
        provenance = (Get-FileHash $provenancePath -Algorithm SHA256).Hash.ToLowerInvariant()
        potrf_cubin = $cubinSha
        frozen_reference = 'fixtures/mjwarp-mass-solve/reference.json'
        oracle = (Get-FileHash $Oracle -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    first_differences = $details
    note = if ($diff -eq 0) { 'generated matrix reproduces the frozen upstream factor bit-exactly through the pinned MathDx potrf_11' } else { 'STOP: generated matrix does not reproduce the frozen upstream factor; do not tune inputs or verdict' }
}
$validationPath = Join-Path $out 'validation.json'
$result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $validationPath -Encoding utf8
Write-Host ("validation status=$status bit_differences=$diff max_abs=$maxAbs passed=$($result.passed)")
if (-not $result.passed) { exit 2 }
