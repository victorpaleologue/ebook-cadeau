// Walks the card creator in a real browser (Chromium through Playwright, which must be
// installed already — this script installs nothing):
//
//   node web/tests/browser-smoke.mjs
//
// It serves web/ on a loopback port with the right MIME types and the server's
// Content-Security-Policy, refuses every request to another host (the app must make
// none on this path), types a title and an author, keeps the typographic card, takes
// the demonstration code, downloads the PDF and checks it, then saves screenshots of
// the desktop and phone layouts to web/tests/screenshots/ (deviceScaleFactor 1).
// Exit status 0 on success, 1 on any failure.

import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { mkdirSync, readFileSync, statSync } from 'node:fs';
import { extname, join, normalize, dirname, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const { chromium } = require('playwright');

const here = dirname(fileURLToPath(import.meta.url));
const web = normalize(join(here, '..'));
const shots = join(here, 'screenshots');
mkdirSync(shots, { recursive: true });

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.wasm': 'application/wasm',
  '.icc': 'application/vnd.iccprofile',
  '.png': 'image/png',
  '.txt': 'text/plain; charset=utf-8',
};
// The policy ebc-server sends for /app/ (crates/ebc-server/src/routes/mod.rs, CSP_APP).
const CSP = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; " +
  "img-src 'self' data: blob: https:; connect-src 'self' https:; worker-src 'self' blob:; object-src 'none'; " +
  "base-uri 'self'; frame-ancestors 'none'";

// Recorded Open Library answers (crates/ebc-catalog/tests/fixtures), served under
// /fixture/ol/ so that the lookup can be pointed here with ?catalog.openlibrary=….
const OPEN_LIBRARY = join(web, '..', 'crates', 'ebc-catalog', 'tests', 'fixtures', 'openlibrary');
function fixture(path) {
  const rest = path.slice('/fixture/ol/'.length);
  if (rest === 'api/books') return join(OPEN_LIBRARY, 'etranger.json');
  const match = /^(authors|works)\/([A-Z0-9]+)\.json$/.exec(rest);
  return match ? join(OPEN_LIBRARY, `${match[1] === 'authors' ? 'author' : 'work'}-${match[2]}.json`) : null;
}

const server = createServer((request, response) => {
  const path = decodeURIComponent(new URL(request.url, 'http://127.0.0.1').pathname);
  if (path.startsWith('/fixture/ol/')) {
    const file = fixture(path);
    try {
      response.writeHead(200, { 'Content-Type': 'application/json' }).end(readFileSync(file));
    } catch {
      response.writeHead(404, { 'Content-Type': 'application/json' }).end('{}');
    }
    return;
  }
  const file = normalize(join(web, path.endsWith('/') ? `${path}index.html` : path));
  if (!file.startsWith(web + sep)) {
    response.writeHead(403).end();
    return;
  }
  try {
    if (!statSync(file).isFile()) throw new Error('not a file');
    response.writeHead(200, {
      'Content-Type': MIME[extname(file)] || 'application/octet-stream',
      'Content-Security-Policy': CSP,
      'X-Content-Type-Options': 'nosniff',
      'Cache-Control': 'no-store',
    });
    response.end(readFileSync(file));
  } catch {
    response.writeHead(404, { 'Content-Type': 'text/plain' }).end('not found');
  }
});
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;

const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
  console.log(`${condition ? 'ok  ' : 'FAIL'} ${message}`);
};

/** Opens the app in a fresh context and reports console errors and foreign requests. */
async function open(browser, contextOptions, query = '') {
  const context = await browser.newContext({ deviceScaleFactor: 1, acceptDownloads: true, ...contextOptions });
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (error) => problems.push(`page error: ${error.message}`));
  page.on('console', (message) => {
    if (message.type() === 'error') problems.push(`console: ${message.text()}`);
  });
  await context.route('**/*', (route) => {
    const url = route.request().url();
    if (url.startsWith(origin) || url.startsWith('blob:') || url.startsWith('data:')) return route.continue();
    problems.push(`external request: ${url}`);
    return route.abort();
  });
  await page.goto(`${origin}/index.html${query}`);
  await page.waitForSelector('#main:not([hidden])', { timeout: 60_000 });
  return { context, page, problems };
}

async function fillCard(page) {
  await page.fill('#title', 'Les Vagues du soir');
  await page.fill('#author', 'Jeanne Roy');
  await page.fill('#giver', 'Alice');
  await page.fill('#recipient', 'Bob');
  await page.fill('#message', 'Joyeux anniversaire ! Bonne lecture.');
}

