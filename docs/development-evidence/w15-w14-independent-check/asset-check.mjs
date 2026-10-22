// W15/W14 独立复核：只读资产比对与代码事实核对（不写仓库、不跑 GPU）。
// 用法：node docs/development-evidence/w15-w14-independent-check/asset-check.mjs
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../../..');
const readText = (rel) => readFileSync(path.join(ROOT, rel), 'utf8');
const readJson = (rel) => JSON.parse(readText(rel));
const sha = (b) => createHash('sha256').update(b).digest('hex');

function walk(dir, base = dir, out = []) {
  for (const name of readdirSync(dir).sort()) {
    const p = path.join(dir, name);
    if (statSync(p).isDirectory()) walk(p, base, out);
    else if (name.endsWith('.h')) out.push(path.relative(base, p).replace(/\\/g, '/'));
  }
  return out;
}

// ---------- W15：头文件原样复制 ----------
const manifest = readJson('docs/development-evidence/w15-verbatim-headers/headers.json');
const headerRoot = path.join(ROOT, 'include/warp-headers');
const cloneRoot = path.join(ROOT, 'target/research/warp/warp/native');
const wheelRoot = path.join(ROOT, 'target/mjwarp-parallel/venv/Lib/site-packages/warp/native');

const onDisk = walk(headerRoot);
const listed = manifest.headers.map((h) => h.path).sort();
const headerRows = manifest.headers.map((h) => {
  const bytes = readFileSync(path.join(headerRoot, h.path));
  let cloneSha = null;
  let wheelSha = null;
  try { cloneSha = sha(readFileSync(path.join(cloneRoot, h.path))); } catch {}
  try { wheelSha = sha(readFileSync(path.join(wheelRoot, h.path))); } catch {}
  return {
    path: h.path,
    bytes: bytes.length,
    manifestBytes: h.bytes,
    sha256: sha(bytes),
    manifestSha256: h.sha256,
    bytesOk: bytes.length === h.bytes,
    shaOk: sha(bytes) === h.sha256,
    cloneSha256: cloneSha,
    cloneOk: cloneSha === h.sha256,
    wheelSha256: wheelSha,
    wheelDiffers: wheelSha !== null && wheelSha !== h.sha256,
  };
});
const license = readFileSync(path.join(ROOT, 'LICENSES/warp-1.15.0/LICENSE.md'));
let cloneLicenseSha = null;
try { cloneLicenseSha = sha(readFileSync(path.join(ROOT, 'target/research/warp/LICENSE.md'))); } catch {}
const notices = readText('THIRD_PARTY_NOTICES.md');
const cloneHead = (() => {
  try { return readText('target/research/warp/.git/HEAD'); } catch { return null; }
})();

const w15 = {
  manifest: {
    upstreamCommit: manifest.upstream.commit,
    tag: manifest.upstream.tag,
    headerCount: manifest.header_count,
    declaredTotalBytes: manifest.headers.reduce((n, h) => n + h.bytes, 0),
    wheelSampleCount: manifest.wheel_vs_clone.files.length,
  },
  actual: {
    headerCount: onDisk.length,
    totalBytes: onDisk.reduce((n, p) => n + statSync(path.join(headerRoot, p)).size, 0),
    setEqualsManifest: JSON.stringify(listed) === JSON.stringify(onDisk),
    missing: listed.filter((p) => !onDisk.includes(p)),
    extra: onDisk.filter((p) => !listed.includes(p)),
    allBytesOk: headerRows.every((r) => r.bytesOk),
    allShaOk: headerRows.every((r) => r.shaOk),
  },
  clone: {
    available: statSync(cloneRoot).isDirectory(),
    gitHeadFile: cloneHead,
    matchesAll: headerRows.every((r) => r.cloneOk),
    mismatch: headerRows.filter((r) => !r.cloneOk).map((r) => r.path),
  },
  wheel: {
    available: statSync(wheelRoot).isDirectory(),
    differsCount: headerRows.filter((r) => r.wheelDiffers).length,
    sampleClaims: manifest.wheel_vs_clone.files.map((f) => {
      const row = headerRows.find((r) => r.path === f.path);
      return {
        path: f.path,
        claimedWheelSha256: f.wheel_sha256,
        actualWheelSha256: row?.wheelSha256 ?? null,
        claimMatches: row?.wheelSha256 === f.wheel_sha256,
      };
    }),
    mixInCopiedTree: headerRows.filter((r) => r.cloneOk === false).map((r) => r.path),
  },
  license: {
    repoSha256: sha(license),
    repoBytes: license.length,
    cloneSha256: cloneLicenseSha,
    equalsClone: cloneLicenseSha === sha(license),
    containsApache: license.toString('utf8').includes('Apache License'),
  },
  notices: {
    hasUpstreamCommit: notices.includes(manifest.upstream.commit),
    hasTag: notices.includes(manifest.upstream.tag),
    hasHeaderPath: notices.includes('include/warp-headers'),
    hasHeaderCount: notices.includes('36'),
    hasApache: notices.includes('Apache License 2.0'),
    hasHeadersJson: notices.includes('w15-verbatim-headers/headers.json'),
  },
  sizeExcludeConfigured: readText('.normify/config.toml').includes('include/warp-headers'),
  headerRows,
};

