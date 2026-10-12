$ErrorActionPreference = 'Stop'
$root = 'C:\Rust\mjwarp-rs'
Set-Location $root
$work = Join-Path $root 'target/t4-g03-retest'
$calls = @(Get-Content (Join-Path $work 'calls.json') -Raw | ConvertFrom-Json)
if ($calls.Count -ne 17 -or @($calls | Where-Object { $_.status -ne 'finished' -or $_.exitCode -ne 0 }).Count) {
    throw 'Validation is incomplete or failed; inspect the ledger without rerunning tests.'
}
function Sum-Property($items, $property) {
    [long](($items | ForEach-Object { $_.$property } | Measure-Object -Sum).Sum)
}
$environment = Get-Content (Join-Path $work 'environment.json') -Raw | ConvertFrom-Json
$modes = foreach ($mode in @('debug','release')) {
    $product = $calls | Where-Object name -eq "$mode-products"
    $native = Get-Content (Join-Path $work "$mode-native-report.json") -Raw | ConvertFrom-Json
    $doc = $calls | Where-Object name -eq "$mode-doc-tests"
    $summaries = @(Get-Content (Join-Path $work "$mode-products-np03.json") -Raw | ConvertFrom-Json)
    $log = Get-Content (Join-Path $work "$mode-products.log") -Raw
    $binaries = @([regex]::Matches($log, '(?ms)^\s*Running (unittests src\\(?:lib|main)\.rs|tests\\[^\r\n ]+\.rs).*?^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; \d+ measured; (\d+) filtered out; finished in ([\d.]+)s') | ForEach-Object {
        [ordered]@{target=$_.Groups[1].Value;passed=[int]$_.Groups[2].Value;failed=[int]$_.Groups[3].Value;ignored=[int]$_.Groups[4].Value;filtered=[int]$_.Groups[5].Value;testSeconds=[double]$_.Groups[6].Value}
    })
    if ($binaries.Count -ne 34 -or (Sum-Property $binaries 'passed') -ne $product.passed) { throw 'Binary summary mismatch' }
    $ptx = @(Get-ChildItem (Join-Path $work "$mode-ptx") -Filter '*.json' | ForEach-Object {
        $v = Get-Content $_.FullName -Raw | ConvertFrom-Json
        $prefix = Join-Path $_.DirectoryName $_.BaseName
        $sourceHash = (Get-FileHash "$prefix.cu").Hash.ToLowerInvariant()
        $ptxHash = (Get-FileHash "$prefix.ptx").Hash.ToLowerInvariant()
        if ($sourceHash -ne $v.sourceSha256 -or $ptxHash -ne $v.ptxSha256) { throw 'PTX evidence hash mismatch' }
        if (-not $v.passed -and $v.entry -ne 'adapter_probe') { throw 'Unexpected rejected production PTX' }
        [ordered]@{file=$_.Name;entry=$v.entry;passed=$v.passed;sourceSha256=$sourceHash;ptxSha256=$ptxHash;fmad=$v.fmad;useFastMath=$v.useFastMath;nvrtcVersion=$v.nvrtcVersion;options=$v.options;scope=if($v.entry -in @('adapter_probe','hessian_probe','fma_probe')){'test-only'}else{'physics'}}
    })
    $batch = @([regex]::Matches($log, 'G03-batch worlds=(\d+) host_wall_ms=([\d.]+) includes_compile_and_transfer=true') | ForEach-Object {
        [ordered]@{worlds=[int]$_.Groups[1].Value;hostWallMilliseconds=[double]$_.Groups[2].Value;includesCompileAndTransfer=$true}
    })
    [ordered]@{
        mode=$mode;passed=($product.passed+$native.passed+$doc.passed);failed=($product.failed+$native.failed+$doc.failed);ignored=$product.ignored
        productPassed=$product.passed;productTargetsObserved=($binaries.Count-2);filteredStandaloneDiagnostic=$product.filtered
        nativePassed=$native.passed;nativeRestrictedPassed=$native.restrictedPassed;docPassed=$doc.passed
        nativeOnlyG01Passed=($calls|Where-Object name -eq "$mode-native-only-g01").passed
        nativeBlocks=(Sum-Property $summaries 'native_passed');secondaryBlocks=(Sum-Property $summaries 'mjwarp_passed');failedBlocks=(Sum-Property $summaries 'failed');pendingBlocks=(Sum-Property $summaries 'pending')
        comparedScalars=(Sum-Property $summaries 'compared_scalars')
        secondaryReasons=@($summaries.details.decision | Group-Object reason | ForEach-Object { @{reason=$_.Name;count=$_.Count} })
        binaries=$binaries;g03BatchColdTimings=$batch;generatedPtx=$ptx
        g01Metrics=@([regex]::Matches($log, 'G01(?:-complete states=\d+| raw) scalars=\d+ max_abs_error=[\d.eE+-]+')|ForEach-Object Value)
    }
}
$debugHash=(Get-FileHash (Join-Path $work 'debug-products-np03.json')).Hash
$releaseHash=(Get-FileHash (Join-Path $work 'release-products-np03.json')).Hash
$parallelDebug=(Get-FileHash (Join-Path $work 'debug-parallel.json')).Hash
$parallelRelease=(Get-FileHash (Join-Path $work 'release-parallel.json')).Hash
$report=[ordered]@{
    schema=1;sourceRevision=$environment.gitRevision;policy=$environment.policy;relativeSlack=$environment.relativeSlack
    fullT4Admitted=$false;remainingGates=@('NP01 full operator/type mapping','NP02 full upstream compiler/kernel evidence','NP04 matched steady-state timing and process peak memory')
    modes=$modes;np03ByteIdentical=($debugHash -eq $releaseHash);parallelDiagnosticByteIdentical=($parallelDebug -eq $parallelRelease)
    gnuPassed=(Sum-Property @($calls|Where-Object name -in @('gnu-host','gnu-doc-tests')) 'passed')
    invocationCount=$calls.Count;allCommandsSucceeded=$true
    rawLogHashes=@(Get-ChildItem $work -Filter '*.log'|ForEach-Object { @{file=$_.Name;sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant();bytes=$_.Length} })
    rawEvidenceHashes=@(Get-ChildItem $work -Filter '*.json'|Where-Object Name -ne 'report.json'|ForEach-Object { @{file=$_.Name;sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant();bytes=$_.Length} })
}
$report | ConvertTo-Json -Depth 30 | Set-Content (Join-Path $work 'report.json') -Encoding utf8
[pscustomobject]$report | Select-Object sourceRevision,gnuPassed,np03ByteIdentical,parallelDiagnosticByteIdentical,invocationCount,allCommandsSucceeded | ConvertTo-Json
$modes | ForEach-Object { [pscustomobject]$_ } | Select-Object mode,passed,failed,ignored,nativeBlocks,secondaryBlocks,failedBlocks,pendingBlocks,comparedScalars | Format-Table -AutoSize
