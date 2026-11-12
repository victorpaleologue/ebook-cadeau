// Shared plumbing of the browser end-to-end tests: the demo constellation as a child
// process, the browser, screenshots, downloads, external print tools, a tiny ZIP reader,
// page-weight accounting and a small accessibility audit.
//
// Nothing here talks to the network beyond 127.0.0.1: every context aborts requests to
// other hosts and records them as problems.

import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:net';
import { createRequire } from 'node:module';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { inflateRawSync } from 'node:zlib';

const require = createRequire(import.meta.url);
// e2e/node_modules/playwright when `npm ci` was run in e2e/ (CI), else a global install
// reachable through NODE_PATH (the development container).
export const { chromium, devices } = require('playwright');

const here = dirname(fileURLToPath(import.meta.url));
export const E2E = resolve(here, '..');
export const ROOT = resolve(E2E, '..');
export const ARTIFACTS = resolve(process.env.EBC_E2E_ARTIFACTS || join(E2E, 'artifacts'));
mkdirSync(ARTIFACTS, { recursive: true });

const TARGET = resolve(ROOT, process.env.CARGO_TARGET_DIR || 'target');
/** Path of a debug binary of the workspace (overridable with EBC_E2E_<NAME>_BIN). */
export function binary(name) {
  const env = process.env[`EBC_E2E_${name.toUpperCase().replace(/-/g, '_')}_BIN`];
  const path = env || join(TARGET, 'debug', name);
  if (!existsSync(path)) {
    throw new Error(`${path} does not exist: run e2e/run.sh (it builds it), or cargo build the crate that provides "${name}"`);
  }
  return path;
}

// ------------------------------------------------------------------------------------
// Processes

const children = new Set();
function killAll() {
  for (const child of children) {
    try { child.kill('SIGKILL'); } catch { /* already gone */ }
  }
}
process.on('exit', killAll);

/** Starts a long-running process and resolves once `ready(stdout)` returns a value. */
function startProcess(name, command, args, env, ready, timeoutMs = 60_000) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { cwd: ROOT, env: { ...process.env, ...env }, stdio: ['ignore', 'pipe', 'pipe'] });
    children.add(child);
    let out = '';
    let err = '';
    let settled = false;
    const timer = setTimeout(() => finish(new Error(`${name} did not start within ${timeoutMs} ms.\nstdout:\n${out}\nstderr:\n${err}`)), timeoutMs);
    function finish(error, value) {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      if (error) {
        child.kill('SIGKILL');
        reject(error);
      } else {
        resolvePromise(value);
      }
    }
    child.stdout.on('data', (chunk) => {
      out += chunk;
      const value = ready(out);
      if (value) finish(null, { child, value, log: () => ({ out, err }) });
    });
    child.stderr.on('data', (chunk) => {
      err += chunk;
      if (err.length > 200_000) err = err.slice(-100_000);
    });
    child.on('exit', (code, signal) => {
      children.delete(child);
      finish(new Error(`${name} exited early (code ${code}, signal ${signal}).\nstdout:\n${out}\nstderr:\n${err}`));
    });
  });
}

async function stopProcess(child) {
  if (!child || child.exitCode !== null) return;
  const exited = new Promise((r) => child.once('exit', r));
  child.kill('SIGTERM');
  const timer = setTimeout(() => child.kill('SIGKILL'), 3_000);
  await exited;
  clearTimeout(timer);
}

/** A TCP port that was free a moment ago (for binaries that cannot report theirs). */
export function freePort() {
  return new Promise((resolvePromise, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => resolvePromise(port));
    });
  });
}

/**
 * Starts a fresh `ebc-demo` constellation on ports chosen by the operating system and
 * returns the base URLs it printed: `{ server, septante, grandeEnseigne, printer, shelf,
 * webmail, smtp, webApp, stop() }`. `webApp` is the URL of the card designer (`/app/`): the constellation's
 * own when it serves it, else a separate `ebc-server` started with `EBC_WEB_DIR` (see
 * the README: `ebc-demo` does not read `EBC_WEB_DIR` at the time of writing).
 */
