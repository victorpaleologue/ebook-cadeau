// Smoke test of the built WebAssembly package (web/pkg), in Node, without any npm
// dependency:  node --test web/tests/
//
// Build the package first with scripts/build-wasm.sh. Checks that need external tools
// run only when the tools are on PATH: `pdftoppm` + `zbarimg` (QR decoding from the
// rendered PDF) and the `ebc` command-line tool (preflight; found on PATH or in
// target/debug or target/release).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, mkdtempSync, readFileSync, readdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync, spawnSync } from 'node:child_process';

const here = dirname(fileURLToPath(import.meta.url));
const web = join(here, '..');
const root = join(web, '..');
const pkgDir = join(web, 'pkg');
const fixtures = join(root, 'crates', 'ebc-wasm', 'tests', 'fixtures');

const pkg = await import(join(pkgDir, 'ebc_wasm.js'));
pkg.initSync({ module: readFileSync(join(pkgDir, 'ebc_wasm_bg.wasm')) });

const json = (text) => JSON.parse(text);

function which(tool) {
  const found = spawnSync('sh', ['-c', `command -v ${tool}`], { encoding: 'utf8' });
  return found.status === 0 ? found.stdout.trim() : null;
}

function ebcBinary() {
  for (const candidate of [join(root, 'target', 'debug', 'ebc'), join(root, 'target', 'release', 'ebc')]) {
    if (existsSync(candidate)) return candidate;
  }
  return which('ebc');
}

const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');

function cardRequest(patch = {}) {
  return {
    book: { title: 'Les Vagues du soir', contributors: [{ name: 'Jeanne Roy' }], language: 'fr' },
    code: 'BCDF-GHJK-LMNL',
    base_url: 'https://ebk.example',
    giver: 'Alice',
    recipient: 'Bob',
    message: 'Bonne lecture !',
    locale: 'fr',
    format: 'poche',
    profile: 'home_a4',
    ...patch,
  };
}

/** Calls a package function and returns the "[code]" of the error it throws. */
function errorCode(fn) {
  try {
    fn();
  } catch (error) {
    const match = /^\[([a-z0-9_]+)\]/.exec(error.message);
    return match ? match[1] : error.message;
  }
  return null;
}

test('the package loads and answers', () => {
  assert.match(pkg.version(), /^\d+\.\d+\.\d+/);
  const isbn = json(pkg.isbn_parse('978-2-07-036002-4'));
  assert.equal(isbn.isbn13, '9782070360024');
  assert.equal(isbn.isbn10, '2070360024');
  assert.equal(errorCode(() => pkg.isbn_parse('978-2-07-036002-5')), 'invalid_isbn');
  const svg = pkg.qr_svg('HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNL');
  assert.ok(svg.startsWith('<svg'), svg.slice(0, 40));
  assert.ok(svg.includes('</svg>'));
  const code = json(pkg.gift_code_parse('bcdf ghjk lmnl'));
  assert.equal(code.display, 'BCDF-GHJK-LMNL');
  assert.equal(errorCode(() => pkg.gift_code_parse('BCDF-GHJK-LMNB')), 'invalid_code');
  const random = json(pkg.gift_code_from_random(new Uint8Array(16).fill(7)));
  assert.match(random.code, /^[BCDFGHJKLMNPQRSTVWXZ]{12}$/);
});

test('render_card writes a PDF, the preview two SVG pages', () => {
  const card = pkg.render_card(JSON.stringify(cardRequest()), null, null);
  const pdf = card.pdf;
  const warnings = json(card.warnings);
  card.free();
  assert.ok(pdf instanceof Uint8Array);
  assert.equal(Buffer.from(pdf.subarray(0, 5)).toString('latin1'), '%PDF-');
  assert.ok(pdf.length > 10_000, `${pdf.length} bytes`);
  assert.deepEqual(warnings, []);
  const preview = json(pkg.preview_card(JSON.stringify(cardRequest()), null));
  assert.equal(preview.pages.length, 2);
  for (const page of preview.pages) {
    assert.ok(page.startsWith('<svg') && page.endsWith('</svg>'));
    assert.ok(!/<script|onload=|onerror=/i.test(page));
  }
  // The rights gate holds in the package too: a page cannot claim a reseller's licence.
  const tiny = readFileSync(join(fixtures, 'golden-cover.png'));
  const resellers = JSON.stringify(cardRequest({ cover: { rights: 'print_licence' } }));
  assert.equal(errorCode(() => pkg.render_card(resellers, tiny, null)), 'rights_basis_not_available');
  const privateCopyPro = JSON.stringify(cardRequest({
    profile: 'upload_eu', created_unix: 1_780_000_000, cover: { rights: 'user_private_copy', declaration: 'private_copy' },
  }));
  assert.equal(errorCode(() => pkg.render_card(privateCopyPro, tiny, null)), 'rights_route_refused');
});