async function previewReady(page) {
  await page.waitForFunction(() => {
    const img = document.getElementById('preview-outside');
    const inside = document.getElementById('preview-inside');
    return img.src.startsWith('blob:') && img.complete && img.naturalWidth > 0 &&
      inside.src.startsWith('blob:') && inside.complete && inside.naturalWidth > 0 &&
      !document.getElementById('preview-status').textContent;
  }, null, { timeout: 30_000 });
}

const browser = await chromium.launch();
try {
  // ---- Desktop, French (the default) ------------------------------------------------
  const desktop = await open(browser, { viewport: { width: 1280, height: 900 } });
  const { page } = desktop;
  check((await page.evaluate(() => document.documentElement.lang)).startsWith('fr'), 'the page starts in French');
  check((await page.textContent('h1')).trim().length > 0, 'the strings come from the package');
  check(await page.isChecked('input[name="cover-mode"][value="typographic"]'), 'the typographic card is the default');
  check(await page.isHidden('#own-image'), 'no image panel until the user asks for one');

  // Every form control has an accessible name.
  const unlabelled = await page.evaluate(() => [...document.querySelectorAll('input, select, textarea, button')]
    .filter((el) => el.type !== 'hidden' && !el.closest('[hidden]'))
    .filter((el) => !(el.labels && el.labels.length) && !el.getAttribute('aria-label') && !el.textContent.trim())
    .map((el) => el.id || el.name || el.outerHTML.slice(0, 60)));
  check(unlabelled.length === 0, `every visible control is labelled ${unlabelled.join(', ')}`);

  await fillCard(page);
  await previewReady(page);
  check(true, 'the live preview shows both sides');
  const warnings = await page.$$eval('#warnings li', (items) => items.map((li) => li.className));
  check(warnings.length === 1 && warnings[0] === 'none', 'no warning for the typographic card');
  check(await page.getAttribute('.preview-pane', 'data-image') === 'no', 'the typographic preview holds no image');

  // Download before any code: refused with a message.
  await page.click('#download');
  await page.waitForFunction(() => document.getElementById('download-status').textContent.length > 0);
  check(await page.$eval('#download-status', (el) => el.classList.contains('is-error')), 'the PDF needs a code first');

  // The demonstration code, then the PDF.
  await page.check('input[name="ebook-mode"][value="demo"]');
  await page.waitForSelector('#gift-result:not([hidden])');
  const code = await page.textContent('#gift-code');
  check(/[BCDFGHJKLMNPQRSTVWXZ]{4}-[BCDFGHJKLMNPQRSTVWXZ]{4}-[BCDFGHJKLMNPQRSTVWXZ]{4}/.test(code), `a demonstration code: ${code.trim()}`);
  await previewReady(page);
  const [download] = await Promise.all([page.waitForEvent('download'), page.click('#download')]);
  const path = await download.path();
  const pdf = readFileSync(path);
  check(pdf.subarray(0, 5).toString('latin1') === '%PDF-' && pdf.length > 10_000, `the PDF downloads (${pdf.length} bytes, ${download.suggestedFilename()})`);
  check(!pdf.includes(Buffer.from('/Subtype/Image')) && !pdf.includes(Buffer.from('/Subtype /Image')), 'the typographic card holds no image');
  await page.waitForFunction(() => document.getElementById('download-status').classList.contains('is-ok'));
  await page.screenshot({ path: join(shots, 'desktop.png'), fullPage: true });
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({ path: join(shots, 'desktop-viewport.png') });
  await page.locator('#step-card').scrollIntoViewIfNeeded();
  await page.screenshot({ path: join(shots, 'desktop-card-step.png') });

  // The image panel and the rights statements (no file chosen: nothing printed).
  await page.check('input[name="cover-mode"][value="own"]');
  check(await page.isVisible('#rights'), 'the rights statements appear with the image option');
  const statements = await page.$$eval('#rights-list input', (inputs) => inputs.map((i) => i.value));
  check(statements.join() === 'own_work,print_licence,publisher_authorised,public_domain,private_copy', `five statements: ${statements}`);
  check(await page.isDisabled('#adapt'), 'adapting needs an image and a statement');

  // An image of the user's, with a statement.
  await page.setInputFiles('#cover-file', join(web, '..', 'crates', 'ebc-wasm', 'tests', 'fixtures', 'golden-cover.png'));
  await page.waitForSelector('#cover-info:not([hidden])');
  check(/\d+ pp/.test(await page.textContent('#cover-quality')), `print quality shown: ${(await page.textContent('#cover-quality')).trim()}`);
  await page.check('#rights-list input[value="private_copy"]');
  const proDisabled = await page.$$eval('#profile option', (opts) => opts.filter((o) => o.disabled).map((o) => o.value));
  check(proDisabled.join() === 'upload_eu,professional_eu', `a private copy closes the professional profiles: ${proDisabled}`);
  check((await page.textContent('#profile-note')).trim().length > 0, 'with the reason shown');
  check(await page.isDisabled('#adapt'), 'a private copy cannot be adapted');
  await page.check('#rights-list input[value="own_work"]');
  check(await page.isEnabled('#adapt'), 'own work can be adapted once ticked');
  check(await page.isHidden('#spine-styles'), 'cover-sampling spine styles stay hidden before the tick');
  await page.check('#adapt');
  check(await page.isVisible('#spine-styles'), 'cover-sampling spine styles appear after the tick');
  await previewReady(page);
  const imageShown = await page.waitForSelector('.preview-pane[data-image="yes"]', { timeout: 20_000 }).then(() => true, () => false);
  check(imageShown, 'the declared image is in the preview');
  await page.click('#font-detect');
  await page.waitForFunction(() => {
    const text = document.getElementById('font-result').textContent;
    return text && !/…|\.\.\./.test(text);
  }, null, { timeout: 60_000 });
  check(true, `font detection answers: ${(await page.textContent('#font-result')).trim()}`);
  await page.screenshot({ path: join(shots, 'desktop-image-option.png'), fullPage: true });

  // Another language.
  await page.selectOption('#ui-locale', 'en');
  await page.waitForFunction(() => document.documentElement.lang.startsWith('en'));
  check((await page.textContent('#h-book')).trim() === 'The book', 'the language switch works');
  check(desktop.problems.length === 0, `no console error, no external request ${desktop.problems.join(' | ')}`);
  await desktop.context.close();

  // ---- ISBN lookup against recorded catalogue answers --------------------------------
  const lookup = await open(browser, { viewport: { width: 1280, height: 900 } },
    `?catalog.openlibrary=${encodeURIComponent(`${origin}/fixture/ol`)}`);
  await lookup.page.fill('#isbn', '978-2-07-036002-4');
  await lookup.page.click('#lookup');
  await lookup.page.waitForSelector('#isbn-status.is-ok, #isbn-status.is-error', { timeout: 30_000 });
  const found = await lookup.page.inputValue('#title');
  check(/tranger/i.test(found), `the lookup fills the title: "${found}" (${(await lookup.page.textContent('#isbn-status')).trim()})`);
  check(/Camus/.test(await lookup.page.inputValue('#author')), 'and the author');
  await previewReady(lookup.page);
  // The cover preview of the lookup is the one request to another host, and only an <img>.
  const foreign = lookup.problems.filter((p) => !/covers\.openlibrary\.org|Failed to load resource/.test(p));
  check(foreign.length === 0, `lookup: nothing else leaves the page ${foreign.join(' | ')}`);
  await lookup.context.close();

  // ---- Phone width ------------------------------------------------------------------
  const phone = await open(browser, { viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true });
  await fillCard(phone.page);
  await previewReady(phone.page);
  const overflow = await phone.page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check(overflow <= 0, `no horizontal scrolling at 390 px (overflow ${overflow} px)`);
  await phone.page.screenshot({ path: join(shots, 'phone.png'), fullPage: true });
  await phone.page.evaluate(() => window.scrollTo(0, 0));
  await phone.page.screenshot({ path: join(shots, 'phone-viewport.png') });
  await phone.page.locator('.preview-pane').scrollIntoViewIfNeeded();
  await phone.page.screenshot({ path: join(shots, 'phone-preview.png') });
  check(phone.problems.length === 0, `phone: no console error ${phone.problems.join(' | ')}`);
  await phone.context.close();
} catch (error) {
  failures.push(String(error && error.stack || error));
  console.error(error);
} finally {
  await browser.close();
  server.close();
}

if (failures.length) {
  console.error(`\n${failures.length} failure(s)`);
  process.exit(1);
}
console.log(`\nscreenshots in ${shots}`);