export async function startConstellation({ webApp = false } = {}) {
  const env = {
    EBC_DEMO_SERVER_PORT: '0',
    EBC_DEMO_SEPTANTE_PORT: '0',
    EBC_DEMO_GRANDE_ENSEIGNE_PORT: '0',
    EBC_DEMO_PRINTER_PORT: '0',
    EBC_DEMO_SHELF_PORT: '0',
    EBC_DEMO_WEBMAIL_PORT: '0',
    EBC_DEMO_PRINTER_TICK_MS: '0',
    EBC_WEB_DIR: join(ROOT, 'web'),
    RUST_LOG: 'warn',
  };
  const started = await startProcess('ebc-demo', binary('ebc-demo'), [], env, (out) => {
    if (!out.includes('Ctrl-C to stop')) return null;
    const url = (label) => {
      const match = new RegExp(`${label}[^\\n]*?((?:http|smtp)://127\\.0\\.0\\.1:\\d+)`).exec(out);
      if (!match) throw new Error(`no URL for "${label}" in the ebc-demo banner:\n${out}`);
      return match[1];
    };
    return {
      server: url('eBook Cadeau \\(server\\)'),
      septante: url('Septante'),
      grandeEnseigne: url('Grande Enseigne'),
      printer: url('Imprimerie'),
      shelf: url('Étagère des autrices'),
      webmail: url('Webmail'),
      smtp: url('Mail-to-device'),
    };
  });
  const world = { ...started.value, notes: [], processes: [started.child] };
  world.stop = async () => {
    await Promise.all(world.processes.map(stopProcess));
  };
  if (webApp) {
    const probe = await fetch(`${world.server}/app/`).catch(() => null);
    if (probe && probe.ok) {
      world.webApp = `${world.server}/app/`;
    } else {
      world.notes.push(`ebc-demo answers ${probe ? probe.status : 'nothing'} on /app/ although EBC_WEB_DIR is set: using a separate ebc-server for the card designer`);
      const port = await freePort();
      const data = mkdtempSync(join(tmpdir(), 'ebc-e2e-data-'));
      // ebc-server logs to stderr only: poll its port instead of reading a banner.
      const child = spawn(binary('ebc-server'), [], {
        cwd: ROOT,
        env: { ...process.env, EBC_BIND: `127.0.0.1:${port}`, EBC_PUBLIC_URL: `http://127.0.0.1:${port}`, EBC_DATA_DIR: data, EBC_WEB_DIR: join(ROOT, 'web'), RUST_LOG: 'warn' },
        stdio: ['ignore', 'ignore', 'pipe'],
      });
      children.add(child);
      let stderr = '';
      child.stderr.on('data', (chunk) => { stderr += chunk; });
      child.on('exit', () => children.delete(child));
      world.processes.push(child);
      const base = `http://127.0.0.1:${port}`;
      const deadline = Date.now() + 30_000;
      for (;;) {
        const answer = await fetch(`${base}/app/`).catch(() => null);
        if (answer && answer.ok) break;
        if (child.exitCode !== null || Date.now() > deadline) {
          throw new Error(`ebc-server with EBC_WEB_DIR did not serve /app/ (exit ${child.exitCode}):\n${stderr}`);
        }
        await new Promise((r) => setTimeout(r, 100));
      }
      const stop = world.stop;
      world.stop = async () => {
        await stop();
        rmSync(data, { recursive: true, force: true });
      };
      world.webApp = `${base}/app/`;
    }
  }
  return world;
}

// ------------------------------------------------------------------------------------
// Browser

/** One Chromium for the whole file. */
export async function launchBrowser() {
  return chromium.launch({ args: ['--disable-gpu'] });
}

/**
 * A fresh context with downloads accepted, deviceScaleFactor 1 (screenshots are
 * comparable from run to run), requests to anything but loopback aborted and recorded,
 * console errors and page errors recorded in `problems`.
 */
export async function newContext(browser, options = {}) {
  const context = await browser.newContext({ acceptDownloads: true, locale: 'fr-FR', ...options, deviceScaleFactor: 1 });
  context.setDefaultTimeout(20_000);
  context.setDefaultNavigationTimeout(30_000);
  const problems = [];
  await context.route('**/*', (route) => {
    const url = route.request().url();
    if (/^https?:\/\/127\.0\.0\.1[:/]/.test(url) || url.startsWith('blob:') || url.startsWith('data:')) return route.continue();
    problems.push(`request to another host: ${url}`);
    return route.abort();
  });
  context.on('page', (page) => watchPage(page, problems));
  return { context, problems };
}

function watchPage(page, problems) {
  page.on('pageerror', (error) => problems.push(`page error at ${page.url()}: ${error.message}`));
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(`console error at ${page.url()} (resource ${message.location().url}): ${message.text()}`);
  });
}