test('the browser writes the very bytes the native library writes (golden cards)', (t) => {
  const digestsFile = join(pkgDir, 'golden-native.sha256');
  if (!existsSync(digestsFile)) {
    t.skip('web/pkg/golden-native.sha256 is missing: run scripts/build-wasm.sh');
    return;
  }
  const expected = new Map(readFileSync(digestsFile, 'utf8').trim().split('\n')
    .map((line) => line.split(/\s+/)).map(([digest, name]) => [name, digest]));
  const cases = [['typographic', false, false], ['cover', true, false], ['pdfx', false, true]];
  const icc = readFileSync(join(root, 'assets', 'icc', 'CGATS001Compat-v2-micro.icc'));
  const cover = readFileSync(join(fixtures, 'golden-cover.png'));
  for (const [name, withCover, withIcc] of cases) {
    const request = readFileSync(join(fixtures, `golden-${name}.json`), 'utf8');
    const card = pkg.render_card(request, withCover ? cover : null, withIcc ? icc : null);
    const digest = sha256(card.pdf);
    card.free();
    assert.equal(digest, expected.get(name), `golden card "${name}" differs from the native rendering`);
  }
});

test('the QR code of the PDF decodes to the upper-case claim URL', (t) => {
  if (!which('pdftoppm') || !which('zbarimg')) {
    t.skip('pdftoppm or zbarimg not on PATH');
    return;
  }
  const urls = json(pkg.claim_urls('https://ebk.example', 'BCDF-GHJK-LMNL'));
  assert.equal(urls.qr_payload, urls.claim_url.toUpperCase());
  const card = pkg.render_card(JSON.stringify(cardRequest()), null, null);
  const dir = mkdtempSync(join(tmpdir(), 'ebc-smoke-'));
  try {
    const pdfPath = join(dir, 'card.pdf');
    writeFileSync(pdfPath, card.pdf);
    card.free();
    execFileSync('pdftoppm', ['-r', '200', '-gray', '-png', pdfPath, join(dir, 'page')]);
    const pages = readdirSync(dir).filter((f) => f.endsWith('.png')).map((f) => join(dir, f));
    assert.ok(pages.length >= 1);
    const decoded = spawnSync('zbarimg', ['--quiet', '--raw', '-Sdisable', '-Sqrcode.enable', ...pages], { encoding: 'utf8' });
    const found = decoded.stdout.split('\n').map((s) => s.trim()).filter(Boolean);
    assert.deepEqual(found, ['HTTPS://EBK.EXAMPLE/K/BCDFGHJKLMNL']);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('the PDF passes the preflight of the command-line tool', (t) => {
  const ebc = ebcBinary();
  if (!ebc) {
    t.skip('the ebc command-line tool is not built');
    return;
  }
  const dir = mkdtempSync(join(tmpdir(), 'ebc-preflight-'));
  try {
    for (const [profile, cliProfile, format] of [['home_a4', 'home-a4', 'poche'], ['home_letter', 'home-letter', 'a6']]) {
      const card = pkg.render_card(JSON.stringify(cardRequest({ profile, format })), null, null);
      const pdfPath = join(dir, `${profile}.pdf`);
      writeFileSync(pdfPath, card.pdf);
      card.free();
      const run = spawnSync(ebc, ['preflight', '--profile', cliProfile, '--format', format, pdfPath], { encoding: 'utf8' });
      assert.equal(run.status, 0, `${profile}/${format}:\n${run.stdout}\n${run.stderr}`);
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('every string the page uses exists in every language', () => {
  const html = readFileSync(join(web, 'index.html'), 'utf8');
  const scripts = ['app.js', ...readdirSync(join(web, 'js')).map((f) => join('js', f))]
    .map((f) => readFileSync(join(web, f), 'utf8')).join('\n');
  const keys = new Set();
  for (const match of html.matchAll(/data-i18n="([a-z0-9_]+)"/g)) keys.add(match[1]);
  for (const match of html.matchAll(/data-i18n-attr="([^"]+)"/g)) {
    for (const pair of match[1].split(';')) keys.add(pair.split(':')[1].trim());
  }
  for (const match of scripts.matchAll(/\bt\('([a-z0-9_]+)'/g)) keys.add(match[1]);
  const options = json(pkg.card_options());
  for (const r of options.rights) keys.add(`ui_rights_${r.id}`);
  for (const p of options.profiles) keys.add(`ui_profile_${p.name}`);
  for (const v of ['excellent', 'acceptable', 'low', 'insufficient']) keys.add(`ui_quality_${v}`);
  for (const d of ['top_to_bottom', 'bottom_to_top']) keys.add(`ui_direction_${d}`);
  assert.ok(keys.size > 80, `${keys.size} keys`);
  const locales = options.locales.map((l) => l.code);
  assert.deepEqual(locales, ['fr', 'en', 'de', 'es', 'it', 'nl', 'pt']);
  for (const locale of locales) {
    const catalog = json(pkg.i18n_catalog(locale));
    const missing = [...keys].filter((key) => !catalog[key]);
    assert.deepEqual(missing, [], `${locale}: missing strings`);
  }
});
