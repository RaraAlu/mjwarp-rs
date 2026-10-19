// 静态SASS符号求值：比较两份内核一轮极分解迭代的浮点数据流。
// 只读两份反汇编文本；不启动GPU、CUDA或Python。
'use strict';
const fs = require('fs');

function parse(file) {
  const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/);
  const ins = [];
  const labels = {};
  for (const line of lines) {
    let m = line.match(/^\s*(\.L_x_\d+):/);
    if (m) { labels[m[1]] = ins.length; continue; }
    m = line.match(/\/\*([0-9a-f]{4,})\*\/\s+(.*?)\s*;/);
    if (!m) continue;
    let text = m[2].trim();
    let pred = null;
    const pm = text.match(/^@(!?)(U?P\w+)\s+(.*)$/);
    if (pm) { pred = { neg: pm[1] === '!', reg: pm[2] }; text = pm[3]; }
    const sp = text.indexOf(' ');
    const op = sp < 0 ? text : text.slice(0, sp);
    const rest = sp < 0 ? '' : text.slice(sp + 1);
    const operands = rest.length ? splitOperands(rest) : [];
    ins.push({ addr: m[1], op, operands, pred, text: m[2].trim() });
  }
  return { ins, labels };
}
function splitOperands(s) {
  const out = []; let depth = 0; let cur = '';
  for (const ch of s) {
    if (ch === '[' || ch === '(') depth++;
    if (ch === ']' || ch === ')') depth--;
    if (ch === ',' && depth === 0) { out.push(cur.trim()); cur = ''; } else cur += ch;
  }
  if (cur.trim()) out.push(cur.trim());
  return out;
}

// 哈希合并的表达式节点。
const nodes = [];
const intern = new Map();
function mk(op, args, extra = '') {
  const key = op + '|' + extra + '|' + args.join(',');
  let id = intern.get(key);
  if (id === undefined) { id = nodes.length; nodes.push({ op, args, extra }); intern.set(key, id); }
  return id;
}
const leaf = (name) => mk('leaf', [], name);
const cst = (text) => mk('const', [], text);
function neg(x) { const n = nodes[x]; return n.op === 'neg' ? n.args[0] : mk('neg', [x]); }
function abs(x) { const n = nodes[x]; return n.op === 'neg' ? mk('abs', [n.args[0]]) : (n.op === 'abs' ? x : mk('abs', [x])); }

function makeState(prefix) {
  const regs = new Map();
  const inputs = new Set();
  return {
    get(r) {
      if (r === 'RZ' || r === 'URZ') return cst('0');
      if (r === 'PT') return cst('true');
      if (!regs.has(r)) { inputs.add(r); regs.set(r, leaf(prefix + r)); }
      return regs.get(r);
    },
    set(r, v) { if (r === 'RZ' || r === 'PT' || r === 'URZ') return; regs.set(r, v); },
    regs, inputs,
  };
}

function value(st, o) {
  o = o.replace(/\.reuse/g, '');
  let m;
  if ((m = o.match(/^-\|(.*)\|$/))) return neg(abs(value(st, m[1])));
  if ((m = o.match(/^\|(.*)\|$/))) return abs(value(st, m[1]));
  if (o.startsWith('!')) return mk('not', [value(st, o.slice(1))]);
  if (o === '-RZ') return cst('-0');
  if (/^-U?R\d+/.test(o)) return neg(value(st, o.slice(1)));
  if (/^U?R\d+$/.test(o) || o === 'RZ' || o === 'URZ' || /^U?P\d+$/.test(o) || o === 'PT') return st.get(o);
  return cst(normConst(o)); // 立即数、常量库或标签。
}

