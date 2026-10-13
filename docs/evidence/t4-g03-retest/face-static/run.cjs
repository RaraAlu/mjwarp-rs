const fs = require('fs');
const assert = require('assert/strict');
const crypto = require('crypto');
const started = performance.now();
const productPath = 'target/t4-face-operators/product.ptx';
const frozenPath = 'target/mjwarp-parallel/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx';
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
assert.equal(hash(fs.readFileSync(productPath)), '6832e3250af25b8108dd65eed170194e3e961385adb24e699f01bcd528376827');
assert.equal(hash(fs.readFileSync(frozenPath)), '1d9e4e3a983ca35090ade3a48fd83aa0dc86a23228bb0865010f681698ab75dd');
const product = fs.readFileSync('target/t4-face-operators/product.ptx', 'utf8').replace(/\r/g, '');
const frozen = fs.readFileSync('target/mjwarp-parallel/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx', 'utf8').replace(/\r/g, '');
const interner = new Map();
const expressions = [];
function intern(op, args) {
  if (['add', 'mul'].includes(op)) args.sort();
  if (op === 'fma' && args[0] > args[1]) [args[0], args[1]] = [args[1], args[0]];
  const key = JSON.stringify([op, ...args]);
  if (!interner.has(key)) { interner.set(key, String(expressions.length)); expressions.push([op, ...args]); }
  return interner.get(key);
}
function parse(text, start, end, inputs) {
  const env = new Map(Object.entries(inputs).map(([k,v]) => [k, intern('input', [v])]));
  const scope = Math.max(0, text.indexOf('.visible .entry _flex_face_kinematics_9a1620ae_cuda_kernel_forward('));
  const begin = text.indexOf(start, scope);
  const stop = text.indexOf(end, begin);
  if (begin < 0 || stop < 0) throw Error('section missing');
  for (const line of text.slice(begin, stop).split('\n')) {
    const match = line.match(/^\s*(mov|add|sub|mul|fma|div|sqrt|abs)(?:\.rn)?\.f32\s+(%f\d+),\s*([^;]+);/);
    if (!match) continue;
    const [, op, dst, raw] = match;
    const args = raw.split(',').map(x => x.trim()).map(x => env.get(x) ?? (x.startsWith('%') ? (() => {throw Error('missing register '+x);})() : intern('literal', [x])));
    env.set(dst, op === 'mov' ? args[0] : intern(op, args));
  }
  return env;
}
const p = parse(product, '$L__BB0_13:', '\n\tsetp.lt.f32', {'%f358':'qx','%f357':'qy','%f356':'qz','%f359':'qw','%f33':'F0x','%f36':'F0y','%f39':'F0z','%f32':'F1x','%f35':'F1y','%f38':'F1z','%f31':'F2x','%f34':'F2y','%f37':'F2z'});
const u = parse(frozen, '$L__BB42_54:', '\n\tsetp.lt.f32', {'%f706':'qx','%f707':'qy','%f708':'qz','%f709':'qw','%f685':'F0x','%f682':'F0y','%f679':'F0z','%f684':'F1x','%f681':'F1y','%f678':'F1z','%f683':'F2x','%f680':'F2y','%f677':'F2z'});
const pairs = [['R0x','%f156','%f425'],['R0y','%f162','%f431'],['R0z','%f166','%f435'],['R1x','%f173','%f442'],['R1y','%f177','%f446'],['R1z','%f181','%f450'],['R2x','%f188','%f457'],['R2y','%f192','%f461'],['R2z','%f196','%f465'],['denom','%f236','%f511'],['omegaX','%f44','%f164'],['omegaY','%f45','%f165'],['omegaZ','%f46','%f166'],['omegaLength','%f47','%f167']];
for (const [name,a,b] of pairs) console.log(name, p.get(a) === u.get(b) ? 'same' : 'DIFFERENT', p.get(a), u.get(b));

