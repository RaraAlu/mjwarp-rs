# QA static checks: scope, thresholds/log-format diff, no CUDA work, no Start-Job
$ErrorActionPreference = 'Continue'
$Root = 'D:\Rust\mjwarp-rs'
Set-Location $Root
$Out = Join-Path $Root 'target\qa-guard\static-check.txt'
$lines = New-Object System.Collections.ArrayList
function Add-Line([string]$s) { [void]$lines.Add($s); Write-Output $s }

Add-Line '== scope: files changed 627b989d..c6531982 (non-evidence)'
$changed = @(git diff --name-only 627b989d..c6531982)
foreach ($f in $changed) { Add-Line "changed: $f" }
Add-Line ("scope_scripts_only=" + (@($changed | Where-Object { $_ -match '^scripts/' }).Count) + " evidence=" + (@($changed | Where-Object { $_ -match '^docs/development-evidence/' }).Count) + " normify=" + (@($changed | Where-Object { $_ -match '^\.normify/' }).Count) + " other=" + (@($changed | Where-Object { $_ -notmatch '^scripts/|^docs/development-evidence/|^\.normify/' }).Count))
Add-Line ("product_or_gates_touched=" + [bool](@($changed | Where-Object { $_ -match '^src/|^tests/|^Cargo|^gates\.toml|^fixtures/' }).Count))

Add-Line ''
Add-Line '== removed lines in guard template (must not contain thresholds/rules/log format)'
$removed = @(git diff 627b989d..c6531982 -- scripts/gpu-window-guard.ps1 | Where-Object { $_ -match '^-' -and $_ -notmatch '^---' })
foreach ($l in $removed) { Add-Line ("removed: " + $l) }
$thresholdRemoved = @($removed | Where-Object { $_ -match '10240|3072|80|consecutive|free_floor|increment_limit|util_abort|guard_sample|guard_stop|Write-GuardLine' })
Add-Line ("threshold_or_rule_lines_removed=" + $thresholdRemoved.Count)

Add-Line ''
Add-Line '== added lines mentioning thresholds (value must stay 10240 / 3072 / 80 / 3)'
$added = @(git diff 627b989d..c6531982 -- scripts/gpu-window-guard.ps1 | Where-Object { $_ -match '^\+' -and $_ -notmatch '^\+\+\+' })
foreach ($l in $added) { if ($l -match '10240|3072|>= 80|util_abort_percent|FreeFloor|IncrementLimit') { Add-Line ("added_threshold_line: " + $l) } }

Add-Line ''
Add-Line '== defaults read from current template (for the record)'
. (Join-Path $Root 'scripts\gpu-window-guard.ps1')
$st = New-GuardState -Label 'static' -LogPath (Join-Path $Root 'target\qa-guard\static-defaults.log')
Add-Line ("defaults free_floor=" + $st.free_floor_mib + " increment=" + $st.increment_limit_mib + " util=" + $st.util_abort_percent + " consecutive=" + $st.util_abort_consecutive + " total=" + $st.total_mib)

Add-Line ''
Add-Line '== CUDA context / window markers in changed scripts'
$markers = @(Select-String -Path (Join-Path $Root 'scripts\gpu-window-guard.ps1'), (Join-Path $Root 'scripts\gpu-guard-sampler-loop.ps1'), (Join-Path $Root 'scripts\gpu-window-guard-dryrun.ps1') -Pattern 'cudarc|CudaContext|CUDA_VISIBLE|nvrtc|Start-Job|--execute|nvidia-smi' -ErrorAction SilentlyContinue)
$startjob = @($markers | Where-Object { $_.Line -match 'Start-Job' })
$cuda = @($markers | Where-Object { $_.Line -match 'cudarc|CudaContext|CUDA_VISIBLE|nvrtc' })
$smi = @($markers | Where-Object { $_.Line -match 'nvidia-smi' })
Add-Line ("start_job_references=" + $startjob.Count + " cuda_api_references=" + $cuda.Count + " nvidia_smi_references=" + $smi.Count)

Add-Line ''
Add-Line '== window logs inside delivery evidence'
$ev = Join-Path $Root 'docs\development-evidence\gpu-window-guard-sampler'
$evFiles = @(Get-ChildItem -LiteralPath $ev -Recurse -File)
$windowLike = @($evFiles | Where-Object { $_.Name -match 'window|attempt' })
Add-Line ("delivery_evidence_files=" + $evFiles.Count + " window_or_attempt_named=" + $windowLike.Count)

Add-Line ''
Add-Line '== trigger references in delivery README/receipt'
$readme = [IO.File]::ReadAllText((Join-Path $ev 'README.md'))
$receipt = [IO.File]::ReadAllText((Join-Path $ev 'receipt.json'))
Add-Line ("readme_cites_stage2_window_doc_path=" + $readme.Contains('g10-p5-cuda-default-pipeline-stage2'))
Add-Line ("readme_mentions_samples_zero=" + ($readme -match 'samples=0'))
Add-Line ("readme_mentions_capture_local_override=" + $readme.Contains('capture') )
Add-Line ("receipt_trigger_entries=" + $receipt.Contains('trigger_evidence'))

Add-Line ''
Add-Line '== resolve-guardpwsh observed in current shell'
Add-Line ("shell=" + $PSVersionTable.PSVersion + " pwsh_on_path=" + [bool](Get-Command pwsh -ErrorAction SilentlyContinue) + " resolved=" + (Resolve-GuardPwsh))

$lines | Set-Content -LiteralPath $Out -Encoding utf8
Add-Line 'STATIC CHECK DONE'
