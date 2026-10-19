'use strict';
const S = require('./symeval.cjs');
const { nodes, show } = S;
const [fa, la, fb, lb] = process.argv.slice(2);
const A = S.run(fa, la, 'A:', S.policyFor);
const B = S.run(fb, lb, 'B:', S.policyFor);
const U = S.makeUnifier();
const used = new Set();
const unmatched = [];
for (const [aa, pa] of A.checkpoints) {
  let hit = null;
  for (const [ab, pb] of B.checkpoints) {
    if (used.has(ab)) continue;
    const mark = U.trail.length;
    if (U.u(pa, pb)) { hit = ab; break; }
    U.undo(mark);
  }
  if (hit) used.add(hit);
  console.log(aa, '->', hit || 'NONE', nodes[pa].op, S.size(pa));
  if (!hit) unmatched.push(pa);
}
console.log('B unmatched', B.checkpoints.filter(([ab]) => !used.has(ab)).map(([ab, p]) => ab + ' ' + nodes[p].op).join('; '));
// 收集A中所有子节点；找出在B中无对应的最小节点。
function subnodes(id, out = new Set()) { if (out.has(id)) return out; out.add(id); for (const a of nodes[id].args) subnodes(a, out); return out; }
const bAll = new Set(); for (const [, p] of B.checkpoints) subnodes(p, bAll);
for (const r of ['R2','R19','R0','R18']) subnodes(A.st.regs.get(r), new Set());
const bList = [...bAll];
const bByOp = new Map(); for (const b of bList) { const k = nodes[b].op; if (!bByOp.has(k)) bByOp.set(k, []); bByOp.get(k).push(b); }
const aAll = new Set(); for (const p of unmatched) subnodes(p, aAll);
const minimal = [];
for (const a of aAll) {
  const n = nodes[a]; if (n.op === 'leaf' || n.op === 'const') continue;
  const cands = bByOp.get(n.op) || [];
  const match = cands.some((b) => { const m = U.trail.length; const ok = U.u(a, b); U.undo(m); return ok; });
  if (match) continue;
  // 子项全部可匹配时，即为最小差异节点。
  const kidsOk = n.args.every((c) => { const cn = nodes[c]; if (cn.op === 'leaf' || cn.op === 'const') return true; return (bByOp.get(cn.op) || []).some((b) => { const m = U.trail.length; const ok = U.u(c, b); U.undo(m); return ok; }); });
  if (kidsOk) minimal.push(a);
}
console.log('minimal unmatched A nodes:', minimal.length);
for (const a of minimal.slice(0, 12)) console.log('  A#' + a, S.show(a, 3));
