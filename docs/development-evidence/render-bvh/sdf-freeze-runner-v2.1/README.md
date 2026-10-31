# freeze / runner extension for criterion v2.1

Task: `2026-11-01-render-sdf-freeze-runner-v21` (Arch; offline, zero GPU, no freeze executed)
Basis: signed `render-sdf-criterion-v2.1` + `sdf-outline-tool-v2.1/` + `sdf-outline-test-support/`.

## What changed (scripts only)

`scripts/validate/render-bvh/freeze-guards-sdf.ps1`
- new parameters `-CriterionVersion 'render-sdf-criterion-v2.1'`, `-BoundUlp 128`, `-OutlineDir`
  (default `target/render-bvh/sdf-validate-window/attemptN/outline`).
- requires and validates `outline-lists.json` before freezing: criterion version, signed bound,
  two arms (`recipe`/`aa2`), per-arm `exact_max_ulp <= 128`, file bytes/sha256, data-row count
  and first pixel must match the machine summary, the list sha256 must equal the signed prototype
  (`cfd1038a..` recipe, `14893fc6..` aa2), and the `outline-bounds-<arm>.txt` sidecar bytes/sha256
  must match (any mismatch throws, no guards written).
- frozen entry count becomes **9 exes + 83 inputs + 2 outline lists = 94**; the aggregate is
  sha256 over all 94 sha256 hex strings in repo-relative path order.
- guards object gains `criterion_version`, `bound_ulp`, `outline_lists[]`
  (`arm`, `entities`, `path`, `bytes`, `sha256`, `rows`, `first_pixel`, `exact_max_ulp`,
  `reference`, `reference_sha256`, `bounds_sidecar{path,bytes,sha256}`) and
  `counts.outline_lists` / `counts.total = 94`; the list `sha256` values are the prototype ones
  (`cfd1038a..` / 10,099 B / 183 rows and `14893fc6..` / 9,509 B / 172 rows).

`scripts/validate/render-bvh/run-window.ps1`
- new helpers `Outline-Info` (rows + first pixel from a list file) and `Verify-OutlineLists`
  (path/bytes/sha256/rows/first_pixel + sidecar bytes/sha256 + `exact_max_ulp <= bound_ulp`;
  mismatch logs `OUTLINE_MISMATCH[...]`, `OUTLINE_SIDECAR_MISMATCH[...]`, over-bound logs
  `OUTLINE_EXACT_MAX_OVER_B[...]`).
- `-GuardsOnly` now checks the outline lists too: `OUTLINE_LISTS_OK=2/2`,
  `SHA_OK/OUTLINE_OK`, total `GUARDS_OK=94/94` for the real attempt4 guards file
  (the dry run below uses a synthetic 2-entry guards file so only the new path is exercised).
- before the body: re-verifies the two lists (`pre-body`), any drift -> `OUTLINE_LIST_DRIFT
  exit9`; exports `MJWARP_RENDER_OUTLINE_LIST_DIR` (absolute directory) and
  `MJWARP_REQUIRE_OUTLINE_CRITERION=1` for the entity commands and records both in `facts.txt`.
- after the body: re-verifies the two lists (`post-window`), any drift -> `OUTLINE_LIST_DRIFT
  exit9`; records `outline_lists_post=2/2` and clears the two env vars.
- when the guards file has no `outline_lists` (older R05/R08 suites), all new behaviour is
  skipped: the runner stays compatible with the historical guards files.

## Dry run (offline, zero GPU, synthetic guards only - NOT a freeze)

`logs/guards-only-dryrun.txt` — synthetic guards with 0 exes + 0 inputs + the two real list
entries from `target/render-bvh/sdf-validate-window/attempt4/outline-v21-selftest/`:

```
OUTLINE_LISTS_OK=2/2
GUARDS_OK=2/2
GUARDS_ONLY_DONE
EXIT=0
```

`logs/guards-only-dryrun-tampered.txt` — same file with the recipe list sha256 replaced by zeros
(full capture: runner stdout + the appended `guard.log`, not a summary):

```
OUTLINE_LISTS_OK=1/2
GUARDS_OK=1/2
INVALID_ROUND exit9
OUTLINE_MISMATCH[guards] arm=recipe expect=0000../10099/183/28969 actual=cfd1038a../10099/183/28969
(the captured log also appends the runner guard.log verbatim)
EXIT=9
```

`logs/syntax-parse.txt` — real PowerShell parser output (`Parser::ParseFile`): both scripts
`parse OK` with token counts and `ascii_only=True` (the earlier file was a capture of the
checking command's own environment errors; it was replaced by this parser output).
Both scripts remain **ASCII-only** (verified: 0 non-ASCII bytes).

## Not done in this slice

- **No freeze was executed**: `freeze-guards-sdf.ps1` was not run, no guards file was produced
  for attempt4, `guards-sdf-validate.json` was not overwritten. This is script readiness only.
- No GPU, no window, no body; the real `-GuardsOnly` 94/94 run happens inside the signed window.
- The script identities are recorded in `receipt.json`; the attempt4 material v3 (slice 5) and
  the double-sign package (slice 6) follow.
