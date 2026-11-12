// The server's former creator API (docs/api.md v0.2 §3). Every call goes to the server this
// page was configured with; nothing else is contacted.
//
// The core service removed this API together with file hosting (docs/api.md v0.3): gifts
// are created by partners only (a store, or a shelf service hosting files — the shelf
// owns uploads; the designer only links to it, see `shelf_url` in web/README.md). The
// designer no longer reaches `orderPrint` (no gift is created on this server any more);
// print orders go through the partner API (docs/api.md §4.6), from the partner.

/**
 * An error answered by the server (`code` from its JSON body), or `unreachable`.
 * `details` is the whole `error` object of the body (e.g. `field` of `invalid_address`,
 * `printer_code` and `findings` of `printer_rejected` / `preflight_failed`).
 */
export class ServerError extends Error {
  constructor(code, message, status = 0, details = {}) {
    super(message);
    this.code = code;
    this.status = status;
    this.details = details;
  }
}

async function send(url, init) {
  let response;
  try {
    response = await fetch(url, { credentials: 'omit', referrerPolicy: 'no-referrer', cache: 'no-store', ...init });
  } catch (error) {
    throw new ServerError('unreachable', String((error && error.message) || error));
  }
  if (!response.ok) {
    let code = `http_${response.status}`;
    let message = response.statusText || code;
    let details = {};
    try {
      const body = await response.json();
      if (body && body.error && typeof body.error === 'object') {
        details = body.error;
        code = String(body.error.code || code);
        message = String(body.error.message || message);
      }
    } catch {
      // not JSON: keep the status
    }
    throw new ServerError(code, message, response.status, details);
  }
  return response.json();
}

/**
 * Stores the card PDF with the gift, then orders a printed copy from the configured
 * printer. `address` is `{name, line1, line2?, postal_code, city, country}` (docs/api.md
 * §3; `country` is an ISO 3166-1 alpha-2 code). Returns
 * `{order_id, status, price_cents, currency}`.
 *
 * Errors (`ServerError`): `invalid_address` (400, `details.field` names the member),
 * `preflight_failed` / `printer_rejected` (422, the printer refused: `details.printer_code`,
 * `details.printer_message`, `details.findings`), `printer_unavailable` (502).
 */
export async function orderPrint(serverUrl, gift, pdf, product, address) {
  const auth = { Authorization: `Bearer ${gift.manage_token}` };
  const id = encodeURIComponent(gift.id);
  const receipt = await send(new URL(`api/v1/gifts/${id}/card.pdf`, serverUrl), {
    method: 'PUT',
    headers: { ...auth, 'Content-Type': 'application/pdf' },
    body: pdf,
  });
  return send(new URL(`api/v1/gifts/${id}/print-orders`, serverUrl), {
    method: 'POST',
    headers: { ...auth, 'Content-Type': 'application/json' },
    body: JSON.stringify({ product, pdf_sha256: receipt.sha256, address }),
  });
}
