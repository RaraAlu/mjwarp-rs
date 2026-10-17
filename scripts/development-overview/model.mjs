// 这里只分组展示，不推导就绪状态或新增依赖。
export const layerPrefix = '2026-10-18-layers-';
export const roadmapPrefix = '2026-10-11-roadmap-';
export const layerRows = [
  ['内核执行层', ['m0-w16', 'm1', 'm2', 'm4', 'm5', 'm6']],
  ['计算物理层', ['m3-inventory', 'm3', 'a1-contract', 'a1', 'a2']],
  ['数据交互层', ['io']],
  ['集成与契约', ['baseline', 'windows', 'linux']],
  ['可选路线评审', ['m7-headers', 'm7-link', 'm7-tile', 'm7-conditional', 'm7-proof']],
];
export const stateNames = {
  ready: '就绪', in_progress: '进行中', blocked: '阻塞', verified: '已登记完成',
  abandoned: '已终止', unscheduled: '未排程', proposed: '候选',
};
export function title(task) { return task.title?.zh || task.title?.en || task.id; }
export function counts(tasks) {
  const result = { total: tasks.length };
  for (const task of tasks) result[task.readiness] = (result[task.readiness] || 0) + 1;
  return result;
}
export function stageOf(task) { return task.phase?.match(/^P(\d+)(?=\D|$)/)?.[0] || ''; }
export function crossLinks(tasks) {
  return tasks.filter(t => !t.id.startsWith(layerPrefix)).flatMap(t =>
    (t.depends_on || []).filter(id => id.startsWith(layerPrefix)).map(from => ({ from, to: t.id })));
}
export function related(tasks, id) {
  const task = tasks.find(t => t.id === id);
  return {
    before: (task?.depends_on || []).map(dep => tasks.find(t => t.id === dep) || { id: dep, readiness: 'missing' }),
    after: tasks.filter(t => (t.depends_on || []).includes(id)),
  };
}
export function ancestors(tasks, id) {
  const byId = new Map(tasks.map(t => [t.id, t]));
  const visited = new Set([id]);
  const pending = [...(byId.get(id)?.depends_on || [])];
  while (pending.length) {
    const next = pending.pop();
    if (visited.has(next)) continue;
    visited.add(next);
    pending.push(...(byId.get(next)?.depends_on || []));
  }
  visited.delete(id);
  return tasks.filter(t => visited.has(t.id));
}
export function filterTasks(tasks, { query = '', state = '', stage = '' } = {}) {
  const q = query.trim().toLowerCase();
  return tasks.filter(t => (!state || t.readiness === state) && (!stage || stageOf(t) === stage)
    && (!q || [t.id, title(t), t.phase, ...(t.modules || []), ...(t.files || [])].join(' ').toLowerCase().includes(q)));
}
export function validateSchedule(value) {
  if (!value?.ok || !Array.isArray(value.tasks) || !value.tasks.length) throw new Error('排程响应缺少任务');
  const ids = new Set();
  for (const task of value.tasks) {
    if (typeof task.id !== 'string' || ids.has(task.id) || typeof task.readiness !== 'string'
      || !Array.isArray(task.depends_on)) throw new Error('排程任务格式异常');
    ids.add(task.id);
  }
  return value;
}
