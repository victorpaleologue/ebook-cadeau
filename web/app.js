// eBook cadeau — the card creator. Plain ES module, no dependency, no external request
// of its own: the WebAssembly engine runs in ./worker.js; the only network calls are the
// catalogue lookup (when the user asks for it) and the preview image of a looked-up cover
// (never drawn into the card). A configured shelf (`shelf_url`) is only linked to.
//
// Every text inserted into the page goes through `textContent` or an attribute: book
// metadata from catalogues is attacker-controlled.

import { Engine } from './js/engine.js';
import { Strings, applyStrings, matchLocale } from './js/i18n.js';
import { lookupBook } from './js/lookup.js';
import { ServerError, orderPrint } from './js/server.js';

const $ = (id) => document.getElementById(id);
const TOTAL_STEPS = 6;
/** A well-formed code used for the preview until a real one exists (never downloadable). */
const SAMPLE_CODE = 'BCDF-GHJK-LMNL';
/** The claim-URL base of the demonstration code (a reserved, non-resolving domain). */
const DEMO_BASE_URL = 'https://ebk.example';
/** A valid ISBN, only used to learn which providers a lookup would ask. */
const PROBE_ISBN = '9780306406157';
const PREVIEW_DELAY_MS = 180;
/**
 * Countries offered for a printed card (ISO 3166-1 alpha-2), named in the page's language
 * by `Intl.DisplayNames`. The server accepts any alpha-2 code; whether the printer posts
 * there is the printer's answer (`unsupported_country`).
 */
const ORDER_COUNTRIES = ['FR', 'BE', 'LU', 'MC', 'CH', 'DE', 'AT', 'NL', 'ES', 'IT', 'PT', 'IE', 'GB', 'CA', 'US'];
const DEFAULT_COUNTRY = 'FR';
/** Address members (docs/api.md §3) → input ids. */
const ADDRESS_INPUTS = {
  name: 'order-name',
  line1: 'order-line1',
  line2: 'order-line2',
  postal_code: 'order-postal-code',
  city: 'order-city',
  country: 'order-country',
};
const REQUIRED_ADDRESS = ['name', 'line1', 'postal_code', 'city'];

const engine = new Engine(new URL('./worker.js', import.meta.url));

/** Everything the user chose. */
const state = {
  uiLocale: 'fr',
  cardLocale: null, // null: follows the page language
  isbn: null, // result of isbn_parse
  coverMode: 'typographic',
  cover: null, // {bytes, url, probe, analysis}
  rights: null, // id of RIGHTS_CHOICES
  adapt: false,
  spine: 'typeset',
  ebookMode: null,
  gift: null, // {code, display, base_url, claim_url?, id?, manage_token?, server}
  demoGift: null,
  codeGift: null,
  fontidAvailable: true,
  warnings: [],
};

let strings = new Strings({}, 'fr');
let options = null;
let config = {};
let sampleCode = SAMPLE_CODE;
let iccLoaded = false;

// ---- Small helpers --------------------------------------------------------------------

const t = (key, params) => strings.t(key, params);

function show(element, visible) {
  element.hidden = !visible;
}

function setText(element, text, kind) {
  element.textContent = text || '';
  element.classList.toggle('is-error', kind === 'error');
  element.classList.toggle('is-ok', kind === 'ok');
}

function radioValue(name) {
  const checked = document.querySelector(`input[name="${name}"]:checked`);
  return checked ? checked.value : null;
}

/** A translated error: `err_<code>`, a key of the catalogue, or the English message. */
function errorText(error) {
  const code = error && error.code;
  if (code && strings.has(`err_${code}`)) return t(`err_${code}`);
  if (code === 'invalid_isbn') return t('isbn_invalid');
  if (code === 'unreachable') return t('ui_server_unreachable');
  if (error instanceof ServerError) return t('ui_server_refused', { reason: error.message || code });
  return (error && error.message) || String(error);
}

function formatBytes(size) {
  if (size < 1024 * 1024) return `${Math.max(1, Math.round(size / 1024))} kB`;
  return `${(size / 1024 / 1024).toFixed(1)} MB`;
}

function profileInfo(name = $('profile').value) {
  return options.profiles.find((p) => p.name === name);
}

function rightsInfo() {
  return options.rights.find((r) => r.id === state.rights) || null;
}

/** An image the user picked is part of the card (it is printed only with a statement). */
function coverActive() {
  return state.coverMode === 'own' && state.cover !== null;
}

function serverUrl() {
  const configured = config.server || (servedByServer() ? location.origin : null);
  if (!configured) return null;
  try {
    const url = new URL(configured, location.href);
    if (!url.pathname.endsWith('/')) url.pathname += '/';
    return url;
  } catch {
    return null;
  }
}

function servedByServer() {
  return /^https?:$/.test(location.protocol) && location.pathname.startsWith('/app/');
}

// ---- Start ----------------------------------------------------------------------------

function fatal(detail) {
  show($('boot'), false);
  show($('main'), false);
  show($('fatal'), true);
  $('fatal-detail').textContent = detail || '';
}