/** A full-page screenshot into the artifacts directory; returns its path. */
export async function shot(page, name) {
  const path = join(ARTIFACTS, `${name}.png`);
  await page.screenshot({ path, fullPage: true, animations: 'disabled' });
  return path;
}

/** Clicks `selector` and waits for the download it starts; saves it under artifacts. */
export async function download(page, selector, name) {
  const [file] = await Promise.all([
    page.waitForEvent('download', { timeout: 30_000 }),
    page.click(selector),
  ]);
  const failure = await file.failure();
  if (failure) throw new Error(`the download started by ${selector} failed: ${failure}`);
  const path = join(ARTIFACTS, name || file.suggestedFilename());
  await file.saveAs(path);
  return { path, bytes: readFileSync(path), suggested: file.suggestedFilename(), url: file.url() };
}

/** Text content of a selector, trimmed (fails clearly when it is absent). */
export async function text(page, selector) {
  const locator = page.locator(selector);
  if ((await locator.count()) === 0) {
    throw new Error(`${page.url()} has no ${selector}; page text: ${(await page.locator('body').innerText()).slice(0, 400)}`);
  }
  return (await locator.first().textContent()).trim();
}

// ------------------------------------------------------------------------------------
// Page weight (the e-reader budget of docs/api.md §2: HTML + same-origin CSS < 30 kB)

/**
 * Records the bytes of every document, style sheet and image the page loads; returns a
 * function giving `{ document, css, images, total, scripts }` for the last navigation.
 */
export function weigh(page) {
  let current = { document: 0, css: 0, images: 0, scripts: 0, other: 0, urls: [] };
  page.on('response', async (response) => {
    const request = response.request();
    if (request.isNavigationRequest() && request.frame() === page.mainFrame()) {
      current = { document: 0, css: 0, images: 0, scripts: 0, other: 0, urls: [] };
    }
    let size = 0;
    try { size = (await response.body()).length; } catch { return; /* redirects, downloads */ }
    const type = request.resourceType();
    const key = type === 'document' ? 'document' : type === 'stylesheet' ? 'css' : type === 'image' ? 'images' : type === 'script' ? 'scripts' : 'other';
    current[key] += size;
    current.urls.push(`${type} ${size} ${request.url()}`);
  });
  return () => ({ ...current, total: current.document + current.css });
}

// ------------------------------------------------------------------------------------
// External tools

/**
 * Whether `tool` can be started. With EBC_E2E_REQUIRE_TOOLS=1 (CI) a missing tool is an
 * error instead of a skipped check.
 */
export function hasTool(tool) {
  const found = ['-v', '--version', '-h'].some((flag) => {
    const run = spawnSync(tool, [flag], { stdio: 'ignore' });
    return !run.error;
  });
  if (!found && process.env.EBC_E2E_REQUIRE_TOOLS === '1') {
    throw new Error(`${tool} is not installed and EBC_E2E_REQUIRE_TOOLS=1 (run scripts/install-print-tools.sh)`);
  }
  return found;
}

/** Runs a command, returns `{ status, stdout, stderr }`. */
export function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: ROOT, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, ...options });
  if (result.error) throw result.error;
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

/**
 * QR payloads of one page of a PDF (pdftoppm at 300 dpi, then zbarimg), or `null` when a
 * tool is missing. The rendered page is kept under artifacts.
 */
export function qrPayloads(pdf, page, name) {
  if (!hasTool('pdftoppm') || !hasTool('zbarimg')) return null;
  const prefix = join(ARTIFACTS, `${name}-page${page}`);
  const rendered = run('pdftoppm', ['-r', '300', '-f', String(page), '-l', String(page), '-png', '-singlefile', pdf, prefix]);
  if (rendered.status !== 0) throw new Error(`pdftoppm failed: ${rendered.stderr}`);
  const decoded = run('zbarimg', ['--raw', '-q', `${prefix}.png`]);
  // zbarimg exits with 4 when it finds nothing.
  return decoded.stdout.split('\n').map((line) => line.trim()).filter(Boolean);
}

/** Text of one PDF page (pdftotext -layout), or `null` without the tool. */
export function pdfText(pdf, page) {
  if (!hasTool('pdftotext')) return null;
  const result = run('pdftotext', ['-f', String(page), '-l', String(page), '-layout', pdf, '-']);
  if (result.status !== 0) throw new Error(`pdftotext failed: ${result.stderr}`);
  return result.stdout;
}

