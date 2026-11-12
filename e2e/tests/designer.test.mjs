// Scenario 5: the card designer web app (web/, served by ebc-server at /app/). Title
// and author typed by hand, the typographic card, the live preview of both sides, the
// demonstration code, the PDF — checked with `ebc preflight` and its QR code decoded —,
// the language switch, the phone layout, and a small accessibility audit.
// Scenario 6: a gift created on the server from the designer, then a printed card ordered
// through the designer's address form from the demo printer (docs/api.md §3 and §5).

import { after, before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { ROOT, audit, binary, freePort, launchBrowser, newContext, overflowX, qrPayloads, run, shot, startConstellation, text } from '../lib/harness.mjs';

let world;
let browser;

before(async () => {
  world = await startConstellation({ webApp: true });
  for (const note of world.notes) console.log(`  NOTE: ${note}`);
  browser = await launchBrowser();
});

after(async () => {
  await browser?.close();
  await world?.stop();
});

/** The app's profile names (from the wasm package) → the `ebc` CLI's. */
const CLI_PROFILE = { upload_eu: 'upload', professional_eu: 'pro', home_a4: 'home-a4', home_letter: 'home-letter', home_a3: 'home-a3' };

async function previewReady(page) {
  await page.waitForFunction(() => {
    const outside = document.getElementById('preview-outside');
    const inside = document.getElementById('preview-inside');
    return outside.src.startsWith('blob:') && outside.complete && outside.naturalWidth > 0 &&
      inside.src.startsWith('blob:') && inside.complete && inside.naturalWidth > 0 &&
      !document.getElementById('preview-status').textContent;
  }, null, { timeout: 60_000 });
}

describe('card designer (/app/)', () => {
  test('5. typographic card: preview, PDF, preflight, language, accessibility', async (t) => {
    for (const note of world.notes) t.diagnostic(`NOTE: ${note}`);
    const { context, problems } = await newContext(browser, { viewport: { width: 1280, height: 900 } });
    const page = await context.newPage();
    const response = await page.goto(world.webApp);
    assert.equal(response.status(), 200);
    assert.match(response.headers()['content-security-policy'] || '', /script-src 'self'/, 'the server sends its Content-Security-Policy for /app/');
    await page.waitForSelector('#main:not([hidden])', { timeout: 60_000 });
    assert.match(await page.getAttribute('html', 'lang'), /^fr/, 'French by default');
    assert.ok(await page.isChecked('input[name="cover-mode"][value="typographic"]'), 'the typographic card is the default');
    let findings = await audit(page);
    assert.deepEqual(findings, [], 'accessibility of the empty form (fr)');

    await page.fill('#title', 'Les Vagues du soir');
    await page.fill('#author', 'Jeanne Roy');
    await page.fill('#giver', 'Alice');
    await page.fill('#recipient', 'Bob');
    await page.fill('#message', 'Joyeux anniversaire ! Bonne lecture.');
    await previewReady(page);
    const previews = await page.evaluate(() => ['preview-outside', 'preview-inside'].map((id) => {
      const img = document.getElementById(id);
      return { alt: img.alt, width: img.naturalWidth, height: img.naturalHeight };
    }));
    for (const preview of previews) {
      assert.ok(preview.alt.includes('Les Vagues du soir'), `the preview says what it shows: alt="${preview.alt}"`);
      assert.ok(preview.width > preview.height, `a preview shows a whole spread (${preview.width}×${preview.height})`);
    }
    assert.equal(await page.getAttribute('.preview-pane', 'data-image'), 'no', 'the typographic preview holds no image');
    const warnings = await page.$$eval('#warnings li', (items) => items.map((li) => li.className));
    assert.deepEqual(warnings, ['none'], 'no warning for the typographic card');

    await page.check('input[name="ebook-mode"][value="demo"]');
    await page.waitForSelector('#gift-result:not([hidden])');
    const shown = await text(page, '#gift-code');
    const code = (/[A-Z]{4}-[A-Z]{4}-[A-Z]{4}/.exec(shown) || [])[0];
    assert.ok(code, `a demonstration code is shown: "${shown}"`);
    await previewReady(page);
    await shot(page, '5-designer-filled');

    const profile = await page.inputValue('#profile');
    const format = await page.inputValue('#format');
    assert.ok(CLI_PROFILE[profile], `a known print profile: ${profile}`);
    const [file] = await Promise.all([page.waitForEvent('download'), page.click('#download')]);
    const pdf = `${process.env.EBC_E2E_ARTIFACTS || new URL('../artifacts/', import.meta.url).pathname}/5-designer-card.pdf`;
    await file.saveAs(pdf);
    assert.match(file.suggestedFilename(), new RegExp(`^ebook-cadeau-.+-${format}\\.pdf$`));
    await page.waitForFunction(() => document.getElementById('download-status').classList.contains('is-ok'));

    const preflight = run(binary('ebc'), ['preflight', '--profile', CLI_PROFILE[profile], '--format', format, pdf]);
    assert.equal(preflight.status, 0, `ebc preflight --profile ${CLI_PROFILE[profile]} --format ${format}:\n${preflight.stdout}\n${preflight.stderr}`);
    const qr = qrPayloads(pdf, 2, '5-designer-card');
    if (qr === null) {
      t.diagnostic('SKIPPED: pdftoppm or zbarimg is missing; the QR code was not decoded');
    } else {
      assert.equal(qr.length, 1, `one QR code inside: ${qr}`);
      assert.ok(qr[0].endsWith(`/K/${code.replaceAll('-', '')}`), `the QR code carries the demonstration code: ${qr[0]}`);
      assert.equal(qr[0], qr[0].toUpperCase(), 'the QR code is upper case (alphanumeric mode)');
    }

    // Another language: every string comes from the package.
    await page.selectOption('#ui-locale', 'en');
    await page.waitForFunction(() => document.documentElement.lang.startsWith('en'));
    assert.equal(await text(page, '#h-book'), 'The book');
    assert.equal(await page.inputValue('#title'), 'Les Vagues du soir', 'switching language keeps what was typed');
    await previewReady(page);
    findings = await audit(page);
    assert.deepEqual(findings, [], 'accessibility of the filled form (en)');
    await shot(page, '5-designer-english');
    assert.deepEqual(problems, [], 'console errors or requests to other hosts (desktop)');
    await context.close();

    // A phone: no sideways scrolling, preview still drawn.
    const phone = await newContext(browser, { viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true });
    const small = await phone.context.newPage();
    await small.goto(`${world.webApp}?lang=de`);
    await small.waitForSelector('#main:not([hidden])', { timeout: 60_000 });
    assert.match(await small.getAttribute('html', 'lang'), /^de/, '?lang=de selects German');
    await small.fill('#title', 'Die Wellen');
    await small.fill('#author', 'Jeanne Roy');
    await previewReady(small);
    assert.equal(await overflowX(small), 0, 'no horizontal scrolling at 390 px');
    await shot(small, '5-designer-phone-de');
    assert.deepEqual(phone.problems, [], 'console errors or requests to other hosts (phone)');
    await phone.context.close();
  });
});

/**
 * An `ebc-server` serving the designer at /app/ with the constellation's printer as its
 * print provider (EBC_PRINTER_URL). Started here so that the scenario does not depend on
 * whether `ebc-demo` serves /app/ itself.
 */
async function startOrderingServer(printer) {
  const port = await freePort();
  const data = mkdtempSync(join(tmpdir(), 'ebc-e2e-order-'));
  const base = `http://127.0.0.1:${port}`;
  const child = spawn(binary('ebc-server'), [], {
    cwd: ROOT,
    env: { ...process.env, EBC_BIND: `127.0.0.1:${port}`, EBC_PUBLIC_URL: base, EBC_DATA_DIR: data, EBC_WEB_DIR: join(ROOT, 'web'), EBC_PRINTER_URL: printer, RUST_LOG: 'warn' },
    stdio: ['ignore', 'ignore', 'pipe'],
  });
  let stderr = '';
  child.stderr.on('data', (chunk) => { stderr += chunk; });
  const stop = () => {
    try { child.kill('SIGKILL'); } catch { /* already gone */ }
    rmSync(data, { recursive: true, force: true });
  };
  const deadline = Date.now() + 30_000;
  for (;;) {
    const answer = await fetch(`${base}/app/`).catch(() => null);
    if (answer && answer.ok) break;
    if (child.exitCode !== null || Date.now() > deadline) {
      stop();
      throw new Error(`ebc-server with EBC_PRINTER_URL did not start (exit ${child.exitCode}):\n${stderr}`);
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  return { base, app: `${base}/app/`, stop };
}

/** A small PDF the server's upload sniffing accepts (the "eBook" of the gift). */
const EBOOK_PDF = Buffer.from('%PDF-1.7\n1 0 obj\n<< /Type /Catalog >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n');

describe('ordering a printed card (/app/ → server → printer)', () => {
  test('6. gift on the server, structured address, printer refusal and acceptance', async (t) => {
    const server = await startOrderingServer(world.printer);
    t.after(server.stop);
    const { context, problems } = await newContext(browser, { viewport: { width: 1280, height: 900 } });
    // The deployment switch of web/config.json: ordering is offered.
    await context.route('**/app/config.json', (route) => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ server: null, base_url: null, print_orders: true, catalog: { base_urls: {}, enable: [], disable: [], contact_email: null } }),
    }));
    const page = await context.newPage();
    const orderCalls = [];
    page.on('request', (request) => {
      if (request.url().includes('/print-orders')) orderCalls.push(request.postDataJSON());
    });
    await page.goto(server.app);
    await page.waitForSelector('#main:not([hidden])', { timeout: 60_000 });
    await page.fill('#title', 'Les Vagues du soir');
    await page.fill('#author', 'Jeanne Roy');

    // Step 5: the gift is created on the server.
    await page.check('input[name="ebook-mode"][value="upload"]');
    await page.setInputFiles('#ebook-file', { name: 'vagues.pdf', mimeType: 'application/pdf', buffer: EBOOK_PDF });
    await page.check('input[name="ebook-rights"][value="own_work"]');
    await page.check('#ebook-confirm');
    await page.click('#create-gift');
    await page.waitForSelector('#gift-result:not([hidden])', { timeout: 30_000 });
    const created = await text(page, '#gift-code');
    assert.match(created, /[A-Z]{4}-[A-Z]{4}-[A-Z]{4}/, `the server's code is shown: "${created}"`);

    // Step 6: a print-ready profile, the poche format, the order form.
    await page.selectOption('#profile', 'upload_eu');
    await page.selectOption('#format', 'poche');
    await page.click('#order summary');
    await page.waitForSelector('#order-form:not([hidden])');
    assert.equal(await page.inputValue('#order-country'), 'FR', 'the country defaults to France');
    const autocomplete = await page.$$eval('#order-address input, #order-address select', (fields) =>
      Object.fromEntries(fields.map((f) => [f.id, f.getAttribute('autocomplete')])));
    assert.deepEqual(autocomplete, {
      'order-name': 'shipping name',
      'order-line1': 'shipping address-line1',
      'order-line2': 'shipping address-line2',
      'order-postal-code': 'shipping postal-code',
      'order-city': 'shipping address-level2',
      'order-country': 'shipping country',
    });
    assert.equal(await text(page, 'label[for="order-postal-code"]'), 'Code postal');
    assert.equal(await page.$eval('#order-country option[value="FR"]', (o) => o.textContent), 'France', 'countries are named in the page language');

    // Nothing is sent while a required field is empty: the gap is marked and focused.
    await page.click('#order-submit');
    await page.waitForSelector('#order-name[aria-invalid="true"]');
    assert.equal(await page.evaluate(() => document.activeElement.id), 'order-name');
    assert.ok(await page.isVisible('#order-address-error'));
    assert.equal(orderCalls.length, 0, 'no order without a complete address');

    await page.fill('#order-name', 'Sacha Martin');
    await page.fill('#order-line1', '12 rue des Lilas');
    await page.fill('#order-postal-code', '69003');
    await page.fill('#order-city', 'Lyon');

    // A country the demo printer does not post to: its refusal is shown, not "unavailable".
    await page.selectOption('#order-country', 'US');
    await page.click('#order-submit');
    await page.waitForFunction(() => document.getElementById('order-status').classList.contains('is-error'), null, { timeout: 60_000 });
    assert.equal(await text(page, '#order-status'), 'Cet imprimeur ne livre pas dans ce pays.');

    // France: accepted by the printer.
    await page.selectOption('#order-country', 'FR');
    await page.click('#order-submit');
    await page.waitForFunction(() => {
      const status = document.getElementById('order-status');
      return status.classList.contains('is-ok') || (status.classList.contains('is-error') && !document.getElementById('order-submit').disabled);
    }, null, { timeout: 60_000 });
    const sent = await text(page, '#order-status');
    assert.ok(await page.$eval('#order-status', (s) => s.classList.contains('is-ok')), `the printer accepted the order: "${sent}"`);
    const orderId = sent.split(/\s/).pop();
    assert.equal(orderCalls.length, 2);
    assert.deepEqual(orderCalls[1].address, { name: 'Sacha Martin', line1: '12 rue des Lilas', postal_code: '69003', city: 'Lyon', country: 'FR' }, 'the documented address object');
    assert.equal(orderCalls[1].product, 'card-poche');

    const atPrinter = await (await fetch(`${world.printer}/v1/orders/${encodeURIComponent(orderId)}`)).json();
    assert.equal(atPrinter.order_id, orderId);
    assert.equal(atPrinter.status, 'accepted');
    assert.equal(atPrinter.product, 'card-poche');
    assert.equal(atPrinter.preflight.passed, true);

    const findings = await audit(page);
    assert.deepEqual(findings, [], 'accessibility with the order form open');
    await shot(page, '6-designer-order');
    // The browser logs the printer's refusal (422) as a failed resource: expected here.
    const unexpected = problems.filter((p) => !(p.includes('/print-orders') && p.includes('status of 422')));
    assert.deepEqual(unexpected, [], 'console errors or requests to other hosts');
    await context.close();
  });
});
