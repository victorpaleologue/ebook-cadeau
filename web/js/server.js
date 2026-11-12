// The server's former creator API (docs/api.md v0.2 §3). Every call goes to the server this
// page was configured with; nothing else is contacted.
//
// The core service removed this API together with file hosting (docs/api.md v0.3): gifts
// are created by partners only (a store, or a shelf service hosting files). The designer
// no longer reaches `createGift` or `orderPrint` (the upload mode is hidden), and the
// server answers 404 to these routes. TODO(shelf): point both at the shelf service.

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
 * Uploads the eBook file with its rights declaration, then creates the gift.
 * Returns the creation response: `{id, code, code_display, claim_url, manage_token}`.
 */
export async function createGift(serverUrl, file, rightsBasis, gift) {
  const upload = await send(new URL('api/v1/uploads', serverUrl), {
    method: 'POST',
    headers: {
      'Content-Type': file.type || 'application/octet-stream',
      'X-File-Name': encodeURIComponent(file.name),
      'X-Rights-Basis': rightsBasis,
    },
    body: file,
  });
  return send(new URL('api/v1/gifts', serverUrl), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ...gift, delivery: { type: 'hosted_file', upload_id: upload.upload_id } }),
  });
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
