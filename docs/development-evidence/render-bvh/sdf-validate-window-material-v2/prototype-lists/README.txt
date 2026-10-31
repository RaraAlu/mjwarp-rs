v2.1 proposal prototype lists - reproduction record
================================================================================
Purpose: make the two prototype exact-difference list files of
`v2.1-proposal.md` (sections A and D) independently reproducible from the committed
fixtures. Proposal evidence only: these lists are NOT a criterion, NOT frozen guards and NOT
used by any product code path.

Generator
  docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists/generate-lists.py
  generator sha256 775d6037c89847acccf0cf839074e816360f9438e7e54ef468a27f0fbed950c3  bytes 3369

Command (offline, zero GPU; writes only the two lists + lists-sha256.txt into --out)
  target/mjwarp-parallel/venv/Scripts/python.exe -B ^
    docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists/generate-lists.py ^
    --repo . --out docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists

  (single line for bash-style shells:
   target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists/generate-lists.py --repo . --out docs/development-evidence/render-bvh/sdf-validate-window-material-v2/prototype-lists
  )

Environment used (offline authoring exception; user-approved 2026-10-22)
  interpreter: target/mjwarp-parallel/venv/Scripts/python.exe  (python 3.12.13)
  packages   : numpy 2.5.4 (mujoco 3.15.0 / warp 1.15.0 / mujoco-warp 3.15.0 are the pinned
               versions of that venv but are not imported by the generator)
  upstream   : target/mjwarp-parallel/upstream at 71da24d956378a87a703b6e1442b13aec0c4ac29
               (identity re-checked by author-scene-a-sphere-outline.ps1 for the main tool; the
               generator only imports the committed replay module and reads fixtures)
  zero GPU, no NVRTC, no kernel launch; host-side exact-f32 replay only.

Inputs (sha256; the replay module pins the same values and exits 6 on any drift)
  fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json  d6ab6dde58034df5c1bc69ab397026a8f7c3aaf58d29d25edb032d3818000dc7
  fixtures/render-bvh/scene-a/s7b/flags/aa2_depth_seg.json         79a275eff5d45f6bef764957e5f4987d0e6abfbb9b7a2cb578c8e1f717d4b7e1
  fixtures/render-bvh/scene-a/model-312.json                      cdd490f0209dd16a57b2685f3139548af8a2b73195cdb0e01a74ab094ef1df00
  fixtures/render-bvh/scene-a/s2/reference_hits.json              609ffc527e8216654cb7b6576f998659026d0e5324429c31c7a67f058c225f84
  fixtures/render-bvh/scene-a/s5/camera_rays.json                 545a1442fc5ac67d8de9abd6d7f7cf0a24e5a86baa3b473f660d8aae83842022
  scripts/authoring/scene_a_outline_replay.py                     (committed replay module used by the generator)

Outputs (exact difference sets of the two `a = dot(vec,vec)` association forms, camera 1)
  scene-a-sphere-outline-recipe.txt  183 rows  10,099 bytes
    sha256 cfd1038a1cc50326de51a25495db68cd8b25c0983121dea88a2fb89ca83724ee
  scene-a-sphere-outline-aa2.txt     172 rows   9,509 bytes
    sha256 14893fc6a2479e8e8f5e9cc07cabe72fd9f37aea638dda25fc26839d59ec8eab
  lists-sha256.txt                   sha256 manifest of the two lists (see file)
  Format: header lines (#) then one row per listed pixel, sorted by pixel index:
    pixel x y ref_seg_geom ref_seg_type ref_depth_bits formA_bits formB_bits ulp_ab

Self-check against attempt3 (the reason these numbers are trusted)
  arm                       list px  exact max |delta|  first differing pixel  attempt3 record
  recipe (spp=1)            183      73 ULP             28969 = (169,120)      183 px / 73 ULP, first 28969
  flags aa2 arm (spp=2)     172      91 ULP             29448 = (168,122)      172 px / 91 ULP, first 29448
  camera 0                  0        -                  -                      cam0 = 0
  form A also reproduces the frozen reference depth bits bitwise on every camera-1 sphere
  pixel (747/747 recipe, 745/745 aa2); the camera-space ray port matches the S5 fixture
  256/256 bitwise. See dump-tool-selfcheck.txt for the same numbers produced through the
  authoring wrapper.

Boundary
  These lists exist only for the v2.1 proposal. They are not guards, not a criterion, and no
  product build/test/gate/script references them. The freeze step of a future attempt4 runs
  the authoring tool (step 3 after the user signs) and puts the list sha256 values into the
  guards file; rewriting a list inside the window is a failure.
