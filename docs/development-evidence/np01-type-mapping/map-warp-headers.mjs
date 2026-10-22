// 2026-10-23-precision-source-paths：为 NP01 清单补「产品内核 → W15 Warp 头文件实现」证据。
// 只读：读 NP01 inventory、pinned 上游 Python、产品 CUDA 源与 include/warp-headers；写 warp-headers.json 并更新 inventory.json。
// 用法：node docs/development-evidence/np01-type-mapping/map-warp-headers.mjs
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '../../..');
const readText = (rel) => readFileSync(path.join(ROOT, rel), 'utf8');
const readJson = (rel) => JSON.parse(readText(rel));
const headerRoot = path.join(ROOT, 'include/warp-headers');
const upstreamRoot = path.join(ROOT, 'target/mjwarp-parallel/upstream/mujoco_warp/_src');
const manifest = readJson('docs/development-evidence/w15-verbatim-headers/headers.json');

// ---------- Warp 头文件符号索引 ----------
function walkHeaders(dir, base = dir, out = []) {
  for (const name of readdirSync(dir).sort()) {
    const p = path.join(dir, name);
    if (statSync(p).isDirectory()) walkHeaders(p, base, out);
    else if (name.endsWith('.h')) out.push(path.relative(base, p).replace(/\\/g, '/'));
  }
  return out;
}
const headerFiles = walkHeaders(headerRoot);
const headerText = new Map(headerFiles.map((h) => [h, readFileSync(path.join(headerRoot, h), 'utf8')]));

// 定义优先顺序：先物理/数学原语头，再运行时/工具头；避免把使用点当定义。
const HEADER_PREF = ['vec.h', 'mat.h', 'quat.h', 'spatial.h', 'builtin.h', 'array.h', 'crt.h', 'cuda_crt.h',
  'deterministic.h', 'tile_cholesky.h', 'tile_solve.h', 'tile_reduce.h', 'tile.h', 'nanovdb/PNanoVDB.h'];

function locateSymbol(name) {
  const esc = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const order = [...HEADER_PREF, ...headerFiles.filter((h) => !HEADER_PREF.includes(h))];
  const callRe = new RegExp(`\\b${esc}\\s*\\(`);
  for (const header of order) {
    const lines = headerText.get(header).split('\n');
    for (let i = 0; i < lines.length; i += 1) {
      const line = lines[i];
      if (!callRe.test(line)) continue;
      if (!/(inline|CUDA_CALLABLE|using|struct|class|typedef)/.test(line)) continue;
      // 定义行：NAME 之前不能已经出现 '('（排除函数体内调用点）。
      const before = line.slice(0, line.search(callRe));
      if (before.includes('(')) continue;
      if (/^\s*(return|if|else|while|for)\b/.test(line)) continue;
      return { header, line: i + 1 };
    }
  }
  const aliasRes = [
    new RegExp(`^\\s*using\\s+${esc}\\s*=`),
    new RegExp(`^\\s*(?:struct|class)\\s+${esc}\\b`),
    new RegExp(`^\\s*typedef[^;]*\\b${esc}\\s*;`),
  ];
  for (const header of order) {
    const lines = headerText.get(header).split('\n');
    for (let i = 0; i < lines.length; i += 1) {
      if (aliasRes.some((re) => re.test(lines[i]))) return { header, line: i + 1 };
    }
  }
  return null;
}

// ---------- 上游 wp.* API 索引 ----------
const RUNTIME_ONLY = new Set(['array', 'array2d', 'array3d', 'empty', 'zeros', 'ones', 'kernel', 'launch', 'launch_tiled',
  'func', 'func_native', 'static', 'tid', 'get_device', 'set_module_options', 'printf', 'where', 'copy', 'sync',
  'block_dim', 'int', 'float', 'vec2i', 'vec3i', 'vec4i', 'vec2f', 'vec3f', 'vec4f', 'vec2h', 'vec3h', 'vec4h',
  'int8', 'int16', 'int32', 'int64', 'uint8', 'uint16', 'uint32', 'uint64', 'bool', 'float16', 'float32', 'float64']);

function functionBodies(text, names) {
  const lines = text.split('\n');
  const bodies = new Map();
  for (const name of names) {
    const start = lines.findIndex((l) => new RegExp(`^def\\s+${name}\\s*\\(`).test(l));
    if (start < 0) continue;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
      if (/^(def|class)\s/.test(lines[i])) { end = i; break; }
    }
    bodies.set(name, lines.slice(start, end).join('\n'));
  }
  return bodies;
}

function wpCalls(text) {
  const calls = new Set();
  for (const m of text.matchAll(/\bwp\.([a-z][a-z0-9_]*)/g)) calls.add(m[1]);
  return calls;
}

