// Scenario 5: the card designer web app (web/, served by ebc-server at /app/). Title
// and author typed by hand, the typographic card, the live preview of both sides, the
// demonstration code, the PDF — checked with `ebc preflight` and its QR code decoded —,
// the language switch, the phone layout, and a small accessibility audit.

import { after, before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { audit, binary, launchBrowser, newContext, overflowX, qrPayloads, run, shot, startConstellation, text } from '../lib/harness.mjs';

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
