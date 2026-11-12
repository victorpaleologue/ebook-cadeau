// The accessibility audit of lib/harness.mjs is home-made: this checks that it finds what
// it claims to find (and stays quiet on a clean page), so a green suite means something.

import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { audit, launchBrowser, newContext } from '../lib/harness.mjs';

let browser;
before(async () => { browser = await launchBrowser(); });
after(async () => { await browser?.close(); });

const CLEAN = `<!doctype html><html lang="fr"><head><title>Propre</title></head>
<body style="background:#fff;color:#222"><h1>Titre</h1>
<label for="a">Nom</label><input id="a"><button>Envoyer</button><img src="data:," alt="">
<p style="opacity:.75">Texte atténué mais lisible</p><button disabled style="opacity:.3">Inactif</button></body></html>`;

const DIRTY = `<!doctype html><html><head><title></title></head>
<body style="background:#fff"><input id="a"><button></button><img src="data:,">
<p style="color:#aaa">Gris clair</p><div style="background:#000"><p style="color:#333">Sombre sur noir</p></div>
<p style="color:#000;opacity:.3">Trop transparent</p><h1 style="color:#888;font-size:30px">Grand titre gris</h1></body></html>`;

test('the audit is quiet on a clean page and finds every planted problem', async () => {
  const { context } = await newContext(browser, { javaScriptEnabled: false });
  const page = await context.newPage();
  await page.setContent(CLEAN);
  assert.deepEqual(await audit(page), []);

  await page.setContent(DIRTY);
  const findings = (await audit(page)).join('\n');
  for (const expected of [
    /html has no lang/, /no title/, /form control without a label: <input#a>/, /button without an accessible name/,
    /img without alt/, /contrast 2\.\d+:1 < 4\.5:1 .*Gris clair/, /contrast 1\.\d+:1 < 4\.5:1 .*Sombre sur noir/,
    /Trop transparent/,
  ]) {
    assert.match(findings, expected);
  }
  // #888 on white is 3.54:1: enough for 30 px text.
  assert.doesNotMatch(findings, /Grand titre gris/);
  await context.close();
});