// ---------- W14：tile 共享内存与发射约束 ----------
const abi = readText('src/runtime/launcher/abi.rs');
const launch = readText('src/runtime/launcher/launch.rs');
const nvrtc = readText('src/runtime/launcher/artifact/nvrtc.rs');
const tileTest = readText('tests/launcher-tile.rs');
const headerTest = readText('tests/launcher-warp-headers.rs');
const has = (text, needle) => text.includes(needle);

// 产品路径 shared_bytes 必须为 0（解析 src 下 `LaunchDims::new(` 的第三个顶层实参）
function splitTopLevel(text) {
  const out = [];
  let depth = 0;
  let current = '';
  for (const ch of text) {
    if (ch === '(') { depth += 1; current += ch; }
    else if (ch === ')') { depth -= 1; current += ch; }
    else if (ch === ',' && depth === 0) { out.push(current.trim()); current = ''; }
    else { current += ch; }
  }
  out.push(current.trim());
  return out;
}
function scanLaunchDims(text) {
  const nonZero = [];
  let calls = 0;
  const marker = 'LaunchDims::new(';
  let at = 0;
  while ((at = text.indexOf(marker, at)) !== -1) {
    let i = at + marker.length;
    let depth = 1;
    const start = i;
    while (i < text.length && depth > 0) {
      if (text[i] === '(') depth += 1;
      else if (text[i] === ')') depth -= 1;
      i += 1;
    }
    if (depth !== 0) break;
    calls += 1;
    const args = splitTopLevel(text.slice(start, i - 1));
    const third = args.length === 3 ? args[2].replace(/_/g, '') : `<${args.length} args>`;
    if (third !== '0') nonZero.push(text.slice(at, i));
    at = i;
  }
  return { calls, nonZero };
}
const rustFiles = [];
(function walkRs(dir) {
  for (const name of readdirSync(dir)) {
    const p = path.join(dir, name);
    if (statSync(p).isDirectory()) walkRs(p);
    else if (name.endsWith('.rs')) rustFiles.push(p);
  }
})(path.join(ROOT, 'src'));
const launchDimsScan = rustFiles.map((p) => scanLaunchDims(readFileSync(p, 'utf8')));
const launchDimsCalls = launchDimsScan.reduce((n, r) => n + r.calls, 0);
const launchDimsNonZero = launchDimsScan.flatMap((r) => r.nonZero);