const PRODUCT_PRIMITIVE = /\b(quat_[a-z0-9_]+|mat[234][a-z0-9_]*|vec[234][a-z0-9_]*|spatial_[a-z0-9_]+|tile_[a-z0-9_]+|atomicAdd|atomic_add|atomic_min|atomic_max|atomic_sub|__fadd_rn|__fmul_rn|__fmaf_rn|fmaf|sqrtf|rsqrtf|atan2f|asinf|acosf|sinf|cosf|tanf|expf|powf|fminf|fmaxf|fabsf|isfinite|isnan|normalize|cross|dot|__syncthreads)\b/g;
const NORMALIZE = new Map([
  ['sqrtf', 'sqrt'], ['rsqrtf', 'rsqrt'], ['atan2f', 'atan2'], ['asinf', 'asin'], ['acosf', 'acos'],
  ['sinf', 'sin'], ['cosf', 'cos'], ['tanf', 'tan'], ['expf', 'exp'], ['powf', 'pow'],
  ['fminf', 'min'], ['fmaxf', 'max'], ['fabsf', 'abs'],
  ['atomicAdd', 'atomic_add'],
]);
// 非 Warp 头树实现的通用词/运行时 API：不参与映射。
const NON_HEADER = new Set(['types', 'full', 'pi', 'where', 'copy', 'sync']);

function productFiles(row) {
  const out = new Set();
  for (const raw of row.productFiles ?? []) {
    let rel = raw.replace(/\\/g, '/');
    if (rel.endsWith('/**')) rel = rel.slice(0, -3);
    const abs = path.join(ROOT, rel);
    let st;
    try { st = statSync(abs); } catch { continue; }
    if (st.isFile()) { out.add(rel); continue; }
    const stack = [rel];
    while (stack.length) {
      const dir = stack.pop();
      for (const name of readdirSync(path.join(ROOT, dir))) {
        const child = `${dir}/${name}`;
        const cs = statSync(path.join(ROOT, child));
        if (cs.isDirectory()) stack.push(child);
        else if (/\.(rs|cu)$/.test(name)) out.add(child);
      }
    }
  }
  return [...out].sort();
}

// ---------- 逐行取证 ----------
const inventory = readJson('docs/development-evidence/np01-type-mapping/inventory.json');
const rows = inventory.rows.map((row) => {
  const upstreamDetected = new Map(); // name -> sources[]
  const upstreamFiles = row.upstream?.files ?? [];
  const functionNames = (row.upstream?.functions ?? []).filter((f) => /^[A-Za-z_][A-Za-z0-9_]*$/.test(f));
  for (const file of upstreamFiles) {
    const rel = `target/mjwarp-parallel/upstream/mujoco_warp/_src/${file}`;
    let text;
    try { text = readText(rel); } catch { continue; }
    const bodies = functionBodies(text, functionNames);
    if (bodies.size > 0) {
      for (const [fn, body] of bodies) {
        for (const call of wpCalls(body)) {
          if (!upstreamDetected.has(call)) upstreamDetected.set(call, []);
          upstreamDetected.get(call).push(`_src/${file}#${fn}`);
        }
      }
    } else {
      for (const call of wpCalls(text)) {
        if (!upstreamDetected.has(call)) upstreamDetected.set(call, []);
        upstreamDetected.get(call).push(`_src/${file}（文件级）`);
      }
    }
  }

  const productDetected = new Map();
  for (const file of productFiles(row)) {
    const text = readText(file);
    for (const m of text.matchAll(PRODUCT_PRIMITIVE)) {
      const name = NORMALIZE.get(m[1]) ?? m[1];
      if (!productDetected.has(name)) productDetected.set(name, new Set());
      productDetected.get(name).add(file);
    }
  }

  const perHeader = new Map();
  const consider = (name, via, sources) => {
    if (RUNTIME_ONLY.has(name) || NON_HEADER.has(name)) return false;
    const hit = locateSymbol(name);
    if (!hit) return false;
    const key = hit.header;
    if (!perHeader.has(key)) perHeader.set(key, { header: hit.header, path: `include/warp-headers/${hit.header}`, symbols: new Map(), via: new Set(), sources: new Set() });
    const entry = perHeader.get(key);
    entry.symbols.set(name, hit.line);
    entry.via.add(via);
    for (const s of sources) entry.sources.add(s);
    return true;
  };
  let upstreamMapped = 0;
  for (const [name, sources] of upstreamDetected) if (consider(name, 'upstream', sources)) upstreamMapped += 1;
  let productMapped = 0;
  for (const [name, sources] of productDetected) if (consider(name, 'product', [...sources])) productMapped += 1;

  const warpHeaders = [...perHeader.values()]
    .sort((a, b) => a.header.localeCompare(b.header))
    .map((e) => ({
      path: e.path,
      symbols: [...e.symbols.entries()].sort((a, b) => a[0].localeCompare(b[0])).map(([name, line]) => ({ name, line })),
      via: [...e.via].sort(),
      sources: [...e.sources].sort().slice(0, 6),
    }));

  const upstreamSources = [...upstreamDetected.values()].flat();
  const upstreamLevel = {
    functionLevel: upstreamSources.filter((s) => s.includes('#')).length,
    fileLevel: upstreamSources.filter((s) => s.includes('（文件级）')).length,
  };

  return {
    row,
    warpHeaders,
    upstreamDetected,
    productDetected,
    coverage: {
      upstreamApisDetected: upstreamDetected.size,
      upstreamApisMapped: upstreamMapped,
      productPrimitivesDetected: productDetected.size,
      productPrimitivesMapped: productMapped,
      headers: warpHeaders.length,
      upstreamLevel,
    },
    unmappedUpstream: [...upstreamDetected.keys()].filter((n) => !RUNTIME_ONLY.has(n) && !locateSymbol(n)).sort(),
  };
});

