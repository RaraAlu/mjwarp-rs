import { layerPrefix, roadmapPrefix, layerRows, stateNames, title, counts, stageOf,
  crossLinks, related, ancestors, filterTasks } from './model.mjs';

const $ = selector => document.querySelector(selector);
const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
let data, view = 'overview', selected = `${layerPrefix}m0-w16`, query = '', state = '', stage = '';
let detailVersion = 0, refreshing = false;
const tasks = () => data?.schedule.tasks || [];
const taskById = id => tasks().find(t => t.id === id);
const badge = task => `<span class="badge ${esc(task.readiness)}">${esc(stateNames[task.readiness] || task.readiness)}</span>`;
const chip = task => `<button class="task ${selected === task.id ? 'selected' : ''}" data-task="${esc(task.id)}"><i class="dot ${esc(task.readiness)}"></i>${esc(title(task))}</button>`;
const chips = values => values.length ? values.map(chip).join('') : '<p class="empty">暂无记录</p>';
const list = values => values.length ? `<div class="task-list">${values.map(t => `<button class="row-task ${selected === t.id ? 'selected' : ''}" data-task="${esc(t.id)}"><span>${esc(title(t))}<small>${esc(t.phase || '未分阶段')} · ${esc(t.id)}</small></span>${badge(t)}</button>`).join('')}</div>` : '<p class="empty">没有符合条件的任务。</p>';
const stageNames = ['范围与契约','执行路线','模型接入','状态与交换','基础物理','碰撞系统','约束与积分','复杂物理','渲染与查询','通用接口','优化与性能','发布与精度'];
const gates = ['p0','p1','p2-upload','p3','p4','p5','p6','p7','p8','p9','p10','p11'];
async function api(path) {
  const response = await fetch(path, { cache: 'no-store', signal: AbortSignal.timeout(25000) });
  const value = await response.json();
  if (!response.ok) throw new Error(value.error || `读取失败：${response.status}`);
  return value;
}
function renderMetrics() {
  const c = counts(tasks());
  $('#metrics').innerHTML = [['', '全部任务', c.total], ['in_progress', '进行中', c.in_progress || 0],
    ['ready', '就绪', c.ready || 0], ['blocked', '阻塞', c.blocked || 0], ['verified', '已登记完成', c.verified || 0],
    ['other', '终止 / 未排程', (c.abandoned || 0) + (c.unscheduled || 0)]].map(([s, label, n]) =>
    `<button class="metric ${s}" data-state="${s}"><strong>${n}</strong><span>${label}</span></button>`).join('');
}
function overview() {
  const connections = crossLinks(tasks()).filter(edge => taskById(edge.from) && edge.to.startsWith(roadmapPrefix));
  // 门禁卡展示门禁本身，不按子任务比例替代验收。
  return `<div class="board"><section class="panel"><div class="panel-heading"><h2>三层迁移主线</h2><span class="badge">M0 → 集成验证</span></div>
    <p class="caption">同排不代表相互依赖。点击任务查真实前置。</p>
    ${layerRows.map(([name, suffixes]) => `<div class="lane"><div class="lane-name">${name}</div><div class="chips">${chips(suffixes.map(s => taskById(layerPrefix + s)).filter(Boolean))}</div></div>`).join('')}
    <div class="legend">${['in_progress','ready','blocked','verified'].map(s=>`<span><i class="dot ${s}"></i> ${stateNames[s]}</span>`).join('')}</div></section>
    <section class="panel"><div class="panel-heading"><h2>完整产品路线</h2><span class="badge">P0 — P11</span></div>
    <p class="caption">点击阶段查看全部工作。状态取自正式门禁。</p><div class="stages">${gates.map((suffix, i) => {
      const gate = taskById(roadmapPrefix + suffix), values = tasks().filter(t => stageOf(t) === `P${i}`), c = counts(values);
      return `<button class="stage" data-stage="P${i}"><b>P${i}</b><span>${stageNames[i]}</span><small>${values.length}项 · 登记完成${c.verified || 0}</small>${gate ? badge(gate) : '<span class="badge">未找到门禁</span>'}</button>`;
    }).join('')}</div><h3 class="caption">精度与交付门禁</h3><div class="chips">${chips(tasks().filter(t => /^2026-10-11-roadmap-np0[1-4]$/.test(t.id)))}</div>
    <p class="caption">层迁移完成 ≠ 物理功能完成。<br>历史子集通过 ≠ 当前源码验收。</p></section></div>
    <section class="panel"><div class="panel-heading"><h2>三层 → 原开发链</h2><span class="badge">${new Set(connections.map(e=>e.to)).size}项原任务 · ${connections.length}条真实前置</span></div>
    <p class="caption">箭头指向后继任务。只展示原排程登记关系。</p><div class="connections">${connections.map(e => `<div class="connection">${chip(taskById(e.from))}<span class="link-arrow">→</span>${chip(taskById(e.to))}</div>`).join('')}</div></section>
    <section class="panel"><div class="panel-heading"><h2>当前执行与后续候选</h2><button data-view="queue">查看完整队列</button></div>${list(tasks().filter(t => t.readiness === 'in_progress' || t.readiness === 'ready'))}</section>
    <section class="panel"><div class="panel-heading"><h2>完成记录与历史证据</h2><button data-view="history">查看全部完成记录</button></div><p class="caption">详情关联验收、回执与提交。无依赖的历史记录单列。</p>${list(tasks().filter(t => t.id === layerPrefix + 'baseline' || t.id === layerPrefix + 'schedule-evidence'))}</section>`;
}
function renderList() {
  const options = Object.entries(stateNames).filter(([s]) => s !== 'proposed').map(([s, name]) => `<option value="${s}" ${state === s ? 'selected' : ''}>${name}</option>`).join('');
  const values = filterTasks(tasks(), { query, state: state === 'other' ? '' : state, stage }).filter(t => state !== 'other' || ['abandoned','unscheduled'].includes(t.readiness));
  return `<section class="panel"><div class="panel-heading"><h2>${view === 'history' ? '完成与历史' : stage ? stage + ' 阶段工作' : '全部开发任务'}</h2><span id="list-count" class="badge">${values.length}项</span></div>
    <div class="filters"><input id="query" aria-label="搜索任务" placeholder="搜索任务、模块或文件" value="${esc(query)}"><select id="state" aria-label="筛选状态"><option value="">全部状态</option>${options}<option value="other" ${state === 'other' ? 'selected' : ''}>终止 / 未排程</option></select><select id="stage" aria-label="筛选阶段"><option value="">全部阶段</option>${gates.map((_,i)=>`<option value="P${i}" ${stage === 'P'+i ? 'selected' : ''}>P${i}</option>`).join('')}</select></div>
    <div id="results">${list(values)}</div></section>`;
}
function renderMain() {
  if (!data) return;
  document.querySelectorAll('nav button').forEach(button => button.classList.toggle('active', button.dataset.view === view));
  if (view === 'overview') $('#main').innerHTML = overview();
  else if (view === 'queue') $('#main').innerHTML = `<div class="queue-grid">${['in_progress','ready','blocked','unscheduled'].map(s=>`<section class="panel ${s === 'blocked' ? 'wide' : ''}"><div class="panel-heading"><h2>${stateNames[s]}</h2><span class="badge">${tasks().filter(t=>t.readiness===s).length}项</span></div>${list(tasks().filter(t=>t.readiness===s))}</section>`).join('')}</div><p class="caption">候选队列不许可并行。须核对模块、文件与独占资源。</p>`;
  else $('#main').innerHTML = renderList();
}
function showView(next) {
  view = next; query = ''; stage = ''; state = next === 'history' ? 'verified' : '';
  renderMain();
}
async function showDetail(id) {
  selected = id;
  const version = ++detailVersion;
  const task = taskById(id);
  if (!task) { $('#detail').innerHTML = '<p>原排程已无此任务。</p>'; return; }
  const { before, after } = related(tasks(), id);
  const completed = ancestors(tasks(), id).filter(t => t.readiness === 'verified');
  const blocks = (task.blockers || []).map(b => {
    const match = b.match(/^(dependency|conflict):(.+)$/);
    if (!match) return `<li>${esc(b)}</li>`;
    return `<li>${match[1] === 'dependency' ? '未满足前置' : '占用冲突'}${chip(taskById(match[2]) || {id:match[2]})}</li>`;
  }).join('');
  $('#detail').innerHTML = `<div class="eyebrow">TASK / 原始记录</div><h2>${esc(title(task))}</h2><div class="identifier">${esc(task.id)}</div>
    <div class="detail-meta">${badge(task)}<span class="badge">${esc(task.phase || '未分阶段')}</span><span class="badge">${esc(task.owner || '未指定负责人')}</span></div>
    <p class="caption">本页展示登记状态，不替代实测。</p>
    <h3>阻塞与占用</h3>${blocks ? `<ul>${blocks}</ul>` : '<p class="muted">引擎未列出阻塞。</p>'}
    <h3>直接前置 · ${before.length}</h3><div class="detail-links">${chips(before)}</div>
    <h3>直接后继 · ${after.length}</h3><div class="detail-links">${chips(after)}</div>
    <details><summary>依赖链完成记录 · ${completed.length}</summary><p class="caption">只列真实前置链中的已完成项。</p><div class="detail-links">${chips(completed)}</div></details>
    <details><summary>模块、文件与独占资源</summary><h3>资源</h3><div class="note">${esc((task.resources || []).join('\n') || '未登记')}</div><h3>模块</h3><div class="note">${esc((task.modules || []).join('\n') || '未登记')}</div><h3>写入范围</h3><div class="note">${esc((task.files || []).join('\n') || '未登记')}</div></details>
    <div id="evidence"><p class="caption">正在读取验收与交付记录…</p></div>`;
  $('#detail').scrollTop = 0;
  document.querySelectorAll('[data-task]').forEach(button => button.classList.toggle('selected', button.dataset.task === id));
  try {
    const { change, delivery } = await api(`api/task?id=${encodeURIComponent(id)}`);
    if (version !== detailVersion) return;
    const receipt = delivery.receipt;
    const milestones = receipt?.milestones;
    const implementation = { complete: '完成', not_applicable: '不适用' };
    $('#evidence').innerHTML = `<h3>原验收条件</h3><ol>${(change.acceptance || []).map(a=>`<li>${esc(a)}</li>`).join('')}</ol>
      <h3>交付与提交</h3><div class="evidence">${esc(delivery.status)}<br>${esc(delivery.path || '')}${delivery.error ? '<br>' + esc(delivery.error) : ''}<br>关闭提交：${esc(change.revision?.after || '未登记')}<br>关闭时间：${esc(change.closed_at || '未关闭')}</div>
      ${milestones ? `<div class="evidence">实现：${esc(implementation[milestones.implementation] || milestones.implementation)}<br>范围验证：${esc(milestones.validation === 'passed' ? '回执登记通过' : milestones.validation)}<br>提交前回执状态：${esc(milestones.delivery_record)}<br>最终交付请核对关闭记录。</div>` : '<p class="caption">本记录未提供三段交付回执。</p>'}
      ${receipt ? `<details><summary>回执验证与剩余缺口</summary>${(receipt.validation || []).map(v=>`<div class="evidence">${esc(v.command)}<br>${esc(v.result)}<br>${esc(v.evidence)}</div>`).join('')}<h3>剩余缺口</h3><ul>${(receipt.remaining || []).map(r=>`<li>${esc(r)}</li>`).join('')}</ul></details>` : ''}
      <h3>任务原始备注</h3><div class="note">${esc(change.note || '未登记')}</div>`;
  } catch (error) { if (version === detailVersion) $('#evidence').innerHTML = `<p class="detail-error">详情读取失败：${esc(error.message)}</p>`; }
}
async function refresh() {
  if (refreshing) return;
  refreshing = true; $('#refresh').disabled = true;
  try {
    data = await api('api/schedule');
    $('#notice').hidden = true;
    const date = new Date(data.fetchedAt).toLocaleString('zh-CN', { timeZone: 'Asia/Shanghai', hour12: false });
    $('#sync-status').textContent = `原排程读取于 ${date}（北京时间）`;
    $('#head').textContent = `HEAD ${data.head.slice(0,12)}`;
    $('#head').title = data.head;
    renderMetrics(); renderMain(); await showDetail(selected);
    const diagnostics = [...(data.schedule.errors || []), ...(data.schedule.warnings || [])];
    if (diagnostics.length) { $('#notice').textContent = `原排程诊断：${JSON.stringify(diagnostics)}`; $('#notice').hidden = false; }
  } catch (error) {
    $('#notice').hidden = false;
    $('#notice').textContent = `刷新失败：${error.message}。${data ? '下方保留旧快照，请勿当作实时状态。' : '尚未取得排程。请检查只读服务。'}`;
    $('#sync-status').textContent = '连接失败 / 当前数据未更新';
  } finally { refreshing = false; $('#refresh').disabled = false; }
}
document.addEventListener('click', event => {
  const button = event.target.closest('button');
  if (!button) return;
  if (button.dataset.task) void showDetail(button.dataset.task);
  else if (button.dataset.view) showView(button.dataset.view);
  else if ('state' in button.dataset) { view = 'all'; query = ''; stage = ''; state = button.dataset.state; renderMain(); }
  else if (button.dataset.stage) { view = 'all'; query = ''; state = ''; stage = button.dataset.stage; renderMain(); }
});
document.addEventListener('input', event => {
  if (event.target.id !== 'query') return;
  query = event.target.value;
  const values = filterTasks(tasks(), { query, state: state === 'other' ? '' : state, stage }).filter(t => state !== 'other' || ['abandoned','unscheduled'].includes(t.readiness));
  $('#results').innerHTML = list(values); $('#list-count').textContent = `${values.length}项`;
});
document.addEventListener('change', event => {
  if (event.target.id === 'state') { state = event.target.value; renderMain(); }
  if (event.target.id === 'stage') { stage = event.target.value; renderMain(); }
});
$('#refresh').addEventListener('click', refresh);
setInterval(() => { if ($('#auto').checked && !document.hidden && document.activeElement?.id !== 'query') void refresh(); }, 60000);
void refresh();