// ------------------------------------------------------------------------------------
// ZIP (just enough to read an EPUB's entries)

/** The entries of a ZIP archive: `Map<name, Buffer>` (stored and deflated only). */
export function unzip(buffer) {
  const entries = new Map();
  let eocd = -1;
  for (let i = buffer.length - 22; i >= Math.max(0, buffer.length - 65_557); i -= 1) {
    if (buffer.readUInt32LE(i) === 0x06054b50) { eocd = i; break; }
  }
  if (eocd < 0) throw new Error('not a ZIP archive (no end of central directory)');
  const count = buffer.readUInt16LE(eocd + 10);
  let offset = buffer.readUInt32LE(eocd + 16);
  for (let n = 0; n < count; n += 1) {
    if (buffer.readUInt32LE(offset) !== 0x02014b50) throw new Error('corrupt central directory');
    const method = buffer.readUInt16LE(offset + 10);
    const compressed = buffer.readUInt32LE(offset + 20);
    const nameLength = buffer.readUInt16LE(offset + 28);
    const extraLength = buffer.readUInt16LE(offset + 30);
    const commentLength = buffer.readUInt16LE(offset + 32);
    const local = buffer.readUInt32LE(offset + 42);
    const name = buffer.subarray(offset + 46, offset + 46 + nameLength).toString('utf8');
    const localData = local + 30 + buffer.readUInt16LE(local + 26) + buffer.readUInt16LE(local + 28);
    const raw = buffer.subarray(localData, localData + compressed);
    entries.set(name, method === 0 ? Buffer.from(raw) : method === 8 ? inflateRawSync(raw) : null);
    offset += 46 + nameLength + extraLength + commentLength;
  }
  return entries;
}

/** Checks the shape of an EPUB and returns its entries (throws with the reason). */
export function readEpub(bytes) {
  if (bytes.subarray(0, 4).toString('latin1') !== 'PK\u0003\u0004') throw new Error('the file is not a ZIP archive');
  if (bytes.subarray(30, 38).toString('latin1') !== 'mimetype' || bytes.subarray(38, 58).toString('latin1') !== 'application/epub+zip') {
    throw new Error('the first entry is not an uncompressed "mimetype" = application/epub+zip');
  }
  const entries = unzip(bytes);
  if (!entries.has('META-INF/container.xml')) throw new Error('no META-INF/container.xml');
  return entries;
}

// ------------------------------------------------------------------------------------
// Accessibility: a small, dependency-free audit (not axe; see the README for its limits)

/**
 * Audits the current page: `lang`, a title, labelled form controls, named buttons and
 * links, `alt` on images, and the contrast of visible text against its effective
 * background (WCAG 2 formula; 4.5:1, or 3:1 for large text). Works with JavaScript
 * disabled (Playwright evaluates in its own world). Returns a list of findings.
 */
