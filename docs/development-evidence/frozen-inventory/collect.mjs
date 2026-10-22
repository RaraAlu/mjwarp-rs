// 2026-10-23-frozen-inventory：生成冻结清单与回执目录（证据生成脚本，只读仓库）。
// 用法：node collect.mjs（在仓库根目录运行；输出 inventory.json 与 receipts.json 到当前目录）。
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../../..');
const OUT_DIR = HERE;
const readText = (rel) => readFileSync(path.join(ROOT, rel), 'utf8');
const readJson = (rel) => JSON.parse(readText(rel));
const sha = (s) => createHash('sha256').update(s, 'utf8').digest('hex');
const norm = (s) => s.replace(/\r\n/g, '\n');
const fileSha = (rel) => sha(readText(rel));

const frozen = readJson('fixtures/precision/t4-kernels.json');
const mid = readJson('fixtures/precision/t4-current-kernels.json');
const np01 = readJson('docs/development-evidence/np01-type-mapping/inventory.json');
const UPSTREAM_COMMIT = '71da24d956378a87a703b6e1442b13aec0c4ac29';

// 1) 上游源身份：冻结哈希 vs pinned checkout 实测（LF 归一化）
const upstreamSources = frozen.upstream_sources.map((u) => {
  const rel = path.join('target/mjwarp-parallel/upstream/mujoco_warp', u.path);
  let actual = null;
  try { actual = sha(norm(readText(rel))); } catch { actual = null; }
  return { path: u.path, frozen: u.sha256, pinnedActual: actual, match: actual === u.sha256 };
});

// 2) 产品源身份：冻结捕获(4dc88934) vs 中基线(8a5bac00) vs 当前 HEAD
const productSources = frozen.product_sources.map((s) => {
  const head = sha(norm(readText(s.path)));
  const midSha = mid.product_sources.find((x) => x.path === s.path)?.sha256 ?? null;
  return {
    path: s.path, frozen: s.sha256, mid: midSha, head,
    sameAsFrozen: head === s.sha256, sameAsMid: head === midSha,
  };
});

// 3) 扫描仓内内核回执（docs 与 fixtures 下的 JSON；含 ptxSha256 与 entry）
function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = path.join(dir, name);
    let st; try { st = statSync(p); } catch { continue; }
    if (st.isDirectory()) walk(p, out);
    else if (name.endsWith('.json') && st.size <= 500_000) out.push(path.relative(ROOT, p).replace(/\\/g, '/'));
  }
  return out;
}
const scannedFiles = [...walk(path.join(ROOT, 'docs')), ...walk(path.join(ROOT, 'fixtures'))];
const receipts = [];
for (const rel of scannedFiles) {
  let raw, j;
  try { raw = readText(rel); j = JSON.parse(raw); } catch { continue; }
  if (!j || typeof j !== 'object' || !j.ptxSha256 || !j.entry) continue;
  receipts.push({
    path: rel, fileSha256: sha(raw), entry: j.entry, ptxSha256: j.ptxSha256,
    sourceSha256: j.sourceSha256 ?? null, options: j.options ?? null, fmad: j.fmad ?? null,
    useFastMath: j.useFastMath ?? null, nvrtcVersion: j.nvrtcVersion ?? null,
    precisionGate: j.precisionGate ?? null,
  });
}
const receiptByPtx = new Map();
for (const r of receipts) {
  if (!receiptByPtx.has(r.ptxSha256)) receiptByPtx.set(r.ptxSha256, []);
  receiptByPtx.get(r.ptxSha256).push(r);
}
const receiptByFileSha = new Map(receipts.map((r) => [r.fileSha256, r]));