// ---------- 写 warp-headers.json ----------
const headerSource = {
  headerRoot: 'include/warp-headers',
  headerCount: headerFiles.length,
  warpVersion: manifest.upstream.tag,
  cloneCommit: manifest.upstream.commit,
  manifest: 'docs/development-evidence/w15-verbatim-headers/headers.json',
  note: '产品内核为手写 NVRTC CUDA 源；本映射把产品 CUDA 原语与 pinned 上游 Python 的 wp.* 调用对到 W15 原样复制的 Warp 头文件符号，作为上游实现路径证据，不改变产品源码。',
};
const warpHeaders = {
  schema: 1,
  task: '2026-10-23-precision-source-paths',
  headerSource,
  rows: rows.map((r) => ({
    id: r.row.id,
    status: r.row.status,
    upstream: r.row.upstream,
    detected: {
      upstreamApis: [...r.upstreamDetected.keys()].sort(),
      productPrimitives: [...r.productDetected.keys()].sort(),
    },
    warpHeaders: r.warpHeaders,
    coverage: r.coverage,
    unmappedUpstream: r.unmappedUpstream,
  })),
};
writeFileSync(path.join(HERE, 'warp-headers.json'), JSON.stringify(warpHeaders, null, 2), 'utf8');

// ---------- 更新 inventory.json ----------
const statusBefore = {};
let filled = 0;
let stillTodo = 0;
for (const r of rows) {
  const row = r.row;
  statusBefore[row.status] = (statusBefore[row.status] ?? 0) + 1;
  const hasEvidence = r.coverage.headers > 0;
  if (hasEvidence) {
    row.warpHeaders = r.warpHeaders;
    row.warpHeaderCoverage = r.coverage;
    const summary = `W15 上游头文件实现路径：${r.warpHeaders.map((h) => h.path).join('、')}（符号与行号见 warp-headers.json）`;
    row.evidence.existing = [...new Set([...(row.evidence.existing ?? []), summary])];
    row.evidence.missing = (row.evidence.missing ?? []).filter((m) => !/上游源码复制路径|W15 落地后/.test(m));
    if (row.status.startsWith('待补')) {
      row.status = r.coverage.upstreamLevel.functionLevel > 0
        ? '部分（W15 头文件路径已对；缺逐字段表）'
        : '部分（W15 头文件路径已对；上游函数定位与逐字段表仍缺）';
      filled += 1;
    } else if (!row.status.includes('W15 头文件路径已对')) {
      row.status = `${row.status}；W15 头文件路径已对`;
    }
  } else {
    row.warpHeaders = [];
    row.warpHeaderCoverage = {
      ...r.coverage,
      note: 'W15 头树无直接函数对应（上游检出的仅为运行时 API，产品侧为纯索引/整型搬运或自有命名原语）；不把该缺口写成通过。',
    };
  }
  if (row.status.startsWith('待补')) stillTodo += 1;
}
inventory.warpHeaderEvidence = {
  source: headerSource,
  coverage: {
    rows: rows.length,
    rowsWithHeaderEvidence: rows.filter((r) => r.coverage.headers > 0).length,
    rowsWithoutHeaderEvidence: rows.filter((r) => r.coverage.headers === 0).map((r) => r.row.id),
    rowsUpgradedFromTodo: filled,
    rowsStillTodo: stillTodo,
    totalHeadersMapped: new Set(rows.flatMap((r) => r.warpHeaders.map((h) => h.path))).size,
  },
  mapping: 'warp-headers.json',
};
inventory.updatedAt = '2026-10-23';
writeFileSync(path.join(HERE, 'inventory.json'), JSON.stringify(inventory, null, 2) + '\n', 'utf8');

console.log(JSON.stringify({
  rows: rows.length,
  headerFiles: headerFiles.length,
  rowsWithEvidence: rows.filter((r) => r.coverage.headers > 0).length,
  upgraded: filled,
  stillTodo,
  rowsWithoutEvidence: rows.filter((r) => r.coverage.headers === 0).map((r) => r.row.id),
  statusBefore,
}, null, 2));