async function loadConfig() {
  let loaded = {};
  try {
    const response = await fetch('config.json', { cache: 'no-store', credentials: 'omit' });
    if (response.ok) loaded = await response.json();
  } catch {
    // no config file (or file://): the defaults below apply
  }
  // Catalogue base URLs come from config.json only (same origin, written by whoever
  // deploys the page). A query parameter used to override them, which let any crafted
  // link send the ISBN look-ups — and put what they return on the card — to any host.
  const catalog = { base_urls: {}, enable: [], disable: [], ...(loaded.catalog || {}) };
  return { ...loaded, catalog, shelf: shelfOf(loaded) };
}

/**
 * The shelf named by config.json (`shelf_url`, optional `shelf_name`): an http(s) URL, or
 * nothing. The designer only links to it: uploads, rights declarations and moderation
 * belong to the shelf (no cross-origin upload from here).
 */
function shelfOf(loaded) {
  if (!loaded || typeof loaded.shelf_url !== 'string' || !loaded.shelf_url) return null;
  try {
    const url = new URL(loaded.shelf_url, location.href);
    if (url.protocol !== 'https:' && url.protocol !== 'http:') return null;
    const name = typeof loaded.shelf_name === 'string' && loaded.shelf_name.trim() ? loaded.shelf_name.trim() : url.host;
    return { url: url.href, name };
  } catch {
    return null;
  }
}

/**
 * Values passed by a link, e.g. from a shelf's gift list:
 * `/app/#code=XXXX-XXXX-XXXX&title=…&author=…&lang=fr`. The fragment is preferred (a
 * browser never sends it to a server, so the code stays out of access logs); the same
 * names in the query string are accepted too. They are removed from the address bar
 * once read.
 */
