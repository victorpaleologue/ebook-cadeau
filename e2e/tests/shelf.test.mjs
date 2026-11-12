// Scenario 7: the shelf "Étagère des autrices" (demo — fictional, not affiliated), the
// service that hosts rights holders' own files and creates `partner_file` gifts with the
// core. In Chromium with **JavaScript off**: the author signs in with the link the demo
// mail sink received (read on the demo webmail), identifies herself, uploads an EPUB she
// wrote with a rights declaration, and makes a gift; the pages carry no script and pass
// the accessibility audit. Then (JavaScript on) the gift's designer link opens the card
// designer with the code, title and author filled in and the code gone from the address
// bar, and a designer configured with `shelf_url` offers "Host your own book on …".

import { after, before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { crc32 } from 'node:zlib';
import { ARTIFACTS, audit, launchBrowser, newContext, shot, startConstellation, text } from '../lib/harness.mjs';

const EMAIL = 'autrice@exemple.fr';
const TITLE = 'Les Vagues du soir';
const AUTHOR = 'Jeanne Roy';

/** A ZIP of stored (uncompressed) entries, in order. */
function zipStored(entries) {
  const parts = [];
  const central = [];
  let offset = 0;
  for (const [name, content] of entries) {
    const data = Buffer.from(content);
    const fileName = Buffer.from(name, 'utf8');
    const crc = crc32(data);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt16LE(0x21, 12); // 1980-01-01
    local.writeUInt32LE(crc, 14);
    local.writeUInt32LE(data.length, 18);
    local.writeUInt32LE(data.length, 22);
    local.writeUInt16LE(fileName.length, 26);
    parts.push(local, fileName, data);
    const entry = Buffer.alloc(46);
    entry.writeUInt32LE(0x02014b50, 0);
    entry.writeUInt16LE(20, 4);
    entry.writeUInt16LE(20, 6);
    entry.writeUInt16LE(0x21, 14);
    entry.writeUInt32LE(crc, 16);
    entry.writeUInt32LE(data.length, 20);
    entry.writeUInt32LE(data.length, 24);
    entry.writeUInt16LE(fileName.length, 28);
    entry.writeUInt32LE(offset, 42);
    central.push(entry, fileName);
    offset += 30 + fileName.length + data.length;
  }
  const directory = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(entries.length, 8);
  end.writeUInt16LE(entries.length, 10);
  end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...parts, directory, end]);
}

/** A tiny EPUB written for this test (own work: no third-party text). */
function epub() {
  return zipStored([
    ['mimetype', 'application/epub+zip'],
    ['META-INF/container.xml', '<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/package.opf" media-type="application/oebps-package+xml"/></rootfiles></container>'],
    ['OEBPS/package.opf', `<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="id">urn:uuid:7b0c0d1e-0000-4000-8000-000000000001</dc:identifier><dc:title>${TITLE}</dc:title><dc:creator>${AUTHOR}</dc:creator><dc:language>fr</dc:language></metadata><manifest><item id="c1" href="c1.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="c1"/></spine></package>`],
    ['OEBPS/c1.xhtml', `<?xml version="1.0" encoding="utf-8"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>${TITLE}</title></head><body><p>La mer montait doucement.</p></body></html>`],
  ]);
}

let world;
let browser;
const gift = { code: null, designer: null };

before(async () => {
  world = await startConstellation({ webApp: true });
  browser = await launchBrowser();
});

after(async () => {
  await browser?.close();
  await world?.stop();
});

/** No script, nothing loaded from elsewhere, no accessibility finding. */
async function checkPage(page, what) {
  const html = await page.content();
  assert.ok(!/<script/i.test(html), `${what}: no script`);
  const findings = await audit(page);
  assert.deepEqual(findings, [], `${what}: accessibility`);
}

