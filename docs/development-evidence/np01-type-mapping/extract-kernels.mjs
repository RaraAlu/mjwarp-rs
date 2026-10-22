// NP01 类型映射清单的只读采集脚本。
// 枚举 src/physics 的产品 CUDA 内核签名，统计参数类型与 double/fma/atomicAdd/__shared__，
// 并记录编译配方与冻结 PTX 摘要。输出 raw-kernels.json 到本目录。
import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../../..');
const src = join(root, 'src');
const files = [];
function walk(dir) {
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) walk(full);
    else if (/\.(rs|cu)$/.test(name)) files.push(full);
  }
}
walk(src);

const rows = [];
for (const file of files) {
  const text = readFileSync(file, 'utf8');
  if (!/__global__/.test(text)) continue;
  const kernels = [];
  for (const match of text.matchAll(/__global__\s+(?:void|[\w:<>, ]+?)\s+([A-Za-z_]\w*)\s*\(([^)]*)\)/g)) {
    const params = match[2];
    const count = pattern => (params.match(new RegExp(pattern, 'g')) || []).length;
    kernels.push({
      name: match[1],
      params: {
        constFloatPtr: count('const\\s+float\\s*\\*'),
        floatPtr: count('(?<!const\\s)float\\s*\\*'),
        double: count('\\bdouble\\b'),
        int: count('\\bint\\b'),
        unsigned: count('\\bunsigned\\b'),
        shared: (params.match(/__shared__/g) || []).length,
      },
    });
  }
  const tokens = pattern => (text.match(new RegExp(pattern, 'g')) || []).length;
  rows.push({
    file: relative(root, file).replace(/\\/g, '/'),
    kernels: kernels.slice(0, 40),
    kernelCount: kernels.length,
    doubles: tokens('\\bdouble\\b'),
    floats: tokens('\\bfloat\\b'),
    fma: tokens('fmaf|__fmaf|fma\\('),
    atomicAdd: tokens('atomicAdd'),
    usesTile: tokens('__shared__'),
  });
}

const recipeText = readFileSync(join(root, 'src/runtime/launcher/recipe.rs'), 'utf8');
const physicsBlock = recipeText.slice(recipeText.indexOf('pub const WARP_1_15_PHYSICS'), recipeText.indexOf('pub const WARP_1_15_DIAGNOSTIC'));
const recipe = {
  name: 'warp-1.15-physics',
  fmad: /fmad:\s*true/.test(physicsBlock),
  fast_math: /fast_math:\s*true/.test(physicsBlock),
  std: '--std=c++17',
  arch: 'compute_70',
  gate: /F32Physics/.test(physicsBlock) ? 'f32-physics' : 'unknown',
};
const frozen = [];
for (const name of readdirSync(join(root, 'src/runtime')).filter(item => /^warp_\w+\.rs$/.test(item))) {
  const text = readFileSync(join(root, 'src/runtime', name), 'utf8');
  const sha = text.match(/PTX_SHA256\s*:\s*&str\s*=\s*"([0-9a-f]+)"/)?.[1] ?? text.match(/"([0-9a-f]{64})"/)?.[1] ?? null;
  frozen.push({ file: 'src/runtime/' + name, sha256: sha });
}

writeFileSync(join(here, 'raw-kernels.json'), JSON.stringify({ recipe, frozen, rows }, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ recipe, frozen, filesWithKernels: rows.length, productDoubles: rows.filter(row => row.file.startsWith('src/physics')).reduce((sum, row) => sum + row.doubles, 0) }, null, 2));
