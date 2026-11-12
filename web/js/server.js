// The server's creator API (docs/api.md §3). Every call goes to the server this page
// was configured with; nothing else is contacted.

/** An error answered by the server (`code` from its JSON body), or `unreachable`. */
export class ServerError extends Error {
  constructor(code, message, status = 0) {
    super(message);
    this.code = code;
    this.status = status;
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
    try {
      const body = await response.json();
      if (body && body.error) {
        code = String(body.error.code || code);
        message = String(body.error.message || message);
      }
    } catch {
      // not JSON: keep the status
    }
    throw new ServerError(code, message, response.status);
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
 * printer. Returns `{order_id, status, price_cents, currency}`.
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
