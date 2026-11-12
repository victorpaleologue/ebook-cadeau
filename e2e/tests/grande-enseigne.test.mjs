// Scenario 4: a gift bought at the fictional store "Grande Enseigne" (gifts as
// `reseller_redirect`): the buyer checks out with the gift option; the recipient, on a
// phone, opens the claim URL, follows the signed redirect to the store, creates an
// account there, accepts the book and finds it in the library; the buyer's order page
// then says the book was added. A fresh `ebc-demo` constellation.

import { after, before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { audit, devices, known, launchBrowser, newContext, overflowX, shot, startConstellation, text } from '../lib/harness.mjs';

const GIVER = 'Dominique';
const RECIPIENT = 'Noa';
const MESSAGE = 'Pour tes vacances.';

let world;
let browser;
const gift = { claimUrl: null, orderUrl: null, activated: false };
let buyer;

before(async () => {
  world = await startConstellation();
  browser = await launchBrowser();
});

after(async () => {
  await buyer?.context.close();
  await browser?.close();
  await world?.stop();
});

describe('Grande Enseigne (reseller_redirect gift)', () => {
  test('4a. the buyer: gift checkout', async (t) => {
    buyer = await newContext(browser, { viewport: { width: 1280, height: 900 } });
    const page = await buyer.context.newPage();
    await page.goto(`${world.grandeEnseigne}/`);
    assert.match(await text(page, '#demo-banner'), /Démo — boutique fictive/);
    assert.equal(await text(page, '#brand'), 'Grande Enseigne');
    await shot(page, '4-grande-enseigne-catalogue');
    await page.click('#book-candide');
    await page.click('#add-to-cart');
    await page.click('#checkout-link');
    await page.fill('#register-email', 'dominique@example.org');
    await page.fill('#register-password', 'motdepasse');
    await page.click('#register-submit');
    await page.waitForSelector('#checkout-form');

    // The store's own wording ("Ebook"), and what happens on the recipient's side.
    assert.equal(await text(page, 'label[for="gift-option"]'), 'Offrir cet Ebook');
    assert.equal(await text(page, '#gift-delivery-note'),
      'La personne qui reçoit la carte ajoute le livre à son compte Grande Enseigne (elle peut le créer à ce moment-là).');
    await page.check('#gift-option');
    await page.fill('#giver-name', GIVER);
    await page.fill('#recipient-name', RECIPIENT);
    await page.fill('#gift-message', MESSAGE);
    await page.check('#card-delivery-home');
    await shot(page, '4-grande-enseigne-checkout');
    const findings = await audit(page);
    known(t, findings.length === 0, `Grande Enseigne checkout accessibility: ${findings.join(' | ')}`);
    await page.click('#pay');
    await page.waitForSelector('#order-ref');
    for (let attempt = 0; attempt < 20 && (await page.locator('#claim-url').count()) === 0; attempt += 1) {
      await page.waitForTimeout(250);
      await page.reload();
    }
    assert.match(await text(page, '#order-ref'), /^[A-Z]+-\d{4}-\d{6}$/);
    assert.equal(await page.getAttribute('#gift-status', 'data-status'), 'created');
    gift.claimUrl = await page.getAttribute('#claim-url', 'href');
    assert.ok(gift.claimUrl.startsWith(`${world.server}/k/`), `the claim URL is on the eBook Cadeau server: ${gift.claimUrl}`);
    assert.equal(await page.locator('#card-pdf').count(), 1, 'the card to print is offered');
    gift.orderUrl = page.url();
    await shot(page, '4-grande-enseigne-order');
  });

  test('4b. the recipient: redirect, account, activation, library', async (t) => {
    if (!gift.claimUrl) return t.skip('needs the gift of 4a');
    const { context, problems } = await newContext(browser, { ...devices['iPhone 15'] });
    const page = await context.newPage();
    await page.goto(gift.claimUrl);
    assert.equal(await text(page, '#gift-title'), 'Candide');
    assert.match(await text(page, '#gift-from'), new RegExp(GIVER));
    assert.equal(await text(page, '#gift-message'), MESSAGE);
    assert.equal(await text(page, '#redeem'), 'Ajouter à ma bibliothèque Grande Enseigne (démo)');
    assert.equal(await page.locator('#download').count(), 0, 'nothing to download: the book stays in the store ecosystem');
    assert.equal(await overflowX(page), 0, 'no horizontal scrolling on the phone');
    await shot(page, '4-phone-reseller-gift-page');

    // The signed redirect lands on the store, which drops the signature from the URL.
    const landings = [];
    page.on('request', (request) => {
      if (request.url().startsWith(`${world.grandeEnseigne}/cadeau/activer?`)) landings.push(request.url());
    });
    const referers = [];
    page.on('request', (request) => {
      const referer = request.headers().referer;
      if (referer && referer.includes('/k/') && !request.url().startsWith(world.server)) referers.push(`${request.url()} <- ${referer}`);
    });
    await page.click('#redeem');
    await page.waitForURL(`${world.grandeEnseigne}/cadeau/activer`);
    assert.equal(landings.length, 1, 'one signed landing on the store');
    const signed = new URL(landings[0]);
    for (const name of ['ebc_gift', 'ebc_order', 'ebc_ts', 'ebc_sig']) {
      assert.ok(signed.searchParams.get(name), `the redirect carries ${name}: ${landings[0]}`);
    }
    assert.deepEqual(referers, [], 'the claim URL (a bearer secret) must not leak to the store through Referer');
    assert.equal(await text(page, '#activation-giver'), GIVER);
    assert.match(await text(page, '#activation-book'), /Candide/);
    await shot(page, '4-phone-store-sign-in');

    await page.fill('#register-email', 'noa@example.org');
    await page.fill('#register-password', 'motdepasse');
    await page.click('#register-submit');
    await page.waitForSelector('#activate');
    assert.equal(await text(page, '#activation-account'), 'noa@example.org');
    await shot(page, '4-phone-store-confirm');
    await page.click('#activate');
    await page.waitForSelector('#activation-done');
    assert.match(await text(page, '#activation-book'), /Candide/);
    assert.match(await text(page, '#activation-sync'), /synchronisation/);
    await shot(page, '4-phone-store-added');
    gift.activated = true;

    await page.click('#nav-library');
    await page.waitForSelector('#library');
    const row = await text(page, '#owned-candide');
    assert.match(row, /Candide/);
    assert.match(row, new RegExp(`Offert par ${GIVER}`));
    assert.match(await text(page, '#sync-note'), /Synchronisez votre liseuse/);
    await shot(page, '4-phone-store-library');
    const findings = await audit(page);
    known(t, findings.length === 0, `Grande Enseigne library accessibility (phone): ${findings.join(' | ')}`);

    // Back on the gift page: fulfilled, with a way to the store.
    await page.goto(gift.claimUrl);
    assert.equal(await page.locator('#redeem').count(), 0, 'a fulfilled gift cannot be redeemed again');
    assert.equal(await page.locator('#store-link').count(), 1, 'the page links to the store');
    await shot(page, '4-phone-reseller-gift-fulfilled');
    assert.deepEqual(problems, [], 'the recipient: console errors or requests to other hosts');
    await context.close();
  });

  test('4c. the buyer sees the book was added', async (t) => {
    if (!gift.activated) return t.skip('needs the activation of 4b');
    const page = await buyer.context.newPage();
    // Fulfilment reaches the store by webhook: allow it a moment.
    let status = null;
    for (let attempt = 0; attempt < 40; attempt += 1) {
      await page.goto(gift.orderUrl);
      status = await page.getAttribute('#gift-status', 'data-status');
      if (status === 'fulfilled') break;
      await page.waitForTimeout(250);
    }
    assert.equal(status, 'fulfilled', 'the order page says the gift is in the recipient\'s library');
    assert.equal(await text(page, '#gift-status'), 'Ajouté à la bibliothèque de son destinataire');
    assert.equal(await page.locator('#cancel-gift').count(), 0, 'a fulfilled gift can no longer be cancelled');
    await shot(page, '4-grande-enseigne-order-fulfilled');
    assert.deepEqual(buyer.problems, [], 'the buyer: console errors or requests to other hosts');
  });
});
