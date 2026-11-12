// The catalogue lookup, sans-IO: the WebAssembly package says what to fetch and reads
// the answers; this module only performs the `fetch` calls, politely and in order.

/** Most follow-up requests per provider (a search result pointing at a record, …). */
const MAX_FOLLOW_UPS = 4;
/** Per-request time limit, in milliseconds. */
const REQUEST_TIMEOUT_MS = 12_000;

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function timeoutSignal(ms) {
  if (typeof AbortSignal !== 'undefined' && typeof AbortSignal.timeout === 'function') {
    return AbortSignal.timeout(ms);
  }
  return undefined;
}

/**
 * Looks `isbn13` up through the providers a page may call. Providers are asked one after
 * the other, in the package's priority order, and the search stops as soon as the merged
 * record has a title and a contributor.
 *
 * Returns `{book, sources, answered}`: the merged `BookMeta` (or `null`), the providers
 * that contributed a record, and whether any provider answered at all (to tell "not
 * found" from "no answer").
 */
export async function lookupBook(engine, isbn13, config, fetchImpl = globalThis.fetch.bind(globalThis)) {
  const configJson = JSON.stringify(config);
  const plan = await engine.json('catalog_lookup_requests', isbn13, configJson);
  const records = [];
  const sources = [];
  let answered = false;
  let book = null;
  for (const provider of plan) {
    const queue = [...provider.requests];
    const budget = provider.requests.length + MAX_FOLLOW_UPS;
    let sent = 0;
    let last = 0;
    let found = false;
    while (queue.length && sent < budget) {
      const request = queue.shift();
      sent += 1;
      const wait = last + (provider.min_interval_ms || 0) - Date.now();
      if (wait > 0) await sleep(wait);
      last = Date.now();
      let response;
      let body;
      try {
        response = await fetchImpl(request.url, {
          method: request.method,
          headers: request.headers,
          credentials: 'omit',
          referrerPolicy: 'no-referrer',
          cache: 'no-store',
          signal: timeoutSignal(REQUEST_TIMEOUT_MS),
        });
        body = new Uint8Array(await response.arrayBuffer());
      } catch {
        continue; // network error, CORS refusal or time-out: try the next one
      }
      const meta = {
        status: response.status,
        headers: [...response.headers],
        final_url: response.url || request.url,
      };
      let parsed;
      try {
        parsed = await engine.json('catalog_parse', provider.provider, isbn13, configJson,
          JSON.stringify(request.request), JSON.stringify(meta), body);
      } catch {
        continue;
      }
      answered = true;
      if (parsed.records.length) {
        records.push(...parsed.records);
        found = true;
      }
      queue.push(...parsed.follow_up);
    }
    if (found) sources.push({ id: provider.provider, name: provider.name, attribution: provider.attribution });
    if (records.length) {
      book = await engine.json('catalog_merge', JSON.stringify(records));
      if (book && book.title && book.contributors && book.contributors.length) break;
    }
  }
  return { book, sources, answered };
}
