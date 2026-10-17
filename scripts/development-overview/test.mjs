import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { once } from 'node:events';
import { get } from 'node:http';
import { counts, stageOf, crossLinks, related, ancestors, filterTasks, validateSchedule, layerPrefix } from './model.mjs';
import { createOverviewServer, readReceipt, NormifyReader } from './server.mjs';

const task = (id, deps = [], readiness = 'blocked', phase = 'P4') => ({ id, title: { zh: id },
  depends_on: deps, readiness, phase, modules: ['mjwarp.physics'], files: ['src/physics/mod.rs'] });
const base = task(layerPrefix + 'baseline', [], 'verified', 'cross/M0');
const m0 = task(layerPrefix + 'm0-w16', [base.id], 'in_progress', 'execution/M0');
const gate = task('2026-10-11-roadmap-p1', [m0.id], 'blocked', 'P1');
const abandoned = task('abandoned', [], 'abandoned', '历史');
const sample = { ok: true, tasks: [base, m0, gate, abandoned], errors: [], warnings: [] };

test('计数只汇总原始状态，不把终止算作完成', () => {
  assert.deepEqual(counts(sample.tasks), { total: 4, verified: 1, in_progress: 1, blocked: 1, abandoned: 1 });
});
test('阶段匹配区分P1与P10，保留三层独立标签', () => {
  assert.equal(stageOf(task('x', [], 'ready', 'P10')), 'P10');
  assert.equal(stageOf(task('x', [], 'ready', 'P4-T4剩余')), 'P4');
  assert.equal(stageOf(m0), '');
});
test('跨链只使用已登记依赖，不把阶段标签当作依赖', () => {
  assert.deepEqual(crossLinks(sample.tasks), [{ from: m0.id, to: gate.id }]);
  assert.deepEqual(crossLinks([base, task('unlinked')]), []);
});
test('前置、后继和历史链完整保留', () => {
  assert.deepEqual(related(sample.tasks, m0.id), { before: [base], after: [gate] });
  assert.deepEqual(ancestors(sample.tasks, gate.id), [base, m0]);
});
test('异常环不会死循环，缺失前置不会假装完成', () => {
  assert.equal(related([task('x', ['missing'])], 'x').before[0].readiness, 'missing');
  assert.deepEqual(ancestors([task('x', ['y']), task('y', ['x'])], 'x').map(t=>t.id), ['y']);
});
test('筛选覆盖全部状态、模块、文件和阶段', () => {
  assert.equal(filterTasks(sample.tasks).length, 4);
  assert.deepEqual(filterTasks(sample.tasks, { state: 'verified' }), [base]);
  assert.deepEqual(filterTasks(sample.tasks, { stage: 'P1' }), [gate]);
  assert.equal(filterTasks(sample.tasks, { query: 'SRC/PHYSICS' }).length, 4);
  assert.equal(filterTasks(sample.tasks, { query: 'mjwarp.physics' }).length, 4);
});
test('拒绝空排程、重复任务及缺失依赖字段', () => {
  assert.equal(validateSchedule(sample), sample);
  for (const value of [{}, { ok: true, tasks: [] }, { ok: true, tasks: [base, base] },
    { ok: true, tasks: [{ id: 'x', readiness: 'ready' }] }]) assert.throws(()=>validateSchedule(value));
});
test('工具调用白名单拒绝写入', async () => {
  await assert.rejects(NormifyReader.prototype.call.call({}, 'normify_change_update', {}), /非只读/);
});
test('回执保持原里程碑，并拒绝任务错配', async () => {
  const root = await mkdtemp(join(tmpdir(), 'mjwarp-overview-'));
  try {
    const folder = join(root, 'docs/development-evidence/changes');
    await mkdir(folder, { recursive: true });
    const change = { id: 'x', note: 'delivery-receipt: docs/development-evidence/changes/x.json' };
    assert.equal((await readReceipt({ id: 'x' }, root)).status, '未绑定回执');
    assert.equal((await readReceipt(change, root)).status, '尚无回执文件');
    await writeFile(join(folder, 'x.json'), JSON.stringify({ task_id: 'x', milestones: { delivery_record: 'pending_commit' } }));
    assert.equal((await readReceipt(change, root)).receipt.milestones.delivery_record, 'pending_commit');
    assert.equal((await readReceipt({ ...change, id: 'y' }, root)).status, '回执读取失败');
    assert.equal((await readReceipt({ id: 'x', note: 'delivery-receipt: ../../secrets.json' }, root)).status, '未绑定回执');
  } finally { await rm(root, { recursive: true, force: true }); }
});
test('回环服务查询原排程，保护只读边界并显式返回故障', async t => {
  const calls = [];
  let fail = false;
  const reader = { async call(name, args) {
    calls.push({ name, args });
    if (fail) throw new Error('模拟服务失效');
    if (args.view === 'schedule') return sample;
    return { ok: true, change: { id: args.id, acceptance: ['真实验收'], note: '' } };
  } };
  const { server, basePath } = createOverviewServer(reader, { token: 'test-token' });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const origin = `http://127.0.0.1:${server.address().port}`;
  const url = origin + basePath;
  try {
    await t.test('首页与脚本只提供白名单资源', async () => {
      const response = await fetch(url);
      assert.equal(response.status, 200);
      assert.match(response.headers.get('content-security-policy'), /frame-ancestors 'none'/);
      assert.match(await response.text(), /开发排程总览/);
      assert.equal((await fetch(url + 'app.mjs')).status, 200);
      assert.equal((await fetch(url + 'server.mjs')).status, 404);
      assert.equal((await fetch(origin + '/wrong/')).status, 404);
    });
    await t.test('全量任务与原引擎完全一致', async () => {
      const value = await (await fetch(url + 'api/schedule')).json();
      assert.deepEqual(value.schedule, sample);
      assert.ok(value.fetchedAt);
      assert.equal(calls.at(-1).name, 'normify_change_list');
      assert.equal(calls.at(-1).args.view, 'schedule');
    });
    await t.test('详情获取原验收，拒绝非法标识', async () => {
      const value = await (await fetch(url + 'api/task?id=x')).json();
      assert.deepEqual(value.change.acceptance, ['真实验收']);
      assert.equal((await fetch(url + 'api/task?id=../../x')).status, 400);
    });
    await t.test('拒绝跨源、跨站及写入请求', async () => {
      assert.equal((await fetch(url, { headers: { origin: 'https://example.com' } })).status, 403);
      assert.equal((await fetch(url, { headers: { 'sec-fetch-site': 'cross-site' } })).status, 403);
      assert.equal((await fetch(url, { method: 'POST' })).status, 405);
      // fetch会重写Host。原生请求才能验证该边界。
      const hostStatus = await new Promise((resolve, reject) => {
        get(url, { headers: { host: 'malicious.example' } }, response => {
          response.resume(); resolve(response.statusCode);
        }).on('error', reject);
      });
      assert.equal(hostStatus, 403);
      const navigation = { 'sec-fetch-site': 'cross-site', 'sec-fetch-mode': 'navigate', 'sec-fetch-dest': 'document' };
      const rawStatus = path => new Promise((resolve, reject) => {
        get(path, { headers: navigation }, response => { response.resume(); resolve(response.statusCode); }).on('error', reject);
      });
      assert.equal(await rawStatus(url), 200);
      assert.equal(await rawStatus(url + 'api/schedule'), 403);
    });
    await t.test('失效返回503，不提供伪造空排程', async () => {
      fail = true;
      const response = await fetch(url + 'api/schedule');
      assert.equal(response.status, 503);
      assert.match((await response.json()).error, /模拟服务失效/);
    });
  } finally { server.closeAllConnections(); await new Promise(resolve=>server.close(resolve)); }
});