// 4) T4 冻结清单 24 条目 / 30 变体
const mappingByEntry = new Map(frozen.mappings.map((m) => [m.entry, m]));
const variantGroups = new Map();
for (const a of frozen.artifacts) {
  if (!variantGroups.has(a.entry)) variantGroups.set(a.entry, []);
  variantGroups.get(a.entry).push(a);
}
const midPtx = new Set(mid.artifacts.map((a) => `${a.entry}|${a.ptx_sha256}`));
const t4Entries = [...variantGroups.entries()].map(([entry, variants]) => {
  const m = mappingByEntry.get(entry) ?? {};
  return {
    entry,
    product: m.product ?? null,
    upstreamFile: m.upstream_file ?? null,
    upstreamSymbols: m.upstream_symbols ?? [],
    fields: m.fields ?? [],
    arithmetic: m.arithmetic ?? null,
    boundary: m.boundary ?? null,
    variants: variants.map((a) => {
      const byFile = receiptByFileSha.get(a.receipt_sha256) ?? null;
      const byPtx = receiptByPtx.get(a.ptx_sha256) ?? [];
      const found = byFile ?? byPtx[0] ?? null;
      return {
        sourceSha256: a.source_sha256, ptxSha256: a.ptx_sha256, receiptSha256: a.receipt_sha256,
        recapturedAtMidRevision: !midPtx.has(`${entry}|${a.ptx_sha256}`),
        receiptInRepo: byPtx.length > 0,
        receiptIdentityMatch: byFile !== null,
        receiptPath: found?.path ?? null,
        receiptOptions: found?.options ?? null,
        receiptFmad: found?.fmad ?? null,
        receiptFastMath: found?.useFastMath ?? null,
        receiptNvrtcVersion: found?.nvrtcVersion ?? null,
        receiptPrecisionGate: found?.precisionGate ?? null,
      };
    }),
  };
});

// 5) NP01 30 路径 -> T4 条目 / 回执条目 / 冻结证据链（手工核对映射）
const T4 = {
  kinematics: ['rigid_kinematics'], com_position: ['rigid_com_position'], com_velocity: ['com_velocity'],
  crb: ['rigid_mass_matrix'], mass_multiply: [], mass_solve: ['rigid_mass_solve'], rne_bias: ['rne_bias'],
  rne_acc: [], gravity_compensation: [], joint_passive: [], smooth_force: [], passive_force: [],
  fixed_tendon: ['fixed_tendon'], spatial_tendon: ['spatial_site', 'spatial_moment', 'spatial_wrap'],
  tendon_wrap: [], tendon: ['tendon_values', 'tendon_indices'], tendon_velocity: [], tendon_passive: [],
  flex_position: ['flex_positions', 'flex_invalidate'], flex_edge: ['flex_edges'], flex_face: ['flex_faces'],
  flex_hessian: ['flex_hessian', 'flex_hessian_validate'], flex_edge_passive: [], fluid_force: [],
  applied_force: [], kinetic_energy: [], attached: ['attached_kinematics', 'initialize_attached'],
  camlight: ['camlight_rigid', 'camlight_com'], sleep_wake: ['sleep_copy', 'tendon_wake'], field_batch: [],
};
const RECEIPTS = {
  kinematics: ['rigid_kinematics'], com_position: ['rigid_com_position'], com_velocity: ['com_velocity'],
  crb: ['resident_crb_init', 'resident_crb_accumulate', 'resident_crb_matrix'],
  mass_multiply: ['borrowed_mass_validate', 'borrowed_mass_multiply'],
  mass_solve: ['sparse_dense_validate', 'sparse_dense_gather', 'mixed_solve_validate', 'mixed_solve_fused', 'resident_mixed_gather'],
  rne_bias: [], rne_acc: [], gravity_compensation: ['gravity_compensation_clear', 'gravity_compensation_validate', 'gravity_compensation_compute', 'gravity_compensation_sum'],
  joint_passive: ['joint_passive_validate', 'joint_passive_compute', 'joint_passive_sum'],
  smooth_force: ['smooth_force_validate', 'smooth_force'],
  passive_force: ['passive_force_clear', 'passive_force_sum', 'passive_force_clear_gravity'],
  fixed_tendon: ['fixed_tendon'], spatial_tendon: ['spatial_site', 'spatial_moment', 'spatial_wrap'],
  tendon_wrap: [], tendon: ['tendon_values', 'tendon_indices'],
  tendon_velocity: ['tendon_velocity_validate', 'tendon_velocity_compute'],
  tendon_passive: ['tendon_passive_validate', 'tendon_passive_clear', 'tendon_passive_compute'],
  flex_position: ['flex_positions', 'flex_invalidate'], flex_edge: ['flex_edges'], flex_face: [],
  flex_hessian: [], flex_edge_passive: ['flex_edge_passive_validate', 'flex_edge_passive_clear', 'flex_edge_passive_compute'],
  fluid_force: ['fluid_force_validate', 'fluid_force_body', 'fluid_force_project'],
  applied_force: ['applied_force_validate', 'applied_force_delta', 'applied_force_publish'],
  kinetic_energy: ['kinetic_energy_validate', 'kinetic_energy_reduce32', 'device_energy_source_validate'],
  attached: ['attached_kinematics', 'initialize_attached'], camlight: ['camlight_rigid', 'camlight_com'],
  sleep_wake: ['sleep_copy', 'tendon_wake'], field_batch: [],
};
const CHAINS = {
  kinematics: ['smooth-kinematics-com-cdof'], com_position: ['smooth-kinematics-com-cdof'],
  com_velocity: [], crb: [], gravity_compensation: ['passive-gravity-force'],
  attached: [], camlight: [], sleep_wake: [],
};
const np01Coverage = np01.rows.map((row) => {
  const recEntries = (RECEIPTS[row.id] ?? []).filter((e) => receipts.some((r) => r.entry === e));
  const recFiles = receipts.filter((r) => recEntries.includes(r.entry));
  return {
    id: row.id, layer: row.layer, status: row.status, paths: row.paths,
    productFiles: row.productFiles, kernels: row.kernels,
    upstream: { files: row.upstream.files, functions: row.upstream.functions, confidence: row.upstream.confidence },
    t4Entries: T4[row.id] ?? [], receiptEntries: recEntries,
    receiptCount: recFiles.length,
    receiptPaths: [...new Set(recFiles.map((r) => r.path))].slice(0, 8),
    frozenChains: CHAINS[row.id] ?? [],
  };
});

