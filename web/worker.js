// The WebAssembly engine, in a module worker: composing a card takes a few hundred
// milliseconds and must never freeze the page.
//
// Protocol (see js/engine.js, the only sender):
//   page → worker  {id, fn, args}           fn is an export of the package, or
//                                            "put" (args: [slot, Uint8Array]) / "drop" (args: [slot])
//   worker → page  {type: "ready", version} once the module is instantiated
//                  {type: "init_error", message}
//                  {id, ok: true, value}     value is a string, or {pdf, warnings} for render_card
//                  {id, ok: false, error: {code, message}, fatal}
//
// Byte arguments that are reused (the cover image, the ICC profile) are stored once in a
// named slot and referred to as {slot: "name"}, so they are not copied at every keystroke.
// A Rust panic leaves the instance in an unknown state: the answer is marked `fatal` and
// the page replaces this worker.

import init, * as wasm from './pkg/ebc_wasm.js';

/** The functions the page may call. Anything else is refused. */
const EXPORTS = new Set([
  'version', 'isbn_parse', 'catalog_providers', 'catalog_lookup_requests', 'catalog_parse',
  'catalog_merge', 'cover_candidates', 'cover_probe', 'scrape_extract', 'analyze_cover',
  'fontid_identify', 'gift_code_from_random', 'gift_code_parse', 'claim_urls', 'qr_svg',
  'card_options', 'preview_card', 'preview_svg', 'i18n_catalog', 'i18n_t',
]);

const slots = new Map();

const ready = init({ module_or_path: new URL('./pkg/ebc_wasm_bg.wasm', import.meta.url) })
  .then(() => wasm.version());

ready.then(
  (version) => self.postMessage({ type: 'ready', version }),
  (error) => self.postMessage({ type: 'init_error', message: String((error && error.message) || error) }),
);

/** `{slot: name}` stands for the bytes stored under that name (`null` when empty). */
function resolve(arg) {
  if (arg && typeof arg === 'object' && !ArrayBuffer.isView(arg) && typeof arg.slot === 'string') {
    return slots.has(arg.slot) ? slots.get(arg.slot) : null;
  }
  return arg;
}

/** Errors of the API read "[code] message"; a panic reads "panic: …". */
function describe(error) {
  const message = String((error && error.message) || error);
  const fatal = message.startsWith('panic:') ||
    (typeof WebAssembly !== 'undefined' && error instanceof WebAssembly.RuntimeError);
  const match = /^\[([a-z0-9_]+)\] ([\s\S]*)$/.exec(message);
  if (match) return { error: { code: match[1], message: match[2] }, fatal };
  return { error: { code: fatal ? 'panic' : 'internal', message }, fatal };
}

self.onmessage = async (event) => {
  const { id, fn, args = [] } = event.data || {};
  try {
    await ready;
    if (fn === 'put') {
      if (typeof args[0] !== 'string' || !(args[1] instanceof Uint8Array)) throw new Error('[bad_request] put(slot, bytes)');
      slots.set(args[0], args[1]);
      self.postMessage({ id, ok: true, value: null });
    } else if (fn === 'drop') {
      slots.delete(args[0]);
      self.postMessage({ id, ok: true, value: null });
    } else if (fn === 'render_card') {
      const card = wasm.render_card(...args.map(resolve));
      const pdf = card.pdf;
      const warnings = card.warnings;
      card.free();
      self.postMessage({ id, ok: true, value: { pdf, warnings } }, [pdf.buffer]);
    } else if (EXPORTS.has(fn)) {
      self.postMessage({ id, ok: true, value: wasm[fn](...args.map(resolve)) });
    } else {
      throw new Error(`[unknown_function] ${String(fn)}`);
    }
  } catch (error) {
    self.postMessage({ id, ok: false, ...describe(error) });
  }
};