function linkParams() {
  const query = new URLSearchParams(location.search);
  const hash = new URLSearchParams(location.hash.replace(/^#/, ''));
  const pick = (name) => {
    const value = hash.get(name) || query.get(name);
    return value ? value.slice(0, 300) : '';
  };
  return { code: pick('code'), title: pick('title'), author: pick('author'), lang: pick('lang') };
}

/** Removes the prefilled values from the address bar (the code is a bearer secret). */
function forgetLinkParams() {
  const url = new URL(location.href);
  for (const name of ['code', 'title', 'author']) url.searchParams.delete(name);
  url.hash = '';
  history.replaceState(null, '', url);
}

/** Applies `linkParams()` once the controls exist. */
async function applyLinkParams(params) {
  if (params.title) {
    $('title').value = params.title;
    $('title').dispatchEvent(new Event('input'));
  }
  if (params.author) {
    $('author').value = params.author;
    $('author').dispatchEvent(new Event('input'));
  }
  if (params.code) {
    const radio = document.querySelector('input[name="ebook-mode"][value="code"]');
    radio.checked = true;
    $('code').value = params.code;
    await onCodeInput();
  }
  if (params.code || params.title || params.author) forgetLinkParams();
}

async function main() {
  if (typeof WebAssembly !== 'object' || typeof Worker !== 'function') {
    fatal('');
    return;
  }
  try {
    await engine.start();
  } catch (error) {
    fatal(String(error && error.message || error));
    return;
  }
  try {
    config = await loadConfig();
    options = await engine.json('card_options');
    const supported = options.locales.map((l) => l.code);
    const params = new URLSearchParams(location.search);
    const linked = linkParams();
    state.uiLocale = matchLocale(params.get('lang') || linked.lang, supported) || options.defaults.locale;
    const sample = await engine.json('gift_code_parse', SAMPLE_CODE).catch(() => null);
    if (sample) sampleCode = sample.code;
    const probe = await engine.json('fontid_identify', new Uint8Array(0), '', '', null).catch(() => null);
    state.fontidAvailable = !(probe && probe.available === false);
    buildControls();
    await setLocale(state.uiLocale);
    bindEvents();
    await applyLinkParams(linked);
  } catch (error) {
    fatal(String(error && error.message || error));
    return;
  }
  $('version').textContent = engine.version;
  show($('boot'), false);
  show($('main'), true);
  show($('footer'), true);
  schedulePreview();
}

/** Fills the selects whose entries come from the package. */
function buildControls() {
  const fill = (select, entries) => {
    select.replaceChildren(...entries.map(([value, label]) => {
      const option = document.createElement('option');
      option.value = value;
      option.textContent = label;
      return option;
    }));
  };
  const locales = options.locales.map((l) => [l.code, l.name]);
  fill($('ui-locale'), locales);
  fill($('card-locale'), locales);
  fill($('book-language'), [['', ''], ...locales]);
  fill($('title-font'), options.title_fonts.map((f) => [f.id, `${f.family} ${f.style}`.trim()]));
  $('title-font').value = options.defaults.title_font;
  fill($('format'), options.formats.map((f) => [f.name, '']));
  $('format').value = options.defaults.format;
  fill($('profile'), options.profiles.map((p) => [p.name, '']));
  $('profile').value = options.defaults.profile;
  fill($('order-country'), ORDER_COUNTRIES.map((code) => [code, code]));
  $('order-country').value = DEFAULT_COUNTRY;
  const list = $('rights-list');
  list.replaceChildren(...options.rights.map((choice) => {
    const label = document.createElement('label');
    label.className = 'choice';
    const input = document.createElement('input');
    input.type = 'radio';
    input.name = 'rights';
    input.value = choice.id;
    const text = document.createElement('span');
    text.dataset.i18n = `ui_rights_${choice.id}`;
    label.append(input, ' ', text);
    return label;
  }));
  $('site').value = config.base_url || (servedByServer() ? location.origin : '');
}

async function setLocale(code) {
  const map = await engine.json('i18n_catalog', code);
  strings = new Strings(map, code);
  state.uiLocale = code;
  const locale = options.locales.find((l) => l.code === code);
  document.documentElement.lang = locale ? locale.bcp47 : code;
  document.title = `${t('product_name')} — ${t('create_title')}`;
  $('ui-locale').value = code;
  if (!state.cardLocale) $('card-locale').value = code;
  applyStrings(document, strings);
  nameCountries(locale ? locale.bcp47 : code);
  for (const element of document.querySelectorAll('.step-of')) {
    element.textContent = t('ui_step_of', { n: element.dataset.step, total: TOTAL_STEPS });
  }
  const url = new URL(location.href);
  url.searchParams.set('lang', code);
  history.replaceState(null, '', url);
  await describeLookupSources();
  refreshAll();
}

/** Names the delivery countries in the page's language, sorted, keeping the choice. */
function nameCountries(tag) {
  const select = $('order-country');
  const chosen = select.value || DEFAULT_COUNTRY;
  let names = null;
  try {
    names = new Intl.DisplayNames([tag], { type: 'region' });
  } catch {
    // no Intl.DisplayNames: the codes stay as they are
  }
  const items = [...select.options].map((option) => {
    let name = option.value;
    try {
      name = (names && names.of(option.value)) || option.value;
    } catch {
      // unknown region: keep the code
    }
    option.textContent = name;
    return option;
  });
  items.sort((a, b) => a.textContent.localeCompare(b.textContent, tag));
  select.replaceChildren(...items);
  select.value = chosen;
}

/** "The search queries, from your browser: …" with the providers a lookup would ask. */
async function describeLookupSources() {
  try {
    const plan = await engine.json('catalog_lookup_requests', PROBE_ISBN, JSON.stringify(config.catalog));
    const names = plan.map((p) => p.name).join(', ');
    $('lookup-privacy').textContent = names ? t('ui_lookup_privacy', { sources: names }) : '';
    $('lookup').disabled = plan.length === 0;
  } catch {
    $('lookup-privacy').textContent = '';
  }
}

/** Re-derives every dynamic text and control state from `state`. */
function refreshAll() {
  refreshBookLanguageLabel();
  refreshCover();
  refreshSpine();
  refreshEbook();
  refreshCardStep();
  renderWarnings();
  refreshPreviewLabels();
}

// ---- Step 1: the book -------------------------------------------------------------------

function refreshBookLanguageLabel() {
  const option = $('book-language').querySelector('option[value=""]');
  if (option) option.textContent = t('ui_language_auto');
}

async function onLookup() {
  const status = $('isbn-status');
  const raw = $('isbn').value.trim();
  $('isbn').removeAttribute('aria-invalid');
  if (!raw) {
    setText(status, t('isbn_invalid'), 'error');
    $('isbn').setAttribute('aria-invalid', 'true');
    return;
  }
  let isbn;
  try {
    isbn = await engine.json('isbn_parse', raw);
  } catch {
    setText(status, t('isbn_invalid'), 'error');
    $('isbn').setAttribute('aria-invalid', 'true');
    return;
  }
  state.isbn = isbn;
  $('isbn').value = isbn.hyphenated;
  setText(status, t('loading'));
  $('lookup').disabled = true;
  try {
    const result = await lookupBook(engine, isbn.isbn13, config.catalog);
    if (!result.book) {
      setText(status, t(result.answered ? 'book_not_found' : 'ui_lookup_failed'), 'error');
    } else {
      fillBook(result.book);
      setText(status, t('ui_found_in', { sources: result.sources.map((s) => s.name).join(', ') }), 'ok');
      await showLookupCover(result.book);
    }
  } catch (error) {
    setText(status, errorText(error), 'error');
  } finally {
    $('lookup').disabled = false;
    refreshSpine();
    schedulePreview();
  }
}

function fillBook(book) {
  $('title').value = book.title || '';
  $('subtitle').value = book.subtitle || '';
  $('author').value = (book.contributors || []).filter((c) => !c.role || c.role === 'author')
    .map((c) => c.name).join(' ; ');
  $('publisher').value = book.publisher || '';
  const language = matchLocale(book.language, options.locales.map((l) => l.code));
  $('book-language').value = language || '';
  validateTitle(false);
}

/** Shows where the cover can be *seen* — an `<img>` only, never drawn into the card. */
async function showLookupCover(book) {
  show($('lookup-cover'), false);
  let candidates = [];
  try {
    candidates = await engine.json('cover_candidates', JSON.stringify(book), '');
  } catch {
    return;
  }
  const preview = candidates.find((c) => c.role === 'preview' && /^https:\/\//.test(c.url));
  if (!preview) return;
  const img = $('lookup-cover-img');
  img.onerror = () => show($('lookup-cover'), false);
  img.onload = () => show($('lookup-cover'), img.naturalWidth > 1);
  img.alt = book.title || '';
  img.src = preview.url;
  $('lookup-cover-note').textContent = t('ui_preview_only', { source: preview.source_name || preview.source });
}

function validateTitle(report) {
  const empty = !$('title').value.trim();
  show($('title-error'), empty && report);
  if (empty && report) $('title').setAttribute('aria-invalid', 'true');
  else $('title').removeAttribute('aria-invalid');
  return !empty;
}

function bookJson() {
  const author = $('author').value.trim();
  const book = {
    title: $('title').value.trim(),
    contributors: author ? author.split(/\s*;\s*/).filter(Boolean).map((name) => ({ name })) : [],
  };
  const subtitle = $('subtitle').value.trim();
  const publisher = $('publisher').value.trim();
  const language = $('book-language').value;
  if (subtitle) book.subtitle = subtitle;
  if (publisher) book.publisher = publisher;
  if (language) book.language = language;
  if (state.isbn) book.isbn13 = state.isbn.isbn13;
  return book;
}

// ---- Step 2: the cover ------------------------------------------------------------------

async function onCoverFile() {
  const file = $('cover-file').files[0];
  show($('cover-error'), false);
  if (!file) return;
  if (file.size > options.limits.max_cover_bytes) {
    coverError('ui_cover_too_large');
    return;
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  let probe;
  try {
    probe = await engine.json('cover_probe', bytes, $('format').value, $('profile').value);
  } catch {
    coverError('ui_cover_unreadable');
    return;
  }
  if (!probe.usable_for_card) {
    coverError('ui_cover_unreadable');
    return;
  }
  if (probe.placeholder) {
    coverError('ui_cover_placeholder');
    return;
  }
  await engine.put('cover', bytes);
  if (state.cover) URL.revokeObjectURL(state.cover.url);
  state.cover = { bytes, url: URL.createObjectURL(file), probe, analysis: null };
  try {
    state.cover.analysis = await engine.json('analyze_cover', { slot: 'cover' });
  } catch (error) {
    if (error.code === 'cover_too_large') {
      await removeCover();
      coverError('ui_cover_too_large');
      return;
    }
    if (error.code === 'cover_decode') {
      await removeCover();
      coverError('ui_cover_unreadable');
      return;
    }
  }
  setText($('font-result'), '');
  refreshAll();
  schedulePreview();
}

function coverError(key) {
  const element = $('cover-error');
  element.textContent = t(key);
  show(element, true);
  $('cover-file').value = '';
}

async function removeCover() {
  if (state.cover) URL.revokeObjectURL(state.cover.url);
  state.cover = null;
  state.adapt = false;
  $('adapt').checked = false;
  $('cover-file').value = '';
  await engine.put('cover', null);
  setText($('font-result'), '');
  refreshAll();
  schedulePreview();
}

async function reprobeCover() {
  if (!state.cover) return;
  try {
    state.cover.probe = await engine.json('cover_probe', { slot: 'cover' }, $('format').value, $('profile').value);
  } catch {
    // keep the previous verdict
  }
  refreshCover();
}

function refreshCover() {
  state.coverMode = radioValue('cover-mode') || 'typographic';
  state.rights = radioValue('rights');
  show($('own-image'), state.coverMode === 'own');
  const info = $('cover-info');
  show(info, state.cover !== null);
  if (state.cover) {
    const { probe } = state.cover;
    $('cover-thumb').src = state.cover.url;
    $('cover-size').textContent = t('ui_image_size', { width: probe.width, height: probe.height });
    const verdict = probe.quality.verdict;
    setText($('cover-quality'), t(`ui_quality_${verdict}`, { ppi: Math.round(probe.quality.effective_ppi) }),
      verdict === 'insufficient' ? 'error' : verdict === 'excellent' || verdict === 'acceptable' ? 'ok' : undefined);
  }
  const notes = [];
  const rights = rightsInfo();
  if (!rights) notes.push(t('ui_rights_required'));
  else {
    if (!rights.professional_order) notes.push(t('ui_own_printer_only'));
    if (rights.basis === 'public_domain') notes.push(t('ui_public_domain_note'));
  }
  $('rights-note').textContent = notes.join(' ');
}

// ---- Step 3: the spine ------------------------------------------------------------------

/** Whether the declared basis lets the artwork be adapted (once the user also says so). */
function adaptationPossible() {
  const rights = rightsInfo();
  return coverActive() && rights !== null && rights.adaptation;
}

function refreshSpine() {
  const possible = adaptationPossible();
  if (!possible) state.adapt = false;
  $('adapt').checked = state.adapt;
  $('adapt').disabled = !possible;
  show($('adapt-unavailable'), !possible);
  // Styles that sample the image appear only once the user has said they may adapt it.
  show($('spine-styles'), state.adapt);
  if (!state.adapt) state.spine = 'typeset';
  for (const input of document.querySelectorAll('input[name="spine"]')) {
    input.checked = input.value === state.spine;
  }
  // Colours of the image as card colours: also an adaptation.
  const swatches = $('swatches');
  const palette = state.adapt && state.cover && state.cover.analysis ? state.cover.analysis.palette : [];
  swatches.replaceChildren(...palette.slice(0, 6).map((swatch) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'swatch';
    button.style.backgroundColor = swatch.color;
    button.setAttribute('aria-label', `${t('ui_ground_colour')} ${swatch.color}`);
    button.title = swatch.color;
    button.addEventListener('click', () => {
      $('ground-auto').checked = false;
      $('ground').disabled = false;
      $('ground').value = swatch.color;
      schedulePreview();
    });
    return button;
  }));
  show(swatches, palette.length > 0);

  // Font detection works on an image the user supplied.
  const detect = $('font-detect');
  const result = $('font-result');
  const needsImage = t('ui_font_needs_image');
  if (!state.fontidAvailable) {
    detect.disabled = true;
    setText(result, t('ui_font_unavailable'));
  } else if (!coverActive()) {
    detect.disabled = true;
    setText(result, needsImage);
  } else {
    detect.disabled = false;
    if (result.textContent === needsImage) setText(result, '');
  }
  refreshDirection();
}

function refreshDirection() {
  const byLocale = (code) => {
    const locale = options.locales.find((l) => l.code === code);
    return locale ? locale.spine_direction : null;
  };
  const direction = byLocale($('book-language').value) ||
    (state.isbn && state.isbn.spine_direction) ||
    byLocale(state.cardLocale || state.uiLocale) || 'bottom_to_top';
  $('direction').textContent = t('ui_direction_is', { direction: t(`ui_direction_${direction}`) });
}

async function onFontDetect() {
  if (!coverActive()) return;
  const result = $('font-result');
  setText(result, t('loading'));
  $('font-detect').disabled = true;
  try {
    const found = await engine.json('fontid_identify', { slot: 'cover' }, '', $('title').value.trim(), null);
    if (!found.available) {
      state.fontidAvailable = false;
      setText(result, t('ui_font_unavailable'));
    } else if (found.suggestion && found.suggestion.title_font) {
      $('title-font').value = found.suggestion.title_font;
      const family = found.suggestion.family;
      setText(result, t(found.suggestion.confident ? 'ui_font_suggested' : 'ui_font_uncertain', { family }), 'ok');
      schedulePreview();
    } else if (found.matches && found.matches.length) {
      setText(result, t('ui_font_uncertain', { family: found.matches[0].family }));
    } else {
      setText(result, t('ui_font_none'));
    }
  } catch (error) {
    setText(result, errorText(error), 'error');
  } finally {
    $('font-detect').disabled = !state.fontidAvailable || !coverActive();
  }
}

// ---- Step 5: the eBook ------------------------------------------------------------------

function siteAddress() {
  return $('site').value.trim();
}

function refreshEbook() {
  state.ebookMode = radioValue('ebook-mode');
  const mode = state.ebookMode;
  // A configured shelf is offered as a link: the shelf owns uploads, rights declarations
  // and moderation; it creates the gift with the core and shows its code, which the user
  // brings back here with "I already have a code" (or a link that fills it in).
  const shelf = config.shelf;
  show($('shelf-choice'), shelf !== null && shelf !== undefined);
  show($('ebook-shelf'), Boolean(shelf) && mode === 'shelf');
  if (shelf) {
    $('shelf-choice-text').textContent = t('ui_ebook_shelf', { shelf: shelf.name });
    $('shelf-help').textContent = t('ui_ebook_shelf_help', { shelf: shelf.name });
    $('shelf-link').textContent = t('ui_shelf_open', { shelf: shelf.name });
    $('shelf-link').href = shelf.url;
  }
  show($('ebook-code'), mode === 'code');
  show($('ebook-demo'), mode === 'demo');
  show($('site-field'), mode === 'code' || mode === 'demo');
  state.gift = mode === 'code' ? state.codeGift : mode === 'demo' ? state.demoGift : null;
  const result = $('gift-result');
  show(result, state.gift !== null);
  if (state.gift) {
    $('gift-code').textContent = state.gift.server ? t('ui_gift_created', { code: state.gift.display }) : t('ui_code_ok', { code: state.gift.display });
    $('gift-key').textContent = state.gift.manage_token ? t('ui_manage_key', { key: state.gift.manage_token }) : '';
  }
}

async function onEbookMode() {
  const mode = radioValue('ebook-mode');
  if (mode === 'demo' && !state.demoGift) await makeDemoGift();
  if (mode === 'code') await onCodeInput();
  refreshEbook();
  refreshCardStep();
  schedulePreview();
}

async function makeDemoGift() {
  for (let attempt = 0; attempt < 8; attempt += 1) {
    const random = new Uint8Array(16);
    crypto.getRandomValues(random);
    try {
      const code = await engine.json('gift_code_from_random', random);
      state.demoGift = { ...code, base_url: '', server: false, demo: true };
      return;
    } catch (error) {
      if (error.code !== 'unfortunate_code') throw error;
    }
  }
}

/** The claim-URL base of a typed code or the demo code; `null` when it is not valid. */
async function validBase(code, fallback) {
  const base = siteAddress() || fallback;
  show($('site-error'), false);
  $('site').removeAttribute('aria-invalid');
  if (!base) return null;
  try {
    await engine.json('claim_urls', base, code);
    return base;
  } catch (error) {
    if (error.code === 'invalid_base_url') {
      show($('site-error'), true);
      $('site').setAttribute('aria-invalid', 'true');
    }
    return null;
  }
}

async function onCodeInput() {
  const status = $('code-status');
  const raw = $('code').value.trim();
  state.codeGift = null;
  $('code').removeAttribute('aria-invalid');
  if (!raw) {
    setText(status, '');
  } else {
    try {
      const code = await engine.json('gift_code_parse', raw);
      const base = await validBase(code.code, '');
      if (base) state.codeGift = { ...code, base_url: base, server: false };
      else show($('site-error'), true);
      setText(status, t('ui_code_ok', { code: code.display }), 'ok');
    } catch (error) {
      setText(status, errorText(error), 'error');
      $('code').setAttribute('aria-invalid', 'true');
    }
  }
  refreshEbook();
  refreshCardStep();
  schedulePreview();
}

async function onSiteInput() {
  if (state.ebookMode === 'code') {
    await onCodeInput();
    return;
  }
  if (state.ebookMode === 'demo' && state.demoGift) {
    const base = await validBase(state.demoGift.code, DEMO_BASE_URL);
    state.demoGift.base_url = base && siteAddress() ? base : '';
    refreshEbook();
    schedulePreview();
  }
}

// ---- Step 6: the card -------------------------------------------------------------------

function refreshCardStep() {
  const formatSelect = $('format');
  const profileSelect = $('profile');
  const rights = coverActive() ? rightsInfo() : null;
  const proRefused = rights !== null && !rights.professional_order;
  for (const option of profileSelect.options) {
    const profile = profileInfo(option.value);
    option.textContent = t(`ui_profile_${option.value}`);
    option.disabled = proRefused && profile.route === 'professional_order';
  }
  if (profileSelect.selectedOptions[0] && profileSelect.selectedOptions[0].disabled) {
    profileSelect.value = options.defaults.profile;
  }
  $('profile-note').textContent = proRefused ? t('ui_rights_pro_disabled') : '';
  const profile = profileInfo();
  const unfit = [];
  for (const option of formatSelect.options) {
    const format = options.formats.find((f) => f.name === option.value);
    const name = option.value === 'poche' ? 'Poche' : option.value.charAt(0).toUpperCase() + option.value.slice(1);
    option.textContent = `${name} — ${format.cover_mm[0]} × ${format.cover_mm[1]} mm`;
    option.disabled = !profile.formats.includes(option.value);
    if (option.disabled) unfit.push(name);
  }
  if (formatSelect.selectedOptions[0] && formatSelect.selectedOptions[0].disabled) {
    formatSelect.value = profile.formats.includes(options.defaults.format) ? options.defaults.format : profile.formats[0];
  }
  $('format-note').textContent = unfit.length ? `${unfit.join(', ')} — ${t('ui_format_unfit')}` : '';
  show($('duplex-hint'), profile.route === 'home_print');
  $('download').setAttribute('aria-describedby', 'download-status');

  // Ordering a printed card: a gift created here, a printer configured, a PDF/X profile.
  const note = $('order-note');
  const canOrder = Boolean(config.print_orders && serverUrl());
  if (!canOrder) {
    note.textContent = t('ui_order_unavailable');
    show($('order-form'), false);
  } else if (!(state.gift && state.gift.server && profile.pdfx)) {
    note.textContent = t('ui_order_needs_pro');
    show($('order-form'), false);
  } else {
    note.textContent = '';
    show($('order-form'), true);
  }
}

function renderWarnings() {
  const list = $('warnings');
  const items = state.warnings.map((warning) => {
    const li = document.createElement('li');
    const params = Object.fromEntries(warning.params || []);
    li.textContent = strings.has(`warn_${warning.code}`) ? t(`warn_${warning.code}`, params) : warning.message;
    return li;
  });
  if (!items.length) {
    const li = document.createElement('li');
    li.className = 'none';
    li.textContent = t('ui_no_warnings');
    items.push(li);
  }
  list.replaceChildren(...items);
}

/** The card request (see crates/ebc-wasm/src/api/card.rs). */
function cardRequest(forPdf) {
  const gift = state.gift;
  const profile = profileInfo();
  const request = {
    book: bookJson(),
    code: gift ? gift.code : sampleCode,
    base_url: (gift && gift.base_url) || siteAddress() || DEMO_BASE_URL,
    spine: state.spine,
    title_font: $('title-font').value,
    locale: state.cardLocale || state.uiLocale,
    format: $('format').value,
    profile: profile.name,
    guides: $('guides').checked,
  };
  if (gift && gift.claim_url) request.claim_url = gift.claim_url;
  if (coverActive()) {
    const rights = rightsInfo();
    request.cover = rights
      ? { rights: rights.basis, declaration: rights.id, adaptation_authorised: state.adapt }
      : { rights: 'none' };
    const credit = $('credit').value.trim();
    if (credit) request.cover.credit = credit;
  }
  if (!$('ground-auto').checked) request.ground = $('ground').value;
  for (const [field, id] of [['giver', 'giver'], ['recipient', 'recipient'], ['message', 'message']]) {
    const value = $(id).value.trim();
    if (value) request[field] = value;
  }
  if (forPdf && profile.pdfx) request.created_unix = Math.floor(Date.now() / 1000);
  return request;
}

const coverArgument = () => (coverActive() ? { slot: 'cover' } : null);

async function ensureIcc() {
  if (iccLoaded) return true;
  try {
    const response = await fetch('pkg/fogra39.icc', { credentials: 'omit' });
    if (!response.ok) return false;
    await engine.put('icc', new Uint8Array(await response.arrayBuffer()));
    iccLoaded = true;
  } catch {
    return false;
  }
  return true;
}

async function renderPdf() {
  const profile = profileInfo();
  const withIcc = profile.pdfx && (await ensureIcc());
  const value = await engine.call('render_card', JSON.stringify(cardRequest(true)), coverArgument(),
    withIcc ? { slot: 'icc' } : null);
  state.warnings = JSON.parse(value.warnings);
  renderWarnings();
  return value.pdf;
}

function fileName() {
  const slug = $('title').value.trim().normalize('NFKD').replace(/[̀-ͯ]/g, '')
    .toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 48) || 'carte';
  return `ebook-cadeau-${slug}-${$('format').value}.pdf`;
}

async function onDownload() {
  const status = $('download-status');
  const link = $('download-link');
  if (!validateTitle(true)) {
    setText(status, t('ui_title_required'), 'error');
    $('title').focus();
    return;
  }
  if (!state.gift) {
    setText(status, t('ui_need_code'), 'error');
    return;
  }
  setText(status, t('ui_generating'));
  $('download').disabled = true;
  try {
    const pdf = await renderPdf();
    const blob = new Blob([pdf], { type: 'application/pdf' });
    if (link.href) URL.revokeObjectURL(link.href);
    link.href = URL.createObjectURL(blob);
    link.download = fileName();
    link.textContent = fileName();
    show(link, true);
    setText(status, t('ui_pdf_ready', { size: formatBytes(blob.size) }), 'ok');
    link.click();
  } catch (error) {
    setText(status, errorText(error), 'error');
  } finally {
    $('download').disabled = false;
  }
}

/** Marks one address input (or none) as invalid, with `message` under the fieldset. */
function markAddress(member, message) {
  for (const [key, id] of Object.entries(ADDRESS_INPUTS)) {
    if (key === member) {
      $(id).setAttribute('aria-invalid', 'true');
      $(id).setAttribute('aria-describedby', 'order-address-error');
    } else {
      $(id).removeAttribute('aria-invalid');
      $(id).removeAttribute('aria-describedby');
    }
  }
  setText($('order-address-error'), message || '');
  show($('order-address-error'), Boolean(message));
  if (member && ADDRESS_INPUTS[member]) $(ADDRESS_INPUTS[member]).focus();
}

/** The address as the server takes it (docs/api.md §3), or `null` after marking a gap. */
function orderAddress() {
  const address = {};
  for (const [key, id] of Object.entries(ADDRESS_INPUTS)) {
    const value = $(id).value.trim();
    if (value) address[key] = key === 'country' ? value.toUpperCase() : value;
  }
  const missing = REQUIRED_ADDRESS.find((key) => !address[key]);
  if (missing) {
    markAddress(missing, t('ui_order_address_incomplete'));
    return null;
  }
  if (!address.country) address.country = DEFAULT_COUNTRY;
  markAddress(null, '');
  return address;
}

/** A translated text for a refused order. */
function orderErrorText(error) {
  const details = (error && error.details) || {};
  if (error && (error.code === 'printer_rejected' || error.code === 'preflight_failed')) {
    if (details.printer_code === 'unsupported_country') return t('ui_order_country_refused');
    const findings = Array.isArray(details.findings)
      ? details.findings.filter((f) => f && f.severity !== 'warning' && f.severity !== 'info')
        .map((f) => String(f.message || f.code || '')).filter(Boolean)
      : [];
    const reason = findings.length ? findings.slice(0, 3).join('; ')
      : String(details.printer_message || details.printer_code || error.message || error.code);
    return t('ui_order_rejected', { reason });
  }
  return errorText(error);
}

async function onOrder() {
  const status = $('order-status');
  if (!state.gift || !state.gift.server) return;
  const address = orderAddress();
  if (!address) {
    setText(status, '');
    return;
  }
  setText(status, t('ui_generating'));
  $('order-submit').disabled = true;
  try {
    const pdf = await renderPdf();
    const order = await orderPrint(serverUrl(), state.gift, pdf, `card-${$('format').value}`, address);
    setText(status, t('ui_order_sent', { id: order.order_id }), 'ok');
  } catch (error) {
    if (error && error.code === 'invalid_address') {
      const field = String((error.details && error.details.field) || '').replace(/^address\.?/, '');
      markAddress(ADDRESS_INPUTS[field] ? field : null, t('err_invalid_address'));
    }
    setText(status, orderErrorText(error), 'error');
  } finally {
    $('order-submit').disabled = false;
  }
}

// ---- The live preview -------------------------------------------------------------------

let previewTimer = null;
let previewRunning = false;
let previewAgain = false;
const previewUrls = [null, null];

function schedulePreview() {
  clearTimeout(previewTimer);
  previewTimer = setTimeout(runPreview, PREVIEW_DELAY_MS);
}

function refreshPreviewLabels() {
  const title = $('title').value.trim();
  $('preview-outside').alt = title ? `${t('ui_preview_outside')} — ${title}` : t('ui_preview_outside');
  $('preview-inside').alt = title ? `${t('ui_preview_inside')} — ${title}` : t('ui_preview_inside');
}

async function runPreview() {
  if (previewRunning) {
    previewAgain = true;
    return;
  }
  const status = $('preview-status');
  refreshPreviewLabels();
  if (!$('title').value.trim()) {
    setText(status, t('ui_preview_empty'));
    for (const id of ['preview-outside', 'preview-inside']) $(id).removeAttribute('src');
    document.querySelector('.preview-pane').classList.add('is-empty');
    return;
  }
  previewRunning = true;
  setText(status, t('ui_preview_updating'));
  try {
    const preview = await engine.json('preview_card', JSON.stringify(cardRequest(false)), coverArgument());
    preview.pages.forEach((svg, index) => {
      const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
      const img = $(index === 0 ? 'preview-outside' : 'preview-inside');
      img.src = url;
      if (previewUrls[index]) URL.revokeObjectURL(previewUrls[index]);
      previewUrls[index] = url;
    });
    const pane = document.querySelector('.preview-pane');
    pane.classList.remove('is-empty');
    // For tests and styling: whether the outside shows an image of the user's.
    pane.dataset.image = preview.pages[0].includes('<image') ? 'yes' : 'no';
    state.warnings = preview.warnings;
    renderWarnings();
    setText(status, '');
  } catch (error) {
    setText(status, t('ui_preview_failed', { reason: errorText(error) }), 'error');
  } finally {
    previewRunning = false;
    if (previewAgain) {
      previewAgain = false;
      schedulePreview();
    }
  }
}

// ---- Events -----------------------------------------------------------------------------

function bindEvents() {
  const form = $('card-form');
  form.addEventListener('submit', (event) => event.preventDefault());
  $('ui-locale').disabled = false;
  $('ui-locale').addEventListener('change', () => setLocale($('ui-locale').value));
  $('lookup').addEventListener('click', onLookup);
  $('isbn').addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
      event.preventDefault();
      onLookup();
    }
  });
  $('isbn').addEventListener('input', () => {
    state.isbn = null;
    setText($('isbn-status'), '');
  });
  for (const id of ['title', 'subtitle', 'author', 'publisher', 'giver', 'recipient', 'message', 'credit']) {
    $(id).addEventListener('input', () => {
      if (id === 'title') validateTitle(false);
      schedulePreview();
    });
  }
  $('title').addEventListener('blur', () => validateTitle(true));
  $('book-language').addEventListener('change', () => {
    refreshDirection();
    schedulePreview();
  });
  for (const input of document.querySelectorAll('input[name="cover-mode"], input[name="rights"]')) {
    input.addEventListener('change', () => {
      refreshAll();
      schedulePreview();
    });
  }
  $('cover-file').addEventListener('change', onCoverFile);
  $('cover-remove').addEventListener('click', removeCover);
  $('adapt').addEventListener('change', () => {
    state.adapt = $('adapt').checked && adaptationPossible();
    refreshSpine();
    schedulePreview();
  });
  for (const input of document.querySelectorAll('input[name="spine"]')) {
    input.addEventListener('change', () => {
      state.spine = radioValue('spine') || 'typeset';
      schedulePreview();
    });
  }
  $('ground-auto').addEventListener('change', () => {
    $('ground').disabled = $('ground-auto').checked;
    schedulePreview();
  });
  $('ground').addEventListener('input', schedulePreview);
  $('title-font').addEventListener('change', schedulePreview);
  $('font-detect').addEventListener('click', onFontDetect);
  $('card-locale').addEventListener('change', () => {
    state.cardLocale = $('card-locale').value === state.uiLocale ? null : $('card-locale').value;
    refreshDirection();
    schedulePreview();
  });
  for (const input of document.querySelectorAll('input[name="ebook-mode"]')) {
    input.addEventListener('change', onEbookMode);
  }
  $('code').addEventListener('input', onCodeInput);
  $('site').addEventListener('change', onSiteInput);
  for (const id of ['format', 'profile']) {
    $(id).addEventListener('change', async () => {
      refreshCardStep();
      await reprobeCover();
      schedulePreview();
    });
  }
  $('guides').addEventListener('change', schedulePreview);
  $('download').addEventListener('click', onDownload);
  $('order-submit').addEventListener('click', onOrder);
  for (const id of Object.values(ADDRESS_INPUTS)) {
    $(id).addEventListener('input', () => $(id).removeAttribute('aria-invalid'));
  }
}

main();
