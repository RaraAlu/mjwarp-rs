# R05-S3a capture window signature record

dual_sign: Chief 2026-10-28 23:4x approved (via Arch); attempt=1; wall budget <=240 s

## signed objects (re-hashed at window open)
```
scripts/capture_render_hfield.py = cb1808f8c185df0e4695bb7107ec941c2e9e0aa8bb883fa1f630dc029d28a599 20314 425 0[0] / 20314 B / 425 lines / non-ASCII 0
fixtures/render-bvh/scene-hfield/scene.xml = b3b9a6b4ee4844c92edec3f217d04b7e788a69f9ab1e864fe7dac765e9c6f53b 3590 63 0[0] / 3590 B / 63 lines
fixtures/render-bvh/scene-hfield/scene-mixed.xml = bf9bb966887466d1f526605875df3f9b8c6b244787e7f95be2eccc3cd3329bdf 3180 60 0[0] / 3180 B / 60 lines
```

MJWARP_CAPTURE_AT=2026-10-29T00:00:00+08:00 (both groups)
legacy baseline: 38 files / 25320515 B / aggregate b5cb50afcc608a67ca9a7b83222029402719f2770a3510ab423df13601e50d8b
scene-shadow/capture* read-only: 16 files

## protocol summary
- hard gates 1..4 (re-hash, 3-point GPU baseline + floor, >=5 min clean monitor with 5 s sampling, re-verify at open)
- two pinned-python captures (hfield-only, hfield-primitive); A/B must be bitwise identical
- zero cargo/rustc/cargo test inside the window; any non-zero exit / A-B mismatch / legacy_reference_untouched=false -> stop and report, no re-capture
- release GPU immediately after the window
