// 直接读取仓库内 .normify 数据，输出与原 Normify 排程相同的 JSON 契约。
// 只读：不写任务文件，不推导架构依赖，不为未登记任务承诺并行安全。
import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';

const OPEN = new Set(['proposed', 'in_progress']);
const TERMINAL = new Set(['verified', 'abandoned']);

function diag(severity, code, message, subject, evidence) {
  const value = { code, severity, message };
  if (subject !== undefined) value.subject = subject;
  if (evidence !== undefined) value.evidence = evidence;
  return value;
}

/** 只解析 [project] 段的 name；完整 TOML 语义不在这里重写。 */
export function parseProjectName(toml) {
  let inProject = false;
  for (const line of toml.split(/\r?\n/)) {
    const section = line.match(/^\s*\[([^\]]+)\]\s*$/);
    if (section) { inProject = section[1].trim() === 'project'; continue; }
    if (!inProject) continue;
    const name = line.match(/^\s*name\s*=\s*"((?:[^"\\]|\\.)*)"/);
    if (name) return name[1];
  }
  return null;
}

const titleOf = (change) => (change.title && typeof change.title === 'object')
  ? change.title : { zh: String(change.title ?? change.id) };

function readSchedules(change) {
  const schedule = change.schedule && typeof change.schedule === 'object' ? change.schedule : undefined;
  return {
    schedule,
    phase: schedule?.phase,
    owner: schedule?.owner,
    dependsOn: Array.isArray(schedule?.depends_on) ? [...schedule.depends_on] : [],
    files: Array.isArray(schedule?.files) ? [...schedule.files] : [],
    resources: Array.isArray(schedule?.resources) ? [...schedule.resources] : [],
    priority: Number.isInteger(schedule?.priority) ? schedule.priority : 50,
  };
}

/** 读取 changes/*.json；坏文件只记警告并跳过，避免单条记录拖垮整个看板。 */
export async function loadRepoChanges(normifyDir) {
  const dir = join(normifyDir, 'changes');
  let names;
  try {
    names = (await readdir(dir)).filter(name => name.endsWith('.json')).sort();
  } catch (error) {
    throw new Error(`不能读取仓库任务目录：${dir}（${error.code || error.message}）`);
  }
  const changes = [];
  const warnings = [];
  for (const name of names) {
    const file = join(dir, name);
    let value;
    try {
      value = JSON.parse(await readFile(file, 'utf8'));
    } catch (error) {
      warnings.push(diag('warning', 'schedule/change-unreadable', '任务文件无法解析，已跳过', { file: name }, { reason: error.message }));
      continue;
    }
    if (!value || typeof value !== 'object' || Array.isArray(value) || typeof value.id !== 'string' || !value.id) {
      warnings.push(diag('warning', 'schedule/change-invalid', '任务文件缺少任务标识，已跳过', { file: name }));
      continue;
    }
    if (value.id !== name.slice(0, -'.json'.length))
      warnings.push(diag('warning', 'schedule/id-mismatch', '任务标识与文件名不一致', { change: value.id, file: name }));
    changes.push(value);
  }
  return { changes, warnings };
}

/** 目录前缀覆盖子路径；与排程互斥判定保持一致。 */
const overlaps = (a, b, separator) => a === b || a.startsWith(b + separator) || b.startsWith(a + separator);
const normalizePath = (value) => String(value).replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();

function pairConflict(left, right) {
  const modules = new Set();
  const files = new Set();
  for (const a of left.modules) for (const b of right.modules) if (overlaps(a, b, '.')) modules.add(a.length <= b.length ? a : b);
  for (const a of left.files.map(normalizePath)) for (const b of right.files.map(normalizePath)) if (overlaps(a, b, '/')) files.add(a.length <= b.length ? a : b);
  const resources = left.resources.filter(name => right.resources.includes(name));
  if (!modules.size && !files.size && !resources.length) return null;
  return { left: left.id, right: right.id, modules: [...modules].sort(), files: [...files].sort(), resources: resources.sort() };
}

const mermaidLabel = (text) => String(text).replace(/[&<>"`\r\n]/g, ch => ({
  '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', '`': '&#96;', '\r': ' ', '\n': ' ',
}[ch]));

function mermaidOf(tasks) {
  const ids = new Map(tasks.map((task, index) => [task.id, 'task_' + index]));
  const lines = ['flowchart TD'];
  for (const task of tasks) lines.push('  ' + ids.get(task.id) + '["' + mermaidLabel(task.id + ' | ' + (task.title?.zh ?? task.id) + ' | ' + task.readiness) + '"]');
  for (const task of tasks) for (const dep of task.depends_on) if (ids.has(dep)) lines.push('  ' + ids.get(dep) + ' --> ' + ids.get(task.id));
  return lines.join('\n') + '\n';
}

/**
 * 仓库格式映射为排程契约。
 * readiness：终态与进行中原样；backlog（待办）与无排程记为 unscheduled；
 * proposed 的前置全部 verified 或不存在时 ready，否则 blocked。
 */