// 6) 冻结证据链（fixtures/gravity-upstream-abi 下 committed 生成内核）
// three-d 的 receipt 只引用独立生成模块（module_artifact.directory），实体在 independent-kinematics。
const chainSpecs = [
  { name: 'smooth-kinematics-com-cdof', dir: 'fixtures/gravity-upstream-abi/independent-kinematics', ptx: 'smooth.ptx' },
  { name: 'passive-gravity-force', dir: 'fixtures/gravity-upstream-abi/generated', ptx: 'module.ptx' },
  { name: 'three-d-kinematics', dir: 'fixtures/gravity-upstream-abi/three-d-kinematics', ptx: null },
];
const chains = chainSpecs.map((spec) => {
  const r = readJson(`${spec.dir}/receipt.json`);
  let moduleDir = spec.dir;
  let ptxRel = spec.ptx ? `${spec.dir}/${spec.ptx}` : null;
  if (!spec.ptx && r.module_artifact?.directory) {
    moduleDir = path.posix.normalize(`${spec.dir}/${r.module_artifact.directory}`);
    ptxRel = `${moduleDir}/smooth.ptx`;
  }
  const refReceipt = moduleDir !== spec.dir ? readJson(`${moduleDir}/receipt.json`) : null;
  const declaredPtx = r.ptx_sha256 ?? r.module_artifact?.ptx_sha256 ?? null;
  const ptxSha = ptxRel ? fileSha(ptxRel) : null;
  const receiptSha = fileSha(`${spec.dir}/receipt.json`);
  const entries = r.kernel_sequence ?? (r.entry ? [r.entry] : (refReceipt?.kernel_sequence ?? []));
  return {
    name: spec.name, directory: spec.dir, receipt: `${spec.dir}/receipt.json`, receiptSha256: receiptSha,
    moduleDirectory: moduleDir, moduleReceipt: refReceipt ? `${moduleDir}/receipt.json` : null,
    upstreamRevision: r.upstream_revision, warpVersion: r.warp_version, module: r.module,
    entries, entryCount: entries.length,
    targetArch: r.target_arch, mode: r.mode, fastMath: r.fast_math, fuseFp: r.fuse_fp,
    ptxFile: ptxRel, ptxDeclaredSha256: declaredPtx, ptxActualSha256: ptxSha,
    ptxMatch: ptxRel ? ptxSha === declaredPtx : null,
    cudaToolkit: r.cuda_toolkit ?? null, ptxCompiler: r.ptx_compiler ?? null,
    compilerProvenance: r.compiler_options_provenance ?? r.compiler_provenance ?? null,
  };
});

