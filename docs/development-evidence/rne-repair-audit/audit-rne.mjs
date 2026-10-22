// RNE 1042 块归属审计（只读）。
// 对比：当前产品位串（authoring dump）× 历史原生 f64 期望 × 重基线后的上游 f32 期望。
// 依赖：docs/development-evidence/g02-rne-authoring-diff/product-rne.json
//       docs/development-evidence/g02-rne-acc-rebaseline/historical-f64/*.json
//       fixtures/rne-acceleration/*.json
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../../..');
const authoring = join(root, 'docs/development-evidence/g02-rne-authoring-diff');
const historical = join(root, 'docs/development-evidence/g02-rne-acc-rebaseline/historical-f64');
const fixtures = join(root, 'fixtures/rne-acceleration');
const outFile = join(here, 'attribution.json');
const ABS = 2e-5;
const REL = 2e-5;
const models = ['mixed-joints', 'inertial-tree', 'rotated-tree', 'zero-dof', 'massless-tree'];

const product = JSON.parse(readFileSync(join(authoring, 'product-rne.json'), 'utf8')).records;

function hexToF32(hex) {
  const buffer = Buffer.from(hex, 'hex');
  const values = [];
  // f32::to_bits() 的数值十六进制 = 大端字节序。
  for (let offset = 0; offset + 4 <= buffer.length; offset += 4) values.push(buffer.readFloatBE(offset));
  return values;
}

function expected(file, state, flag, gravity) {
  const value = JSON.parse(readFileSync(file, 'utf8'));
  return value.cases[state].qfrc[flag ? 1 : 0][gravity];
}

const records = [];
for (const model of models) {
  for (const record of product.filter(item => item.model === model)) {
    const actual = hexToF32(record.qfrc_bias_hex);
    const oldValues = expected(join(historical, model + '.json'), record.state, record.flag, record.gravity);
    const newValues = expected(join(fixtures, model + '.json'), record.state, record.flag, record.gravity);
    const evaluate = values => {
      let worst = 0;
      let worstDof = 0;
      let over = 0;
      for (let dof = 0; dof < actual.length; dof++) {
        const normalized = Math.abs(actual[dof] - values[dof]) / (ABS + REL * Math.abs(values[dof]));
        if (normalized > 1) over += 1;
        if (normalized > worst) { worst = normalized; worstDof = dof; }
      }
      return { maxNormalized: Number(worst.toFixed(9)), worstDof, dofsOver: over };
    };
    records.push({ model, gravity: record.gravity, state: record.state, flag: record.flag, nv: actual.length,
      historicalF64: evaluate(oldValues), rebaselinedF32: evaluate(newValues) });
  }
}
const failOld = records.filter(row => row.historicalF64.maxNormalized > 1);
const failNew = records.filter(row => row.rebaselinedF32.maxNormalized > 1);
const round = value => Number(value.toFixed(6));
const summary = {
  scope: 'product bit dump (current code) vs historical native f64 expected vs rebaselined upstream f32 expected',
  tolerances: { absolute: ABS, relative: REL },
  productDump: 'docs/development-evidence/g02-rne-authoring-diff/product-rne.json（当前产品代码离线 dump，240 条记录）',
  expectedSources: {
    historicalF64: 'docs/development-evidence/g02-rne-acc-rebaseline/historical-f64/（b13d1c5 前的夹具值）',
    rebaselinedF32: 'fixtures/rne-acceleration/（b13d1c5 后，上游 f32 参考）',
  },
  records: records.length,
  historicalF64: {
    failedRecords: failOld.length,
    blockEstimate: failOld.length * 521,
    worst: round(Math.max(...records.map(row => row.historicalF64.maxNormalized))),
    failing: failOld.map(row => ({ model: row.model, gravity: row.gravity, state: row.state, flag: row.flag, ...row.historicalF64 })),
  },
  rebaselinedF32: {
    failedRecords: failNew.length,
    blockEstimate: failNew.length * 521,
    worst: round(Math.max(...records.map(row => row.rebaselinedF32.maxNormalized))),
    failing: failNew,
  },
  stateGroups: {
    '2026-10-14 文档 reported': [
      { model: 'mixed-joints', gravity: 0, state: 3, flag: false, blocks: 521 },
      { model: 'mixed-joints', gravity: 2, state: 7, flag: false, blocks: 521 },
    ],
    'current product vs historical f64': failOld.map(row => ({ model: row.model, gravity: row.gravity, state: row.state, flag: row.flag, blocks: 521, maxNormalized: row.historicalF64.maxNormalized })),
    'current product vs rebaselined f32': failNew.map(row => ({ model: row.model, gravity: row.gravity, state: row.state, flag: row.flag, blocks: 521 })),
  },
  coverage: {
    blocksPerState: '每个 (state, gravity, flag) 组覆盖 worlds [1,2,5,513] × 8 rounds，按 state 计 1+2+5+513 = 521 个 world-field 块',
    historical1042: '两组 × 521 = 1042；不是单一组 521 的统计扩大',
    remainingAfterRebaseline: failNew.length * 521,
  },
  details: records.map(row => ({ model: row.model, gravity: row.gravity, state: row.state, flag: row.flag,
    historicalMax: row.historicalF64.maxNormalized, historicalDof: row.historicalF64.worstDof, historicalDofsOver: row.historicalF64.dofsOver,
    rebaselinedMax: row.rebaselinedF32.maxNormalized, rebaselinedDofsOver: row.rebaselinedF32.dofsOver })),
};
writeFileSync(outFile, JSON.stringify(summary, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ failedHistorical: summary.historicalF64.failedRecords, blocksHistorical: summary.historicalF64.blockEstimate,
  failedRebaselined: summary.rebaselinedF32.failedRecords, worstHistorical: summary.historicalF64.worst,
  worstRebaselined: summary.rebaselinedF32.worst }, null, 2));