export async function audit(page) {
  return page.evaluate(() => {
    const findings = [];
    const describe = (el) => {
      const id = el.id ? `#${el.id}` : '';
      const cls = el.classList && el.classList.length ? `.${[...el.classList].join('.')}` : '';
      const label = (el.textContent || el.getAttribute('alt') || el.getAttribute('name') || '').trim().replace(/\s+/g, ' ').slice(0, 40);
      return `<${el.tagName.toLowerCase()}${id}${cls}> "${label}"`;
    };
    const visible = (el) => {
      if (el.closest('[hidden]') || el.closest('[aria-hidden="true"]')) return false;
      const style = getComputedStyle(el);
      if (style.display === 'none' || style.visibility === 'hidden') return false;
      const box = el.getBoundingClientRect();
      return box.width > 0 && box.height > 0;
    };
    const lang = document.documentElement.getAttribute('lang');
    if (!lang || !lang.trim()) findings.push('html has no lang');
    if (!document.title.trim()) findings.push('the document has no title');

    const nameOf = (el) => {
      if (el.getAttribute('aria-label')?.trim()) return el.getAttribute('aria-label').trim();
      const ids = el.getAttribute('aria-labelledby');
      if (ids) {
        const joined = ids.split(/\s+/).map((id) => document.getElementById(id)?.textContent || '').join(' ').trim();
        if (joined) return joined;
      }
      if (el.labels && el.labels.length) return [...el.labels].map((l) => l.textContent).join(' ').trim();
      if (el.getAttribute('title')?.trim()) return el.getAttribute('title').trim();
      return '';
    };
    for (const el of document.querySelectorAll('input, select, textarea')) {
      if (['hidden', 'submit', 'button', 'reset'].includes(el.type) || !visible(el)) continue;
      if (!nameOf(el)) findings.push(`form control without a label: ${describe(el)}`);
    }
    for (const el of document.querySelectorAll('button, a[href], input[type=submit], input[type=button]')) {
      if (!visible(el)) continue;
      const own = (el.textContent || '').trim() || el.value || [...el.querySelectorAll('img[alt]')].map((img) => img.alt).join('').trim();
      if (!own && !nameOf(el)) findings.push(`${el.tagName.toLowerCase()} without an accessible name: ${describe(el)}`);
    }
    for (const img of document.querySelectorAll('img')) {
      if (!img.hasAttribute('alt')) findings.push(`img without alt: ${img.getAttribute('src')?.slice(0, 60)}`);
    }

    // Contrast.
    const parse = (value) => {
      const m = /^rgba?\(\s*([\d.]+)[,\s]+([\d.]+)[,\s]+([\d.]+)(?:\s*[,/]\s*([\d.]+%?))?\s*\)$/.exec(value);
      if (!m) return null;
      let a = m[4] === undefined ? 1 : m[4].endsWith('%') ? parseFloat(m[4]) / 100 : parseFloat(m[4]);
      if (Number.isNaN(a)) a = 1;
      return [parseFloat(m[1]), parseFloat(m[2]), parseFloat(m[3]), a];
    };
    const over = (top, bottom) => {
      const a = top[3];
      return [0, 1, 2].map((i) => top[i] * a + bottom[i] * (1 - a)).concat(1);
    };
    const background = (el) => {
      const layers = [];
      for (let node = el; node && node.nodeType === 1; node = node.parentElement) {
        const style = getComputedStyle(node);
        if (style.backgroundImage && style.backgroundImage !== 'none') return null; // unknown
        const color = parse(style.backgroundColor);
        if (color && color[3] > 0) {
          layers.push(color);
          if (color[3] >= 1) break;
        }
      }
      let result = [255, 255, 255, 1];
      for (const layer of layers.reverse()) result = over(layer, result);
      return result;
    };
    const luminance = (rgb) => {
      const c = rgb.slice(0, 3).map((v) => {
        const s = v / 255;
        return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
      });
      return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    };
    const seen = new Set();
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      if (!node.textContent.trim()) continue;
      const el = node.parentElement;
      if (!el || seen.has(el) || !visible(el) || ['SCRIPT', 'STYLE', 'NOSCRIPT', 'OPTION'].includes(el.tagName)) continue;
      // WCAG 1.4.3 exempts inactive controls.
      if (el.closest(':disabled, [aria-disabled="true"]')) continue;
      seen.add(el);
      const style = getComputedStyle(el);
      let fg = parse(style.color);
      const bg = background(el);
      if (!fg || !bg) continue;
      let opacity = 1;
      for (let n = el; n && n.nodeType === 1; n = n.parentElement) opacity *= parseFloat(getComputedStyle(n).opacity || '1');
      fg = over([fg[0], fg[1], fg[2], fg[3] * opacity], bg);
      const [l1, l2] = [luminance(fg), luminance(bg)].sort((a, b) => b - a);
      const ratio = (l1 + 0.05) / (l2 + 0.05);
      const size = parseFloat(style.fontSize);
      const bold = parseInt(style.fontWeight, 10) >= 700;
      const large = size >= 24 || (bold && size >= 18.66);
      const needed = large ? 3 : 4.5;
      if (ratio + 1e-6 < needed) {
        findings.push(`contrast ${ratio.toFixed(2)}:1 < ${needed}:1 for ${describe(el)} (${style.color} on rgb(${bg.slice(0, 3).map(Math.round).join(', ')}), ${size}px)`);
      }
    }
    return findings;
  });
}

/** Horizontal overflow of the page in CSS pixels (0 when nothing scrolls sideways). */
export async function overflowX(page) {
  return page.evaluate(() => Math.max(0, document.documentElement.scrollWidth - window.innerWidth));
}

/** A recorder of known product defects: logged, not failed (each says why). */
export function known(t, condition, message) {
  if (!condition) {
    t.diagnostic(`KNOWN DEFECT (not failing the suite): ${message}`);
    console.log(`  KNOWN DEFECT: ${message}`);
  }
}
