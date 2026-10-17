import http from 'node:http';
import { spawn, execFileSync } from 'node:child_process';
import { createInterface } from 'node:readline';
import { readFile, realpath } from 'node:fs/promises';
import { dirname, resolve, relative, isAbsolute, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { randomBytes } from 'node:crypto';
import { validateSchedule } from './model.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, '../..');
const allowedTools = new Set(['normify_change_list']);

// 独立只读进程复用原引擎。网页不写任务文件。
export class NormifyReader {
  constructor(entry, root, { timeout = 20000 } = {}) {
    this.pending = new Map();
    this.sequence = 0;
    this.timeout = timeout;
    this.child = spawn(process.execPath, [resolve(entry), '--root', resolve(root), '--read-only'], {
      stdio: ['pipe', 'pipe', 'pipe'], windowsHide: true,
    });
    createInterface({ input: this.child.stdout }).on('line', line => {
      try {
        const message = JSON.parse(line);
        const pending = this.pending.get(message.id);
        if (!pending) return;
        clearTimeout(pending.timer);
        this.pending.delete(message.id);
        if (message.error) pending.reject(new Error(message.error.message));
        else pending.resolve(message.result);
      } catch { this.fail(new Error('只读服务返回无效协议')); }
    });
    this.child.stderr.on('data', data => process.stderr.write(data));
    this.child.on('error', error => this.fail(error));
    this.child.on('exit', () => this.fail(new Error('只读服务已退出')));
    this.child.stdin.on('error', error => this.fail(error));
  }
  fail(error) {
    this.failure = error;
    for (const pending of this.pending.values()) { clearTimeout(pending.timer); pending.reject(error); }
    this.pending.clear();
  }
  request(method, params) {
    if (this.failure) return Promise.reject(this.failure);
    return new Promise((resolvePromise, reject) => {
      const id = ++this.sequence;
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error('读取排程超时'));
      }, this.timeout);
      this.pending.set(id, { resolve: resolvePromise, reject, timer });
      this.child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id, method, params }) + '\n');
    });
  }
  async initialize() {
    await this.request('initialize', { protocolVersion: '2024-11-05', capabilities: {},
      clientInfo: { name: 'mjwarp-development-overview', version: '1.0.0' } });
    this.child.stdin.write(JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }) + '\n');
  }
  async call(name, args) {
    if (!allowedTools.has(name)) throw new Error('拒绝非只读工具');
    const result = await this.request('tools/call', { name, arguments: args });
    if (result.isError) throw new Error(result.content?.find(c => c.type === 'text')?.text || '读取失败');
    const value = result.structuredContent ?? JSON.parse(result.content.find(c => c.type === 'text').text);
    if (value.ok === false) throw new Error(JSON.stringify(value));
    return value;
  }
  close() { this.fail(new Error('服务已关闭')); this.child.kill(); }
}

export async function readReceipt(change, root) {
  const match = change.note?.match(/delivery-receipt:\s*(docs\/development-evidence\/changes\/[a-z0-9-]+\.json)/);
  if (!match) return { status: '未绑定回执' };
  const path = match[1];
  try {
    const base = await realpath(root);
    const actual = await realpath(resolve(base, path));
    const rel = relative(base, actual);
    if (rel.startsWith(`..${sep}`) || rel === '..' || isAbsolute(rel)) throw new Error('回执越出仓库');
    const receipt = JSON.parse(await readFile(actual, 'utf8'));
    if (receipt.task_id !== change.id) throw new Error('回执任务不匹配');
    return { status: '已读取；未复验', path, receipt };
  } catch (error) {
    if (error.code === 'ENOENT') return { status: '尚无回执文件', path };
    return { status: '回执读取失败', path, error: error.message };
  }
}

