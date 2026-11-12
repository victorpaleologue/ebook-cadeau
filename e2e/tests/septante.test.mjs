// Scenarios 1–3: a gift bought at the fictional store "Septante" (DRM-free, gifts as
// `hosted_file`), followed in real browsers from the checkout to three kinds of
// recipient device:
//
//   1. the buyer on a desktop browser: catalogue → product → cart → account → checkout with
//      the gift option → print at home → pay → order page → card PDF, whose inside page
//      carries a QR code decoded here (pdftoppm + zbarimg);
//   2. the recipient on a phone (Pixel 7 emulation): opens the decoded URL, reads the
//      gift page, downloads the EPUB ("read on this phone"), asks for a pair code;
//   3. e-reader browsers with JavaScript DISABLED: a Kobo types the printed code on the
//      home page and downloads the EPUB; a PocketBook types the pair code at /e.
//
// One fresh `ebc-demo` constellation for the three (they share the gift).

import { after, before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import {
  audit, devices, download, known, launchBrowser, newContext, overflowX, pdfText,
  qrPayloads, readEpub, shot, startConstellation, text, weigh,
} from '../lib/harness.mjs';

// The real-world user agents of the simulated devices (demo/ereaders/src/profile.rs).
const KOBO_UA = 'Mozilla/5.0 (Linux; U; Android 2.0; en-us;) AppleWebKit/538.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/538.1 (Kobo Touch 0390/4.38.23038)';
const POCKETBOOK_UA = 'Mozilla/5.0 (Linux; U; Android 2.3.4; en-us; PocketBook/629 Build/GRJ22) AppleWebKit/533.1 (KHTML, like Gecko) Version/4.0 Mobile Safari/533.1';
/** docs/api.md §2: an e-reader page weighs less than 30 kB (HTML + style sheet). */
const EINK_BUDGET_BYTES = 30_000;

const GIVER = 'Camille';
const RECIPIENT = 'Sacha';
const MESSAGE = 'Joyeux anniversaire ! Bonne lecture.';

let world;
let browser;
/** What scenario 1 hands to the next ones. */
const gift = { code: null, claimUrl: null, qrUrl: null, pair: null, orderRef: null };

before(async () => {
  world = await startConstellation();
  browser = await launchBrowser();
});

after(async () => {
  await browser?.close();
  await world?.stop();
});

/** Fails the test when a context saw console errors or requests to other hosts. */
function noProblems(problems, what, expected = []) {
  const unexpected = problems.filter((problem) => !expected.some((pattern) => pattern.test(problem)));
  assert.deepEqual(unexpected, [], `${what}: console errors or requests to other hosts`);
}

/** Checks the stamped EPUB the recipient received. */
function checkEpub(file, what) {
  let entries;
  try {
    entries = readEpub(file.bytes);
  } catch (error) {
    assert.fail(`${what}: ${file.suggested} (${file.bytes.length} bytes) is not an EPUB: ${error.message}`);
  }
  assert.match(file.suggested, /\.epub$/i, `${what}: the file name`);
  const watermark = entries.get('OEBPS/watermark.xhtml');
  assert.ok(watermark, `${what}: no OEBPS/watermark.xhtml in the EPUB (entries: ${[...entries.keys()].join(', ')})`);
  assert.match(watermark.toString('utf8'), new RegExp(`Offert à ${RECIPIENT} par ${GIVER}`), `${what}: the watermark names the recipient and the giver`);
}

describe('Septante (hosted_file gift)', () => {
  test('1. the buyer: gift checkout, card PDF and its QR code', async (t) => {
    const { context, problems } = await newContext(browser, { viewport: { width: 1280, height: 900 } });
    const page = await context.newPage();

    await page.goto(`${world.septante}/`);
    assert.match(await text(page, '#demo-banner'), /Démo — boutique fictive, sans lien avec une enseigne réelle/, 'the demo notice is visible');
    assert.equal(await text(page, '#brand'), 'Septante');
    await shot(page, '1-septante-catalogue');

    await page.click('#book-candide');
    assert.equal(await text(page, '#book-title'), 'Candide');
    assert.match(await text(page, '#book-author'), /Voltaire/);
    await shot(page, '1-septante-product');

    await page.click('#add-to-cart');
    await page.waitForURL(/\/panier$/);
    assert.equal(await page.locator('#cart tbody tr').count(), 1, 'one book in the cart');
    assert.equal(await text(page, '#cart-count'), '1');
    await page.click('#checkout-link');

    // An account is needed to check out: the sign-in page comes first.
    await page.waitForSelector('#register-form');
    await page.fill('#register-email', 'camille@example.org');
    await page.fill('#register-password', 'motdepasse');
    await page.click('#register-submit');
    await page.waitForSelector('#checkout-form');

    // The checkout wording, as the buyer reads it.
    assert.equal(await text(page, '#gift-block legend'), "C'est pour offrir ?");
    assert.equal(await text(page, 'label[for="gift-option"]'), 'Offrir cet ebook');
    const help = await text(page, '#gift-option-help');
    for (const words of ['eBook Cadeau', 'couverture du livre', "un dos d'un centimètre", 'QR code', 'liseuse, tablette ou téléphone']) {
      assert.ok(help.includes(words), `the gift option help mentions "${words}": ${help}`);
    }
    assert.equal(await text(page, '#gift-delivery-note'), 'La personne qui reçoit la carte télécharge le livre sans créer de compte.');
    assert.equal(await text(page, 'label[for="card-delivery-home"]'), 'Carte à imprimer chez moi (gratuit)');
    assert.match(await text(page, 'label[for="card-delivery-post"]'), /^Carte imprimée et envoyée par la poste — \d+,\d\d\s€ par carte/);
    assert.equal(await page.isChecked('#gift-option'), false, 'the gift option is not ticked by default');

    await page.check('#gift-option');
    await page.fill('#giver-name', GIVER);
    await page.fill('#recipient-name', RECIPIENT);
    await page.fill('#gift-message', MESSAGE);
    await page.check('#card-delivery-home');
    await shot(page, '1-septante-checkout');
    const findings = await audit(page);
    known(t, findings.length === 0, `Septante checkout accessibility: ${findings.join(' | ')}`);

    await page.click('#pay');
    await page.waitForSelector('#order-ref');
    // The store refreshes the gift from the server on every load: wait for the card.
    for (let attempt = 0; attempt < 20 && (await page.locator('#card-pdf').count()) === 0; attempt += 1) {
      await page.waitForTimeout(250);
      await page.reload();
    }
    gift.orderRef = await text(page, '#order-ref');
    assert.match(gift.orderRef, /^SEP-\d{4}-\d{6}$/);
    assert.equal(await page.getAttribute('#order-status', 'data-status'), 'paid');
    assert.equal(await page.getAttribute('#gift-status', 'data-status'), 'created');
    assert.equal(await text(page, '#gift-recipient'), RECIPIENT);
    assert.equal(await text(page, '#gift-giver'), GIVER);
    assert.equal(await text(page, '#gift-message-text'), MESSAGE);
    assert.match(await text(page, '#order-card-line'), /Carte à imprimer chez moi \(gratuit\)\s*0,00\s€/);
    gift.code = await text(page, '#claim-code');
    assert.match(gift.code, /^[A-Z]{4}-[A-Z]{4}-[A-Z]{4}$/, 'the code is printed as XXXX-XXXX-XXXX');
    gift.claimUrl = await page.getAttribute('#claim-url', 'href');
    assert.equal(gift.claimUrl, `${world.server}/k/${gift.code.replaceAll('-', '')}`, 'the claim URL is the server address + /k/ + the compact code');
    assert.equal(await text(page, '#claim-url'), gift.claimUrl);
    assert.equal(await text(page, '#card-pdf'), 'Télécharger la carte à imprimer (PDF)');
    await shot(page, '1-septante-order');

    // The card, as the buyer saves it.
    const card = await download(page, '#card-pdf', '1-septante-card.pdf');
    assert.equal(card.bytes.subarray(0, 5).toString('latin1'), '%PDF-', 'the card is a PDF');
    assert.match(card.suggested, /^carte-ebook-cadeau-.+\.pdf$/);
    const qr = qrPayloads(card.path, 2, '1-septante-card');
    if (qr === null) {
      t.diagnostic('SKIPPED: pdftoppm or zbarimg is missing; the QR code was not decoded (the order page URL is used instead)');
      gift.qrUrl = gift.claimUrl.toUpperCase();
    } else {
      assert.deepEqual(qr, [gift.claimUrl.toUpperCase()], 'the inside page carries one QR code: the claim URL in upper case');
      gift.qrUrl = qr[0];
      const outside = qrPayloads(card.path, 1, '1-septante-card');
      assert.deepEqual(outside, [], 'no QR code on the outside of the card');
    }
    const inside = pdfText(card.path, 2);
    if (inside !== null) {
      assert.ok(inside.includes(gift.code), `the code is printed inside:\n${inside}`);
      for (const words of [RECIPIENT, GIVER, 'Joyeux anniversaire', new URL(world.server).host]) {
        assert.ok(inside.includes(words), `"${words}" is printed inside:\n${inside}`);
      }
    }
    noProblems(problems, 'the buyer');
    await context.close();
  });

  test('2. the recipient on a phone: gift page, EPUB, pair code', async (t) => {
    if (!gift.qrUrl) return t.skip('needs the gift of scenario 1');
    const { context, problems } = await newContext(browser, { ...devices['Pixel 7'] });
    const page = await context.newPage();
    const weight = weigh(page);

    // What the camera app opens: the URL in upper case (`/K/CODE`).
    await page.goto(gift.qrUrl);
    assert.match(page.url(), /\/K\/[A-Z]{12}$/, 'the upper-case URL is served as is');
    assert.equal(await text(page, '#gift-title'), 'Candide');
    assert.match(await text(page, '#gift-author'), /Voltaire/);
    assert.match(await text(page, '#gift-from'), new RegExp(GIVER));
    assert.equal(await text(page, '#gift-message'), MESSAGE);
    const choices = await page.locator('#device-chooser a').evaluateAll((links) => links.map((a) => `${a.id}${a.classList.contains('on') ? '*' : ''}`));
    assert.deepEqual(choices, ['device-kobo', 'device-kindle', 'device-pocketbook', 'device-tolino', 'device-phone*', 'device-computer'],
      'the device chooser offers six devices, the phone detected');
    assert.equal(await text(page, '#download'), 'Lire sur ce téléphone ou envoyer vers une application');
    const others = await text(page, '#other-ways');
    assert.ok(others.includes(new URL(world.server).host) && others.includes(gift.code), `the e-reader instructions name the address and the code: ${others}`);
    assert.equal(await overflowX(page), 0, 'no horizontal scrolling on the phone');
    assert.equal(await page.locator('script').count(), 0, 'the gift page has no script');
    const sizes = weight();
    assert.ok(sizes.total < EINK_BUDGET_BYTES, `the gift page weighs ${sizes.total} bytes (budget ${EINK_BUDGET_BYTES})`);
    await shot(page, '2-phone-gift-page');
    const findings = await audit(page);
    assert.deepEqual(findings, [], 'accessibility of the gift page');

    // The chooser switches the page to another device without losing the gift.
    await page.click('#device-kindle');
    assert.match(page.url(), /\?device=kindle$/);
    assert.equal(await page.getAttribute('#device-kindle', 'class'), 'on');
    await page.click('#device-phone');

    const epub = await download(page, '#download', '2-phone-candide.epub');
    checkEpub(epub, 'the phone download');

    // The phone stays on the gift page: ask for a code for an e-reader.
    await page.goto(gift.claimUrl);
    await page.click('#pair-start');
    await page.waitForSelector('#pair-code');
    gift.pair = await text(page, '#pair-code');
    assert.match(gift.pair, /^[A-Z]{3}-[A-Z]{3}$/, 'a pair code of six letters, shown as XXX-XXX');
    assert.equal(await text(page, '#pair-url'), `${new URL(world.server).host}/e`);
    assert.match(await text(page, '#pair-expires'), /15/, 'the page says how long the pair code lasts');
    await shot(page, '2-phone-pair-code');
    noProblems(problems, 'the phone');
    await context.close();
  });

  test('3a. a Kobo without JavaScript types the printed code', async (t) => {
    if (!gift.code) return t.skip('needs the gift of scenario 1');
    const { context, problems } = await newContext(browser, {
      userAgent: KOBO_UA, viewport: { width: 480, height: 640 }, javaScriptEnabled: false, isMobile: true, hasTouch: true,
    });
    const page = await context.newPage();
    const weight = weigh(page);

    await page.goto(`${world.server}/`);
    assert.equal(await page.locator('script').count(), 0, 'the home page has no script');
    assert.ok(weight().total < EINK_BUDGET_BYTES, `home page: ${JSON.stringify(weight())}`);
    assert.equal(await overflowX(page), 0, 'no horizontal scrolling at 480 px');
    await shot(page, '3-kobo-home');
    // The cheapest way the card allows: lower case, no dashes.
    await page.fill('#code-input', gift.code.replaceAll('-', '').toLowerCase());
    await page.click('#code-submit');
    await page.waitForSelector('#gift-title');
    assert.match(page.url(), new RegExp(`/k/${gift.code.replaceAll('-', '')}$`), 'the form leads to the gift page');
    assert.equal(await page.getAttribute('body', 'class'), 'eink', 'the page is laid out for e-ink');
    assert.equal(await page.getAttribute('#device-kobo', 'class'), 'on', 'the Kobo is recognised');
    assert.equal(await text(page, '#gift-title'), 'Candide');
    assert.equal(await page.locator('script').count(), 0, 'the gift page has no script');
    const sizes = weight();
    assert.ok(sizes.total < EINK_BUDGET_BYTES, `the gift page weighs ${sizes.total} bytes (budget ${EINK_BUDGET_BYTES})`);
    assert.equal(await overflowX(page), 0, 'no horizontal scrolling at 480 px');
    const height = await page.evaluate(() => document.getElementById('download').getBoundingClientRect().height);
    assert.ok(height >= 48, `the download button is a large target (${height} px)`);
    await shot(page, '3-kobo-gift-page');

    const epub = await download(page, '#download', '3-kobo-candide.epub');
    checkEpub(epub, 'the Kobo download');
    noProblems(problems, 'the Kobo');
    await context.close();
  });

  test('3b. a PocketBook without JavaScript types the pair code at /e', async (t) => {
    if (!gift.pair) return t.skip('needs the pair code of scenario 2');
    const { context, problems } = await newContext(browser, {
      userAgent: POCKETBOOK_UA, viewport: { width: 600, height: 800 }, javaScriptEnabled: false, isMobile: true, hasTouch: true,
    });
    const page = await context.newPage();
    const weight = weigh(page);
    await page.goto(`${world.server}/e`);
    assert.equal(await page.locator('script').count(), 0);
    assert.ok(weight().total < EINK_BUDGET_BYTES, `/e: ${JSON.stringify(weight())}`);
    assert.equal(await overflowX(page), 0);
    await shot(page, '3-pocketbook-pair-form');

    // A wrong code first: refused, the form is offered again.
    await page.fill('#pair-input', 'zzzzzz');
    await page.click('#pair-submit');
    await page.waitForSelector('#error');
    assert.ok(await page.isVisible('#pair-form'), 'the form is offered again after a wrong code');
    await shot(page, '3-pocketbook-pair-refused');

    await page.fill('#pair-input', gift.pair.replace('-', '').toLowerCase());
    const epub = await download(page, '#pair-submit', '3-pocketbook-candide.epub');
    checkEpub(epub, 'the PocketBook download');

    // Single use: the same pair code again is refused.
    await page.goto(`${world.server}/e`);
    await page.fill('#pair-input', gift.pair);
    await page.click('#pair-submit');
    await page.waitForSelector('#error');
    // The two refusals are 404 documents, which Chromium logs as failed resources.
    const refused = new RegExp(`\\(resource ${world.server}/e\\): Failed to load resource: the server responded with a status of 404`);
    noProblems(problems, 'the PocketBook', [refused]);
    await context.close();
  });
});