const qp = parse(product, '$L__BB0_62:', '\n\tmul.f32 \t%f329', {'%f358':'qx','%f357':'qy','%f356':'qz','%f359':'qw','%f61':'rx','%f72':'ry','%f84':'rz','%f371':'rw'});
const qu = parse(frozen, '$L__BB42_103:', '\n\tmul.f32 \t%f600', {'%f706':'qx','%f707':'qy','%f708':'qz','%f709':'qw','%f184':'rx','%f195':'ry','%f206':'rz','%f701':'rw'});
const qPairs = [['nextX','%f95','%f217'],['nextY','%f96','%f218'],['nextZ','%f97','%f219'],['nextW','%f98','%f220']];
const np = parse(product, '\n\tmul.f32 \t%f329', '\n\tsetp.gt.f32', {'%f95':'nx','%f96':'ny','%f97':'nz','%f98':'nw'});
const nu = parse(frozen, '\n\tmul.f32 \t%f600', '\n\tsetp.leu.f32', {'%f217':'nx','%f218':'ny','%f219':'nz','%f220':'nw'});
const rows = pairs.map(([name,a,b]) => ({name,product_register:a,upstream_register:b,nominal_graph_same:p.get(a)===u.get(b)}));
rows.push(...qPairs.map(([name,a,b]) => ({name,product_register:a,upstream_register:b,nominal_graph_same:qp.get(a)===qu.get(b)})));
rows.push({name:'nextLength',product_register:'%f99',upstream_register:'%f221',nominal_graph_same:np.get('%f99')===nu.get('%f221')});
assert.equal(rows.length,19);
assert(rows.every(row => row.nominal_graph_same));
function excerpt(text, needle, count) {
  const lines = text.split('\n');
  const scope = Math.max(0, lines.findIndex(line => line.includes('.visible .entry _flex_face_kinematics_9a1620ae_cuda_kernel_forward(')));
  const index = lines.findIndex((line, index) => index >= scope && line.includes(needle));
  assert(index >= 0);
  return {line:index+1,lines:lines.slice(index,index+count)};
}
const rounding = [0,1,2].map((axis) => {
  const product_register = [314,318,322][axis];
  const upstream_register = [585,589,593][axis];
  const a = excerpt(product, 'mul.rn.f32 \t%f'+product_register+',', 2);
  const b = excerpt(frozen, 'mul.f32 \t%f'+upstream_register+',', 2);
  assert(a.lines[1].trim().startsWith('sub.rn.f32'));
  assert(b.lines[1].trim().startsWith('sub.f32'));
  return {axis,product:a,upstream:b,explicit_rounding_same:false};
});
// 只核对声明式限定差异；不预测驱动最终融合。
const bad = product.replace('fma.rn.f32 \t%f147, %f359, %f146, 0fBF800000;', 'fma.rn.f32 \t%f147, %f359, %f146, 0fC0000000;');
const badEnv = parse(bad, '$L__BB0_13:', '\n\tsetp.lt.f32', {'%f358':'qx','%f357':'qy','%f356':'qz','%f359':'qw','%f33':'F0x','%f36':'F0y','%f39':'F0z','%f32':'F1x','%f35':'F1y','%f38':'F1z','%f31':'F2x','%f34':'F2y','%f37':'F2z'});
assert.notEqual(badEnv.get('%f156'),u.get('%f425'));
assert.throws(()=>parse(product,'$L__BB0_13:','\n\tsetp.lt.f32',{}),/missing register/);
const apiSources = [
  'target/mjwarp-parallel/venv/Lib/site-packages/warp/native/warp.h',
  'target/mjwarp-parallel/venv/Lib/site-packages/warp/_src/context.py',
  'target/mjwarp-parallel/venv/Lib/site-packages/warp/_src/codegen.py',
  'target/mjwarp-parallel/upstream/mujoco_warp/_src/passive.py',
  'target/flex-stretch-upstream/passive.py'
].map(path=>({path,sha256:hash(fs.readFileSync(path))}));
const directories = ['target/flex-stretch-upstream','target/flex-stretch-checkout','target/flex-hessian-verification','target/research'];
function walk(dir,files=[]) {
  for (const entry of fs.readdirSync(dir,{withFileTypes:true})) {
    const path = dir+'/'+entry.name;
    if(entry.isDirectory()) walk(path,files);
    else if(entry.isFile()) files.push(path);
    else throw Error('unsupported file type: '+path);
  }
  return files;
}
const inventory = directories.map(directory=>{
  const files=walk(directory);
  const generated=files.filter(path=>/\.(cu|ptx|cubin)$/i.test(path));
  return {directory,files:files.length,generated_files:generated};
});
assert(inventory.every(row=>row.generated_files.length===0));
assert.notEqual(apiSources[3].sha256,apiSources[4].sha256);
const report = {
  schema:1,scope:'existing-PTX-nominal-graph-and-contraction-permissions-only',
  inputs:[{path:productPath,sha256:hash(fs.readFileSync(productPath))},{path:frozenPath,sha256:hash(fs.readFileSync(frozenPath))}],
  assumptions:['identical-q-and-F-input-register-labels','finite-operands-no-NaN-payload-claim','commute-add-and-multiply-operands-only','preserve-operation-nesting-and-FMA-boundaries','default-and-explicit-rn-have-same-nominal-rounding-but-not-same-optimization-permission'],
  nominal_graphs:rows,rounding_permission_differences:rounding,
  nominal_graph_matches:rows.length,rounding_permission_differences_count:3,
  includes_geometry_assembly:false,includes_trig_control_flow:false,includes_normalization_branch:false,
  negative_checks:2,api_sources:apiSources,local_artifact_search:inventory,
  actual_hessian_generated_artifact:false,actual_machine_instructions_obtained:false,
  residual_first_difference_proven:false,residual_bit_differences:2078,
  candidate:'JIT contraction permission differs for quaternion-xyz final multiply/subtract; actual contraction remains unmeasured',
  official_semantics:'https://docs.nvidia.com/cuda/archive/12.8.1/parallel-thread-execution/index.html#floating-point-instructions-mul',
  product_changes:0,gpu_executions:0,physics_compiler_invocations:0,python_executions:0,full_acceptance:false,
  elapsed_ms:performance.now()-started
};
fs.writeFileSync('docs/evidence/t4-g03-retest/face-static/report.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({nominal_graph_matches:19,rounding_permission_differences:3,negative_checks:2,full_acceptance:false}));