export function createOverviewServer(reader, { project = 'mjwarp-rs', root = repoRoot, token = randomBytes(24).toString('hex') } = {}) {
  const basePath = `/${token}/`;
  const server = http.createServer(async (request, response) => {
    const host = `127.0.0.1:${server.address().port}`;
    const origin = `http://${host}`;
    response.setHeader('Cache-Control', 'no-store');
    response.setHeader('X-Content-Type-Options', 'nosniff');
    response.setHeader('Referrer-Policy', 'no-referrer');
    response.setHeader('Content-Security-Policy', "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'none'");
    const send = (status, type, body) => { response.writeHead(status, { 'Content-Type': type }); response.end(body); };
    const json = (status, value) => send(status, 'application/json; charset=utf-8', JSON.stringify(value));
    const url = new URL(request.url, origin);
    // 允许用户从其他页面打开空白外壳，数据接口仍拒绝跨站。
    const topNavigation = url.pathname === basePath && request.headers['sec-fetch-mode'] === 'navigate'
      && request.headers['sec-fetch-dest'] === 'document';
    if (request.headers.host !== host || (request.headers.origin && request.headers.origin !== origin)
      || (request.headers['sec-fetch-site'] === 'cross-site' && !topNavigation)) return json(403, { error: '只允许本机同源读取' });
    if (request.method !== 'GET') return json(405, { error: '仅支持只读请求' });
    if (!url.pathname.startsWith(basePath)) return json(404, { error: '地址失效' });
    const route = url.pathname.slice(basePath.length);
    try {
      if (route === 'api/schedule') {
        const schedule = validateSchedule(await reader.call('normify_change_list', { project, view: 'schedule' }));
        let head = '无法读取';
        try { head = execFileSync('git', ['-C', root, 'rev-parse', 'HEAD'], { encoding: 'utf8', windowsHide: true }).trim(); } catch { /* 不用未知提交冒充证据。 */ }
        return json(200, { schedule, fetchedAt: new Date().toISOString(), project, head });
      }
      if (route === 'api/task') {
        const id = url.searchParams.get('id') || '';
        if (!/^[a-z0-9][a-z0-9-]{0,199}$/.test(id)) return json(400, { error: '任务标识无效' });
        const value = await reader.call('normify_change_list', { project, id });
        if (!value.change) return json(404, { error: '未找到任务' });
        return json(200, { change: value.change, delivery: await readReceipt(value.change, root) });
      }
      const assets = { '': ['index.html', 'text/html'], 'app.mjs': ['app.mjs', 'text/javascript'],
        'model.mjs': ['model.mjs', 'text/javascript'], 'style.css': ['style.css', 'text/css'] };
      if (!Object.hasOwn(assets, route)) return json(404, { error: '未找到资源' });
      const [file, type] = assets[route];
      send(200, `${type}; charset=utf-8`, await readFile(resolve(here, file)));
    } catch (error) { json(503, { error: error.message }); }
  });
  return { server, basePath };
}

async function main() {
  const { values } = parseArgs({ options: { 'normify-server': { type: 'string' },
    'normify-root': { type: 'string' }, port: { type: 'string', default: '0' } } });
  const entry = values['normify-server'] || process.env.NORMIFY_SERVER;
  if (!entry) throw new Error('请用 --normify-server 指定已安装的 stdio.mjs');
  const root = values['normify-root'] || dirname(execFileSync('git', ['-C', repoRoot, 'config', '--get', 'normify.directory'], { encoding: 'utf8', windowsHide: true }).trim());
  const port = Number(values.port);
  if (!Number.isInteger(port) || port < 0 || port > 65535) throw new Error('端口无效');
  const reader = new NormifyReader(entry, root);
  try {
    await reader.initialize();
    const { server, basePath } = createOverviewServer(reader);
    server.on('error', error => { console.error(error.message); reader.close(); process.exitCode = 1; });
    server.listen(port, '127.0.0.1', () => console.log(`http://127.0.0.1:${server.address().port}${basePath}`));
    for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => { reader.close(); server.close(); });
  } catch (error) { reader.close(); throw error; }
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