// 立即数统一为32位位型；十六进制按整数，十进制按f32。
function normConst(o) {
  const buf = new DataView(new ArrayBuffer(4));
  if (/^-?0x[0-9a-f]+$/i.test(o)) {
    const v = BigInt(o.replace(/^-/, '')) * (o.startsWith('-') ? -1n : 1n);
    return 'b' + BigInt.asUintN(32, v).toString(16);
  }
  if (/^[+-]?INF$/.test(o)) { buf.setFloat32(0, o.startsWith('-') ? -Infinity : Infinity); return 'b' + buf.getUint32(0).toString(16); }
  if (/^[+-]?\d+(\.\d*)?(e[+-]?\d+)?$/i.test(o)) { buf.setFloat32(0, parseFloat(o)); return 'b' + buf.getUint32(0).toString(16); }
  return o;
}

// 可交换浮点运算统一排序，并把乘数符号提到外层。
function fp(op, mods, args) {
  if (op === 'FMUL') {
    let [a, b] = args; let s = 0;
    if (nodes[a].op === 'neg') { a = nodes[a].args[0]; s ^= 1; }
    if (nodes[b].op === 'neg') { b = nodes[b].args[0]; s ^= 1; }
    const r = mk('FMUL', [a, b].sort((x, y) => x - y), mods);
    return s ? neg(r) : r;
  }
  if (op === 'FADD') return mk('FADD', args.slice().sort((x, y) => x - y), mods);
  if (op === 'FFMA') {
    let [a, b, c] = args; let s = 0;
    if (nodes[a].op === 'neg') { a = nodes[a].args[0]; s ^= 1; }
    if (nodes[b].op === 'neg') { b = nodes[b].args[0]; s ^= 1; }
    const [x, y] = [a, b].sort((p, q) => p - q);
    return mk('FFMA', [x, y, c], mods + (s ? '-' : '+'));
  }
  throw new Error(op);
}

const COPY = new Set(['MOV', 'IMAD.MOV.U32', 'IMAD.MOV']);
const SKIP = /^(BSSY|BSYNC|NOP|WARPSYNC|STG|STL|BMOV|DEPBAR|YIELD)/;