describe('the shelf (hosts rights holders\' files, a partner of the core)', () => {
  test('7a. JavaScript off: sign in by e-mail link, identify, upload with a declaration, make a gift', async () => {
    const { context, problems } = await newContext(browser, { javaScriptEnabled: false, viewport: { width: 1000, height: 900 } });
    const page = await context.newPage();
    await page.goto(`${world.shelf}/`);
    assert.match(await text(page, '#intro'), /ni catalogue ni recherche/);
    assert.match(await text(page, '#not-purchases'), /licence personnelle/);
    await checkPage(page, 'home');
    await shot(page, '7-shelf-home');

    await page.click('#nav-signin');
    await page.fill('#email', EMAIL);
    await checkPage(page, 'sign-in');
    await page.click('#signin-submit');
    await page.waitForSelector('#link-sent');
    await shot(page, '7-shelf-link-sent');

    // The link the mail sink received, read on the demo webmail.
    await page.goto(`${world.webmail}/?to=${encodeURIComponent(EMAIL)}`);
    const link = page.locator('a[id^="mail-link-"]').first();
    const href = await link.getAttribute('href');
    assert.ok(href.startsWith(`${world.shelf}/connexion/`), href);
    await link.click();
    await page.waitForSelector('#confirm-signin');
    await page.click('#confirm-signin');
    await page.waitForSelector('#account');
    assert.ok(await page.isVisible('#identity-missing'), 'identification comes first');
    await checkPage(page, 'account');

    await page.click('#identity-link');
    await page.fill('#name', AUTHOR);
    await page.fill('#address', '12 rue des Lilas, 69003 Lyon');
    await checkPage(page, 'identity');
    await page.click('#identity-submit');
    await page.waitForSelector('#upload-link');

    await page.click('#upload-link');
    await page.waitForSelector('#upload-form');
    assert.match(await text(page, '#not-purchases'), /option cadeau de la librairie/);
    const file = join(ARTIFACTS, 'les-vagues-du-soir.epub');
    writeFileSync(file, epub());
    await page.setInputFiles('#file', file);
    await page.fill('#title', TITLE);
    await page.fill('#author', AUTHOR);
    await page.check('#basis-own_work');
    await page.check('#declared');
    await checkPage(page, 'upload');
    await shot(page, '7-shelf-upload');
    await page.click('#upload-submit');
    await page.waitForSelector('#book-title');
    assert.equal(await text(page, '#book-title'), TITLE);
    assert.equal(await page.getAttribute('#book-status', 'data-state'), 'live');
    assert.match(await text(page, '#declaration'), /auteur/);

    await page.fill('#recipient', 'Sacha');
    await page.fill('#giver', 'Jeanne');
    await page.click('#gift-submit');
    const code = page.locator('[id^="code-c_"]').first();
    await code.waitFor();
    gift.code = (await code.textContent()).trim();
    assert.match(gift.code, /^[B-DF-HJ-NP-TV-XZ]{4}-[B-DF-HJ-NP-TV-XZ]{4}-[B-DF-HJ-NP-TV-XZ]{4}$/);
    gift.designer = await page.locator('[id^="designer-"]').first().getAttribute('href');
    assert.ok(gift.designer.startsWith(`${world.server}/app/#code=`), gift.designer);
    await checkPage(page, 'book');
    await shot(page, '7-shelf-book');

    // The code works at the core.
    const claim = await fetch(`${world.server}/k/${gift.code.replace(/-/g, '')}`);
    assert.equal(claim.status, 200);
    assert.deepEqual(problems, [], 'console errors or requests to other hosts');
    await context.close();
  });

  test('7b. the gift\'s designer link fills in the code, title and author, and drops the code from the address bar', async (t) => {
    if (!gift.designer) return t.skip('no gift from 7a');
    const { context, problems } = await newContext(browser, { viewport: { width: 1280, height: 900 } });
    const page = await context.newPage();
    await page.goto(gift.designer);
    await page.waitForSelector('#main:not([hidden])', { timeout: 60_000 });
    await page.waitForFunction(() => document.getElementById('code').value !== '');
    assert.equal(await page.inputValue('#code'), gift.code);
    assert.equal(await page.inputValue('#title'), TITLE);
    assert.equal(await page.inputValue('#author'), AUTHOR);
    assert.ok(await page.isChecked('input[name="ebook-mode"][value="code"]'));
    assert.ok(!page.url().includes(gift.code) && !page.url().includes('code='), page.url());
    await shot(page, '7-designer-from-shelf');
    assert.deepEqual(problems, [], 'console errors or requests to other hosts');
    await context.close();
  });

  test('7c. a designer configured with shelf_url links to the shelf (no upload of its own)', async () => {
    const { context, problems } = await newContext(browser, { viewport: { width: 1280, height: 900 } });
    await context.route('**/app/config.json', (route) => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ server: null, base_url: null, print_orders: false, shelf_url: world.shelf, shelf_name: 'Étagère des autrices', catalog: { base_urls: {}, enable: [], disable: [], contact_email: null } }),
    }));
    const page = await context.newPage();
    await page.goto(world.webApp);
    await page.waitForSelector('#main:not([hidden])', { timeout: 60_000 });
    assert.ok(await page.isVisible('#shelf-choice'));
    assert.match(await text(page, '#shelf-choice-text'), /Héberger votre propre livre sur Étagère des autrices/);
    await page.check('input[name="ebook-mode"][value="shelf"]');
    assert.ok(await page.isVisible('#shelf-link'));
    assert.equal(await page.getAttribute('#shelf-link', 'href'), `${world.shelf}/`);
    assert.equal(await page.locator('input[type="file"][id="ebook-file"]').count(), 0, 'no upload from the designer');
    await shot(page, '7-designer-shelf-choice');
    assert.deepEqual(problems, [], 'console errors or requests to other hosts');
    await context.close();
  });
});
