# QA gate/check for the T4 identity live-check independent check.
# IMPORTANT: logs go to target/ (NOT the evidence dir) because the gate scope IS this
# evidence dir - writing into it during the gate run would make normify check report
# the just-run gates as stale.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repo = 'D:\Rust\mjwarp-rs'
$logs = Join-Path $repo 'target\qa-t4-check\gate-check'
New-Item -ItemType Directory -Force -Path $logs | Out-Null
& normify gate 2026-10-25-t4-identity-live-check-coverage-independent-check --repo $repo *>&1 |
    Tee-Object -FilePath (Join-Path $logs 'gate.log')
$gateExit = $LASTEXITCODE
Write-Host "GATE_EXIT=$gateExit"
& normify check 2026-10-25-t4-identity-live-check-coverage-independent-check --repo $repo *>&1 |
    Tee-Object -FilePath (Join-Path $logs 'check.log') -Append
$checkExit = $LASTEXITCODE
Write-Host "CHECK_EXIT=$checkExit"
[ordered]@{ git_head = ((& git -C $repo rev-parse HEAD) -join ''); gate_exit = $gateExit; check_exit = $checkExit } |
    ConvertTo-Json | Out-File -Encoding utf8 (Join-Path $logs 'gate-check-exit.json')