function run(file, start, prefix, policy, stopAt) {
  const { ins, labels } = parse(file);
  const st = makeState(prefix);
  const trace = []; const checkpoints = [];
  let pc = labels[start];
  if (pc === undefined) throw new Error('label ' + start);
  let steps = 0;
  while (true) {
    if (++steps > 20000) throw new Error('runaway');
    const I = ins[pc];
    if (!I) throw new Error('fell off');
    const base = I.op.split('.')[0];
    const mods = I.op.slice(base.length);
    trace.push(I.addr + ' ' + I.text);
    if (base === 'BRA' || base === 'CALL' || base === 'EXIT' || base === 'RET') {
      let take = true;
      if (I.pred) {
        const p = st.get(I.pred.reg);
        const decision = policy(p, I); checkpoints.push([I.addr, p]);
        if (decision === 'stop') return { st, trace, checkpoints, stopped: I.addr };
        take = I.pred.neg ? !decision : decision;
      }
      if (!take) { pc++; continue; }
      if (base === 'CALL') throw new Error('call taken at ' + I.addr + ' ' + I.text);
      if (base !== 'BRA') throw new Error('exit at ' + I.addr);
      const target = I.operands[0].match(/\((\.L_x_\d+)\)/)[1];
      if (stopAt && target === stopAt) return { st, trace, checkpoints, stopped: I.addr };
      pc = labels[target];
      continue;
    }
    if (SKIP.test(I.op)) { pc++; continue; }
    const ops = I.operands;
    // 谓词执行中，同谓词选择折叠到已知分支；死寄存器残值不进入比较。
    const pctx = I.pred ? [st.get(I.pred.reg), !I.pred.neg] : null;
    const val = (o) => assume(value(st, o), pctx);
    let dest = ops[0];
    let result;
    if (COPY.has(I.op)) {
      result = val(I.op === 'MOV' ? ops[1] : ops[3]);
      if (I.op !== 'MOV' && !(ops[1] === 'RZ' && ops[2] === 'RZ')) result = mk(I.op, ops.slice(1).map((o) => val(o)));
    } else if (base === 'FMUL' || base === 'FADD' || base === 'FFMA') {
      result = fp(base, mods, ops.slice(1).map((o) => val(o)));
    } else if (base === 'FSEL') {
      // FSEL d,a,b,P：P真取a。
      let p = value(st, ops[3]); let a = val(ops[1]); let b = val(ops[2]);
      if (nodes[p].op === 'not') { p = nodes[p].args[0]; [a, b] = [b, a]; }
      result = mk('sel', [p, assume(a, [p, true]), assume(b, [p, false])]);
    } else if (/^(FSETP|ISETP|LOP3|PLOP3|FCHK|IADD3|SHF|LEA|IMAD|IABS|I2F|F2I|I2FP|FRND|MUFU|LOP|SEL|ISCADD|PRMT|POPC|FLO|IMNMX|FMNMX|DMUL|DADD|DFMA|F2F|LDL|LDG|LDS|LDC|ULDC|S2R|UMOV|USHF|UIMAD|UIADD3|ULOP3|SHL|SHR|BMSK|BREV|FSWZADD|P2R|R2P|VOTE|CS2R|IDP|HFMA2)/.test(base)) {
      // 其余操作按名称与全部源操作数保持不透明。
      const twoDest = /^(FSETP|ISETP|PLOP3|DSETP)/.test(base) || (base === 'LOP3' && /^P\d|^PT$/.test(ops[0]));
      const srcs = twoDest ? ops.slice(2) : ops.slice(1);
      result = mk(I.op, srcs.map((o) => val(o)));
      if (base === 'LOP3' && /^P\d|^PT$/.test(ops[0])) {
        const srcs2 = ops.slice(2);
        st.set(ops[1], mk(I.op + '#r', srcs2.map((o) => val(o))));
      }
      if (/^(IADD3|IMAD)/.test(base) && /^P\d$/.test(ops[1] || '')) throw new Error('carry dest ' + I.text);
    } else {
      throw new Error('unknown op ' + I.op + ' at ' + I.addr);
    }
    if (I.pred) {
      const old = st.get(dest);
      const p = st.get(I.pred.reg);
      result = I.pred.neg ? mk('sel', [p, old, result]) : mk('sel', [p, result, old]);
    }
    st.set(dest, result);
    pc++;
  }
}

// 分支取向：除法与开方取快路径，极分解继续迭代，三角函数取小角。
function policyFor(p, I) {
  const n = nodes[p];
  const consts = n.args.map((a) => nodes[a].op === 'const' ? nodes[a].extra : '');
  if (n.op.startsWith('FCHK')) return false;
  if (n.op.startsWith('ISETP') && consts.includes('b32')) return 'stop';
  if (n.op.startsWith('ISETP') && consts.includes('b727fffff')) return false;
  if (n.op.startsWith('ISETP') && consts.includes('b1ffffff')) return true;
  if (n.op.startsWith('FSETP.GEU') && consts.includes('b358637bd')) return true;
  if (n.op.startsWith('FSETP.GE.') && consts.includes('b47ce4780')) return false;
  if (n.op.startsWith('FSETP.GT.') && consts.includes('0')) return true;
  throw new Error('no policy for predicate ' + n.op + ' ' + consts + ' at ' + I.addr + ' ' + I.text);
}