const w14 = {
  constraintsSource: 'docs/development-evidence/layers-w14.md',
  abi: {
    hasSharedBytesConst: has(abi, 'const SHARED_BYTES: u32'),
    hasSharedBytesGetter: has(abi, 'pub fn shared_bytes'),
    sharedBytesFourByteAligned: has(abi, 'shared_bytes.is_multiple_of(4)'),
    hasTileRule: has(abi, 'Tile(u32)'),
    tileThreadsMultipleOf32: has(abi, '!threads.is_multiple_of(32)'),
    tileThreadsAtMost1024: has(abi, 'threads > 1024'),
  },
  launch: {
    defaultLimit48KiB: has(launch, 'per_block: 48 * 1024'),
    dynamicOptInFlag: has(launch, 'dynamic_opt_in'),
    rejectsNonOptInDynamic: has(launch, '动态共享内存未opt-in'),
    rejectsOverLimit: has(launch, '共享内存超过设备上限'),
    staticLimitCheck: has(launch, '静态共享内存超过设备上限'),
    staticDynamicExclusive: has(launch, '静态与动态共享内存不能同时声明'),
    blockMustEqualRule: has(launch, 'if self.block != (expected, 1, 1)'),
    planWithLimits: has(launch, 'pub fn plan_with_limits'),
    deviceAttributeQueryPresent: false, // 由下方全仓扫描补充
  },
  nvrtc: {
    tempHeaderDirUnique: has(nvrtc, 'HEADER_DIR_ID.fetch_add'),
    cleanupAfterCompile: has(nvrtc, 'remove_headers_dir(&directory)'),
    deviceDefaultExecutionSpace: has(nvrtc, '--device-as-default-execution-space'),
    headerCompileNoLongerRejected: !has(nvrtc, '内存头文件编译待W15'),
  },
  tests: {
    tile: {
      names: [...tileTest.matchAll(/fn (tile_block_rule_must_equal_the_compiled_thread_count|launch_dims_reject_dynamic_shared_memory_without_opt_in_and_over_limits|static_shared_memory_is_checked_against_device_limits|nvrtc_compiles_static_shared_tile_kernel_with_verbatim_warp_headers)/g)].map((m) => m[1]),
      coversIllegalTileRules: ['Tile(0)', 'Tile(48)', 'Tile(2048)'].every((s) => has(tileTest, s)),
      coversBlockMismatch: has(tileTest, 'InvalidArray("线程块违反ABI规则")'),
      coversNoOptIn: has(tileTest, '动态共享内存未opt-in'),
      coversOverDefaultAndOptIn: has(tileTest, '64 * 1024') && has(tileTest, '160 * 1024'),
      coversStaticOverLimit: has(tileTest, '静态共享内存超过设备上限'),
      coversBadSharedBytes: has(tileTest, 'InvalidSignature'),
      nvrtcTestIgnored: has(tileTest, 'ignore = "requires NVRTC'),
      noGpuLaunchCalls: !/CudaStream|DevicePtr|synchronize\(|\.launch\(/.test(tileTest),
      ptxOnlyAssertion: has(tileTest, 'artifact.ptx().contains(".shared")'),
    },
    headers: {
      manifestTest: has(headerTest, 'copied_headers_match_the_frozen_manifest'),
      licenseShaPinned: has(headerTest, '6d1d968fb225eca367cb7f0b8831ab012a35d92b547e945e17ef8e7b05c3e5cc'),
      upstreamCommitPinned: has(headerTest, 'ffa038b1adf3927a8e893d4536f1a7562f17d749'),
      noticesCheckedInTest: has(headerTest, 'THIRD_PARTY_NOTICES.md'),
      nvrtcTestIgnored: has(headerTest, 'ignore = "requires NVRTC'),
      registersExternalHeaderDigest: has(headerTest, 'with_external_header(&fp16)'),
      noGpuLaunchCalls: !/CudaStream|DevicePtr|synchronize\(|\.launch\(/.test(headerTest),
      cacheHitAndKeyChangeAsserted: has(headerTest, 'second compile must hit the cache') && has(headerTest, 'header change must recompile'),
    },
  },
  productLaunchDimsCalls: launchDimsCalls,
  productLaunchDimsAllZero: launchDimsNonZero.length === 0,
  productLaunchDimsNonZero: launchDimsNonZero,
};
const allSrc = rustFiles.map((p) => readFileSync(p, 'utf8')).join('\n');
w14.launch.deviceAttributeQueryPresent = /cudaDeviceGetAttribute|MaxSharedMemoryPerBlock|SharedMemoryPerBlockOptin/.test(allSrc);

const result = {
  schema: 1,
  task: '2026-10-23-w15-w14-independent-check',
  checkedCommits: { w15: 'b1de2db', w14: 'f98dc34' },
  generatedBy: 'asset-check.mjs（只读）',
  w15,
  w14,
  findings: [
    {
      id: 'W14-F1',
      severity: '一般',
      title: '共享内存上限未接线设备属性查询，README「按设备属性核对」表述偏强',
      detail: '冻结约束（layers-w14.md）要求「产品按设备属性核对（默认每块上限与 opt-in 上限）」；实现提供 SharedMemoryLimits::new(per_block, opt_in) 由调用方注入，但全仓无 cudaDeviceGetAttribute/MaxSharedMemoryPerBlock 查询，产品 plan() 使用固定保守默认 48KiB。当前产品内核 shared_bytes=0，无现网影响；tile 内核上线前需接线设备查询，否则 48KiB 以上合法静态共享或 opt-in 上限可能与实际设备不一致。',
      evidence: ['src/runtime/launcher/launch.rs:11-36', 'src/runtime/launcher/launch.rs:175', 'docs/development-evidence/layers-w14.md（tile ABI 冻结·共享内存上限）', 'docs/development-evidence/w14-tile-launcher/README.md（约束实现点表）'],
    },
    {
      id: 'REPRO-F1',
      severity: '建议',
      title: 'NVRTC 测试复现命令缺 NVRTC DLL 搜索路径',
      detail: 'W14/W15 README 的 ignored 测试命令只设 MJWARP_CUDA_INCLUDE。实测仅设该变量时测试报 LauncherError::Compiler("NVRTC不可用")；把 target/toolchains/cuda-12.8.1/bin 加入 PATH 后两项 ignored 测试均通过。属环境前置登记缺失，不是实现缺陷。',
      evidence: ['docs/development-evidence/w14-tile-launcher/README.md（验证表）', 'docs/development-evidence/w15-verbatim-headers/README.md（验证表）', '本目录 test-runs.json（env 记录）'],
    },
    {
      id: 'W15-F1',
      severity: '建议',
      title: 'wheel_vs_clone 仅列 7 个样本，实际 36 个文件全部与 wheel 不同',
      detail: 'headers.json.wheel_vs_clone 列出 7 个代表文件的 wheel/clone 差异；独立比对 wheel 安装目录与仓内 36 个文件，36/36 与 wheel 不同、36/36 与 clone 一致。结论（未混用 wheel）被加强而非推翻；建议在清单中说明样本性质或补全。',
      evidence: ['docs/development-evidence/w15-verbatim-headers/headers.json（wheel_vs_clone）', '本目录 asset-check.json（w15.wheel）'],
    },
    {
      id: 'W15-F2',
      severity: '建议',
      title: 'SDK 头摘要进键仅在 W15 probe 测试登记，W14 tile 编译测试未登记',
      detail: 'W15 的 NVRTC probe 测试调用 with_external_header(cuda_fp16.h) 使 SDK 头内容进入 ArtifactKey；W14 tile 编译测试只传 with_include_path(include 目录)，键中只有路径字符串，未含 cuda_fp16.h 内容摘要。当前产品路径未使用 Warp 头树，无现网影响；后续产品走上游源码路线时需在调用点登记 SDK 头摘要，THIRD_PARTY_NOTICES 的表述才完全成立。',
      evidence: ['tests/launcher-warp-headers.rs:117-126', 'tests/launcher-tile.rs:168-176', 'THIRD_PARTY_NOTICES.md（Warp 头文件原样复制节）'],
    },
  ],
  observations: [
    '36 个头文件 bytes/SHA-256 与 manifest 全等，集合无多/少，总字节 1,520,709 与声明一致。',
    '36/36 与 clone tag ffa038b1 的 warp/native 文件逐字节一致；36/36 与 wheel 不同：来源唯一性成立。',
    'LICENSE.md 与 clone LICENSE.md 逐字节一致（sha256 6d1d968f…），THIRD_PARTY_NOTICES 来源/许可声明齐全。',
    'BlockRule::Tile 校验 32 倍数且 ≤1024；validate_rule 要求 block 严格等于规则线程数；MathDx potrf 32 与 Warp 默认 256 均可表达。',
    '静态共享内存 4 字节对齐、静态/动态互斥、非 opt-in 拒绝、默认 48KiB 上限均有实现与对应测试。',
    'W14/W15 的 NVRTC 测试均为 compile-only（无 CudaStream/DevicePtr/launch 调用），PTX 断言分别检查入口名与 .shared。',
    '产品路径 LaunchDims::new 第三个实参全部为 0（全 src 扫描 0 处非零）。',
    'W14 提交未触碰 src/physics；产品内核未切换，与声明一致。',
    'W14 未做 GPU 内核执行/失败路径半边 tile 验证，已在 w14 README 边界如实声明，不登记为差异。',
  ],
};
writeFileSync(path.join(HERE, 'asset-check.json'), JSON.stringify(result, null, 2), 'utf8');
console.log(JSON.stringify({ w15: { allShaOk: w15.actual.allShaOk, setEquals: w15.actual.setEqualsManifest, cloneMatches: w15.clone.matchesAll, wheelDiffers: w15.wheel.differsCount, licenseEqualsClone: w15.license.equalsClone }, w14: { productAllZero: w14.productLaunchDimsAllZero, deviceQuery: w14.launch.deviceAttributeQueryPresent, tileTests: w14.tests.tile.names.length }, findings: result.findings.length }, null, 2));
