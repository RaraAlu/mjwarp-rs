# QA post-fix gate/check on the main tree (HEAD 3b7db351), output kept for the record.
# NOTE: logs go to target/qa-fma-check/gate-check/ (NOT the evidence dir) because the
# gate scope is this evidence dir: writing into it during the gate run changes the
# scope/input hash and makes `normify check` report the just-run gates as stale.
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$repo = 'D:\Rust\mjwarp-rs'
$logs = Join-Path $repo 'target\qa-fma-check\gate-check'
New-Item -ItemType Directory -Force -Path $logs | Out-Null
& normify gate 2026-10-25-exact-f32-fma-relmat-independent-check --repo $repo *>&1 |
    Tee-Object -FilePath (Join-Path $logs 'gate.log')
$gateExit = $LASTEXITCODE
Write-Host "GATE_EXIT=$gateExit"
& normify check 2026-10-25-exact-f32-fma-relmat-independent-check --repo $repo *>&1 |
    Tee-Object -FilePath (Join-Path $logs 'check.log') -Append
$checkExit = $LASTEXITCODE
Write-Host "CHECK_EXIT=$checkExit"
[ordered]@{ git_head = ((& git -C $repo rev-parse HEAD) -join ''); gate_exit = $gateExit; check_exit = $checkExit } |
    ConvertTo-Json | Out-File -Encoding utf8 (Join-Path $logs 'gate-check-exit.json')