// 结构合一：输入寄存器按一一映射，交换律两种次序回溯。
function makeUnifier() {
  const fwd = new Map(); const back = new Map(); const memo = new Set(); const trail = [];
  function undo(mark) {
    while (trail.length > mark) {
      const t = trail.pop();
      if (t[0] === 'map') { fwd.delete(t[1]); back.delete(t[2]); } else memo.delete(t[1]);
    }
  }
  function u(a, b) {
    if (a === b) return true;
    const key = a + ':' + b;
    if (memo.has(key)) return true;
    const A = nodes[a]; const B = nodes[b];
    if (A.op === 'leaf' && B.op === 'leaf') {
      const fa = fwd.get(A.extra); const fb = back.get(B.extra);
      if (fa === undefined && fb === undefined) { fwd.set(A.extra, B.extra); back.set(B.extra, A.extra); trail.push(['map', A.extra, B.extra]); return true; }
      return fa === B.extra;
    }
    if (A.op !== B.op || A.extra.replace(/[+-]$/, '') !== B.extra.replace(/[+-]$/, '') || A.args.length !== B.args.length) {
      return false;
    }
    if (A.op === 'const') return A.extra === B.extra;
    const mark = trail.length;
    const orders = [];
    if (A.op === 'FADD' || A.op === 'FMUL') orders.push([0, 1], [1, 0]);
    else if (A.op === 'FFMA') orders.push([0, 1, 2], [1, 0, 2]);
    else orders.push(A.args.map((_, i) => i));
    if (A.op === 'FFMA' && A.extra.slice(-1) !== B.extra.slice(-1)) return false;
    for (const ord of orders) {
      let ok = true;
      for (let i = 0; i < ord.length && ok; i++) ok = u(A.args[i], B.args[ord[i]]);
      if (ok) { memo.add(key); trail.push(['memo', key]); return true; }
      undo(mark);
    }
    return false;
  }
  return { u, fwd, back, trail, undo };
}

function size(id, seen = new Set()) {
  if (seen.has(id)) return 0; seen.add(id);
  return 1 + nodes[id].args.reduce((s, a) => s + size(a, seen), 0);
}
function show(id, depth = 4) {
  const n = nodes[id];
  if (n.op === 'leaf' || n.op === 'const') return n.extra;
  if (depth === 0) return '#' + id;
  return n.op + (n.extra ? '[' + n.extra + ']' : '') + '(' + n.args.map((a) => show(a, depth - 1)).join(', ') + ')';
}

module.exports = { run, policyFor, makeUnifier, nodes, size, show, parse };

if (require.main === module) {
  const [fa, la, fb, lb, sa, sb] = process.argv.slice(2);
  const A = run(fa, la, 'A:', policyFor, sa);
  const B = run(fb, lb, 'B:', policyFor, sb);
  console.log('A stopped', A.stopped, 'inputs', [...A.st.inputs].join(' '));
  console.log('B stopped', B.stopped, 'inputs', [...B.st.inputs].join(' '));
  const fpOut = (S) => [...S.st.inputs].filter((r) => /^R\d+$/.test(r)).map((r) => [r, S.st.regs.get(r)]);
  const outA = fpOut(A); const outB = fpOut(B);
  const U = makeUnifier();
  const used = new Set();
  const report = [];
  // 先合一最大的输出，以尽早固定映射。
  outA.sort((x, y) => size(y[1]) - size(x[1]));
  for (const [ra, ea] of outA) {
    if (nodes[ea].op === 'leaf' && nodes[ea].extra === 'A:' + ra) { report.push([ra, 'unchanged']); continue; }
    let found = null;
    for (const [rb, eb] of outB) {
      if (used.has(rb)) continue;
      const mark = U.trail.length;
      if (U.u(ea, eb)) { found = rb; break; }
      U.undo(mark);
    }
    if (found) { used.add(found); report.push([ra, '=> ' + found, size(ea)]); } else report.push([ra, 'NO MATCH', size(ea), show(ea, 3)]);
  }
  for (const r of report) console.log(r.join('  '));
  console.log('input map', [...U.fwd.entries()].map((e) => e.join('->')).join(' '));
}

// 已知谓词真值时折叠顶层选择；穿过取负与绝对值。
function assume(x, ctx) {
  if (!ctx) return x;
  const [p, truth] = ctx;
  const n = nodes[x];
  if (n.op === 'sel' && n.args[0] === p) return assume(truth ? n.args[1] : n.args[2], ctx);
  if (n.op === 'neg') return neg(assume(n.args[0], ctx));
  if (n.op === 'abs') return abs(assume(n.args[0], ctx));
  return x;
}