export function buildRepoPlan(changes, { warnings = [] } = {}) {
  const errors = [];
  const notes = [...warnings];
  const seen = new Set();
  const tasks = changes.map(change => {
    if (seen.has(change.id)) errors.push(diag('error', 'schedule/duplicate-id', '任务 ID 重复', { change: change.id }));
    seen.add(change.id);
    const fields = readSchedules(change);
    return {
      id: change.id,
      title: titleOf(change),
      status: change.status,
      ...(fields.phase !== undefined ? { phase: fields.phase } : {}),
      ...(fields.owner !== undefined ? { owner: fields.owner } : {}),
      depends_on: fields.dependsOn,
      modules: Array.isArray(change.modules?.modify) ? [...change.modules.modify] : [],
      files: fields.files,
      resources: fields.resources,
      priority: fields.priority,
      readiness: 'blocked',
      blockers: [],
      scheduled: fields.schedule !== undefined,
    };
  });
  tasks.sort((a, b) => a.priority - b.priority || a.id.localeCompare(b.id, 'en'));
  const scheduled = new Set(tasks.filter(task => task.scheduled).map(task => task.id));
  const byId = new Map(tasks.map(task => [task.id, task]));
  for (const task of tasks) {
    for (const dep of task.depends_on) {
      if (dep === task.id) {
        errors.push(diag('error', 'schedule/self-dependency', '任务不能依赖自身', { change: task.id }));
        task.blockers.push('dependency:' + dep);
      } else if (!byId.has(dep)) {
        // 仓库历史任务可能已归档；不存在的目标按已满足处理，但保留显式诊断。
        notes.push(diag('warning', 'schedule/dependency-missing', '前置任务不存在，按已满足处理', { change: task.id, dependency: dep }));
      } else if (byId.get(dep).status !== 'verified') {
        task.blockers.push('dependency:' + dep);
      }
    }
  }
  // 候选任务与运行任务争用写入范围时同样不可并行。
  const running = tasks.filter(task => task.status === 'in_progress');
  for (const task of tasks) {
    if (task.status !== 'proposed') continue;
    for (const other of running) if (pairConflict(task, other)) task.blockers.push('conflict:' + other.id);
  }
  for (const task of tasks) {
    if (task.status === 'verified' || task.status === 'abandoned' || task.status === 'in_progress') {
      task.readiness = task.status;
    } else if (task.status === 'backlog' || !task.scheduled) {
      task.readiness = 'unscheduled';
      task.blockers = [];
    } else if (task.status === 'proposed') {
      task.readiness = task.blockers.length ? 'blocked' : 'ready';
    } else {
      errors.push(diag('error', 'schedule/unknown-status', '任务状态无法识别', { change: task.id }, { status: task.status }));
      task.readiness = 'blocked';
    }
    delete task.scheduled;
  }
  const conflicts = [];
  const open = tasks.filter(task => OPEN.has(task.status));
  for (let i = 0; i < open.length; i++) for (let j = i + 1; j < open.length; j++) {
    const left = open[i];
    const right = open[j];
    if (!scheduled.has(left.id) && !scheduled.has(right.id)) continue;
    const conflict = pairConflict(left, right);
    if (conflict) conflicts.push(conflict);
  }
  const waves = [];
  if (!errors.length) {
    const done = new Set(tasks.filter(task => task.status === 'verified').map(task => task.id));
    const running = tasks.filter(task => task.status === 'in_progress');
    if (running.length) {
      waves.push(running.map(task => task.id));
      for (const task of running) done.add(task.id);
    }
    buildWaves(tasks.filter(task => scheduled.has(task.id)), byId, done, waves);
  }
  const plan = {
    schema_version: 1,
    ok: errors.length === 0,
    tasks,
    ready: tasks.filter(task => task.readiness === 'ready').map(task => task.id),
    running: tasks.filter(task => task.status === 'in_progress').map(task => task.id),
    blocked: tasks.filter(task => task.readiness === 'blocked').map(task => task.id),
    unscheduled: tasks.filter(task => task.readiness === 'unscheduled').map(task => task.id),
    completed: tasks.filter(task => task.status === 'verified').map(task => task.id),
    waves,
    conflicts,
    errors,
    warnings: notes,
    mermaid: '',
  };
  if (!plan.ok) {
    plan.ready = [];
    plan.waves = [];
    for (const task of plan.tasks) if (task.readiness === 'ready') {
      task.readiness = 'blocked';
      task.blockers.push('invalid-plan');
      plan.blocked.push(task.id);
    }
  }
  plan.mermaid = mermaidOf(plan.tasks);
  return plan;
}

/** 按真实前置逐波推进；同波内仍按文件/资源互斥拆分。 */
function buildWaves(tasks, byId, done, waves) {
  const pending = tasks.filter(task => task.status === 'proposed');
  while (pending.length) {
    const wave = [];
    for (const task of pending) {
      if (!task.depends_on.every(dep => !byId.has(dep) || done.has(dep))) continue;
      if (wave.some(other => pairConflict(other, task))) continue;
      wave.push(task);
    }
    if (!wave.length) return;
    waves.push(wave.map(task => task.id));
    for (const task of wave) {
      done.add(task.id);
      pending.splice(pending.indexOf(task), 1);
    }
  }
}

export class RepoReader {
  constructor(normifyDir) {
    this.dir = normifyDir;
    this.projectName = undefined;
  }
  /** 只读 [project].name；缺文件时返回 null。 */
  async project() {
    if (this.projectName !== undefined) return this.projectName;
    try {
      this.projectName = parseProjectName(await readFile(join(this.dir, 'config.toml'), 'utf8'));
    } catch {
      this.projectName = null;
    }
    return this.projectName;
  }
  /** 与 NormifyReader 相同的调用面：仅供只读排程查询。 */
  async call(name, args = {}) {
    if (name !== 'normify_change_list') throw new Error('拒绝非只读工具');
    const { changes, warnings } = await loadRepoChanges(this.dir);
    if (!changes.length) throw new Error('仓库内 .normify/changes 没有可读任务');
    if (args.id) return { ok: true, change: changes.find(change => change.id === args.id) ?? null };
    return buildRepoPlan(changes, { warnings });
  }
}
