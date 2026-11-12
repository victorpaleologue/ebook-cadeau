// Static checks of the web app's sources:  node --test web/tests/
//
// * no external URL: the app loads nothing from another origin by itself (no CDN, no
//   font service, no analytics). Allowed: XML namespaces, loopback hosts and the
//   reserved `.example` domain;
// * no inline script or event-handler attribute (the server's Content-Security-Policy
//   forbids them), no `innerHTML`-style sink (book metadata is attacker-controlled).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, dirname, extname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = join(dirname(fileURLToPath(import.meta.url)), '..');
const TEXT = new Set(['.html', '.js', '.mjs', '.css', '.json', '.svg', '.webmanifest']);
const SKIP_DIRS = new Set(['tests', 'node_modules']);

function files(dir = web) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) {
      if (!SKIP_DIRS.has(name)) out.push(...files(path));
    } else if (TEXT.has(extname(name))) {
      out.push(path);
    }
  }
  return out;
}

const ALLOWED = [
  /^https?:\/\/www\.w3\.org\//,
  /^https?:\/\/(127\.0\.0\.1|localhost)(:[0-9*]+)?([/;]|$)/,
  /^https?:\/\/[a-z0-9.-]+\.example(:[0-9]+)?(\/|$)/,
];

test('the app has sources to check', () => {
  const list = files().map((f) => relative(web, f));
  for (const expected of ['index.html', 'app.js', 'worker.js', 'style.css', 'pkg/ebc_wasm.js']) {
    assert.ok(list.includes(expected), `${expected} missing (build the package first)`);
  }
});

test('no external URL in the web app', () => {
  const offenders = [];
  for (const file of files()) {
    const text = readFileSync(file, 'utf8');
    for (const match of text.matchAll(/\b(?:https?|wss?|ftp):\/\/[^\s"'`)<>\\]*/gi)) {
      if (!ALLOWED.some((re) => re.test(match[0]))) offenders.push(`${relative(web, file)}: ${match[0]}`);
    }
    // Protocol-relative references in markup or CSS.
    for (const match of text.matchAll(/(?:src|href|action)\s*=\s*["']\s*\/\/[^"']*|url\(\s*["']?\/\/[^)]*/gi)) {
      offenders.push(`${relative(web, file)}: ${match[0]}`);
    }
    for (const match of text.matchAll(/@import\s+(?:url\()?["']?(?:https?:)?\/\//gi)) {
      offenders.push(`${relative(web, file)}: ${match[0]}`);
    }
  }
  assert.deepEqual(offenders, []);
});

test('no inline script, no inline event handler, no HTML sink', () => {
  const offenders = [];
  for (const file of files()) {
    const name = relative(web, file);
    const text = readFileSync(file, 'utf8');
    if (name.endsWith('.html')) {
      for (const match of text.matchAll(/<script\b([^>]*)>/gi)) {
        if (!/\bsrc=/.test(match[1])) offenders.push(`${name}: inline <script>`);
      }
      for (const match of text.matchAll(/<[^>]+\s(on[a-z]+)\s*=/gi)) offenders.push(`${name}: ${match[1]} attribute`);
      if (/javascript:/i.test(text)) offenders.push(`${name}: javascript: URL`);
    }
    if ((name.endsWith('.js') || name.endsWith('.mjs')) && !name.startsWith('pkg/')) {
      for (const sink of ['innerHTML', 'outerHTML', 'insertAdjacentHTML', 'document.write', 'eval(', 'new Function']) {
        if (text.includes(sink)) offenders.push(`${name}: ${sink}`);
      }
      if (/setAttribute\(\s*['"]style['"]/.test(text)) offenders.push(`${name}: style attribute (CSP)`);
    }
  }
  assert.deepEqual(offenders, []);
});