// 7) 统计与缺口
const flatVariants = t4Entries.flatMap((e) => e.variants);
const stats = {
  t4: {
    entries: t4Entries.length, variants: flatVariants.length,
    variantsWithReceiptInRepo: flatVariants.filter((v) => v.receiptInRepo).length,
    variantsReceiptIdentityMatch: flatVariants.filter((v) => v.receiptIdentityMatch).length,
    recapturedAtMidRevision: flatVariants.filter((v) => v.recapturedAtMidRevision).length,
  },
  np01: {
    rows: np01Coverage.length,
    rowsWithT4: np01Coverage.filter((r) => r.t4Entries.length > 0).length,
    rowsWithReceipts: np01Coverage.filter((r) => r.receiptEntries.length > 0).length,
    rowsWithFrozenChain: np01Coverage.filter((r) => r.frozenChains.length > 0).length,
    rowsWithoutAnyKernelEvidence: np01Coverage.filter((r) => !r.t4Entries.length && !r.receiptEntries.length && !r.frozenChains.length).map((r) => r.id),
    rowsMarkedPartial: np01Coverage.filter((r) => r.status !== '待补').length,
    rowsMarkedTodo: np01Coverage.filter((r) => r.status === '待补').length,
  },
  receipts: {
    scannedJsonFiles: scannedFiles.length, receipts: receipts.length,
    distinctEntries: new Set(receipts.map((r) => r.entry)).size,
    distinctPtx: new Set(receipts.map((r) => r.ptxSha256)).size,
    optionVariants: new Set(receipts.map((r) => JSON.stringify({ o: r.options, f: r.fmad, fm: r.useFastMath, n: r.nvrtcVersion, g: r.precisionGate }))).size,
  },
  upstream: { total: upstreamSources.length, matched: upstreamSources.filter((u) => u.match).length },
  productSources: {
    total: productSources.length,
    headDrift: productSources.filter((s) => !s.sameAsFrozen).length,
    midDrift: productSources.filter((s) => !s.sameAsMid).length,
  },
};
const gaps = [
  { item: '上游生成内核覆盖', status: '待补', detail: 'T4 24 条目只覆盖 smooth/passive/sleep/math/block_cholesky/types/util_misc 7 个上游源中的显式符号；上游 Warp 生成内核（warp.codegen / builtin 头文件级）无逐符号清单。', blocker: '待 W15（include/warp-headers 落地后可对照上游头文件源码路径）与上游生成内核清单冻结。' },
  { item: '编译器库覆盖', status: '待补', detail: '全部仓内回执的编译选项一致（nvrtc 12.8、fmad=true、fastMath=false、--std=c++17、--gpu-architecture=compute_70、f32-physics 门禁），但没有 Warp 编译器库（builtin/codegen）内部精度边界的逐项证据。', blocker: '待 W15/外部条件。' },
  { item: '历史捕获实体', status: '部分在仓', detail: `30 个 T4 变体中 ${flatVariants.filter((v) => v.receiptInRepo).length} 个可按 PTX 哈希在仓内定位回执、${flatVariants.filter((v) => v.receiptIdentityMatch).length} 个回执文件哈希与冻结值逐字节一致；其余需历史 target/t4-g03-retest 产物，相关 6 项 audit 已按裁决从 all-probes 排除（three-layer §5.2）。`, blocker: '历史冻结产物/重采集工具。' },
  { item: '产品源身份漂移', status: '已量化', detail: `当前 HEAD 有 ${productSources.filter((s) => !s.sameAsFrozen).length}/${productSources.length} 个产品源与冻结捕获身份（${frozen.source_revision.slice(0, 8)}）不一致；${flatVariants.filter((v) => v.recapturedAtMidRevision).length} 个 T4 变体在中基线 ${mid.source_revision.slice(0, 8)} 有重采集哈希；旧 30 份集合不能直接证明当前源码。`, blocker: '重采集/重基线需另行裁决登记。' },
  { item: 'NP01 无内核证据路径', status: '已量化', detail: `NP01 30 条路径中 ${stats.np01.rowsWithT4} 条有 T4 冻结条目、${stats.np01.rowsWithReceipts} 条能在仓内定位内核回执、${stats.np01.rowsWithFrozenChain} 条有 fixtures 冻结证据链；无任何内核证据的路径：${stats.np01.rowsWithoutAnyKernelEvidence.join('、') || '无'}。`, blocker: '按路径补证据或显式登记外部条件。' },
  { item: '排除条目', status: '不含在分母', detail: `T4 排除测试条目：${frozen.excluded_test_entries.join('、')}；排除理由沿用 fixtures/precision/t4-kernels.json 的 remaining 与 audit.json。`, blocker: '无（历史边界）。' },
];

const inventory = {
  schema: 1,
  task: '2026-10-23-frozen-inventory',
  scope: 'NP02 冻结清单：冻结提交、上游符号、生成内核与编译选项的全量盘点（只做证据，不改产品/测试/容差/fixtures）。',
  upstream: { repo: 'google-deepmind/mujoco_warp', commit: UPSTREAM_COMMIT, warpVersion: '1.15.0', pinnedCheckout: 'target/mjwarp-parallel/upstream' },
  compileOptions: {
    productRecipe: { name: 'warp-1.15-physics', source: 'src/runtime/launcher/recipe.rs', purpose: 'physics', fmad: true, fastMath: false, std: '--std=c++17', arch: '--gpu-architecture=compute_70', defines: [], precisionGate: 'f32-physics (check_physics_ptx)' },
    capturePath: { source: 'src/runtime/transfer/kernel.rs', fmad: true, useFastMath: false, options: ['--std=c++17', '--gpu-architecture=compute_70'], precisionCheck: 'check_physics_ptx' },
    uniformAcrossReceipts: stats.receipts.optionVariants === 1,
    frozenChainAuthoring: { targetArch: 89, mode: 'release', fastMath: false, fuseFp: true, cudaToolkitVersion: '12.9', ptxCompilerVersion: '12.9.86', device: 'NVIDIA GeForce RTX 4070 Ti SUPER', note: 'fixtures 冻结链按 pinned Warp 默认编译（sm_89），与产品 nvrtc compute_70 配方不同；PTX 前向兼容，产品直接加载。' },
  },
  upstreamSources,
  productSources,
  t4: { frozenCaptureRevision: frozen.source_revision, midRevision: mid.source_revision, entries: t4Entries, excludedTestEntries: frozen.excluded_test_entries, remaining: frozen.remaining },
  np01PathCoverage: np01Coverage,
  frozenEvidenceChains: chains,
  stats,
  gaps,
};
writeFileSync(path.join(OUT_DIR, 'inventory.json'), JSON.stringify(inventory, null, 2), 'utf8');
writeFileSync(path.join(OUT_DIR, 'receipts.json'), JSON.stringify({ schema: 1, note: '仓内（docs 与 fixtures）全部内核回执的编译选项与哈希目录；用于逐变体对照。', receipts }, null, 2), 'utf8');
console.log(JSON.stringify({ dir: OUT_DIR, stats }, null, 2));
